use crate::cyk::{BackPointer, CykResult};
use crate::error::{Error, Result};
use std::fmt;

/// Árbol de derivación de la gramática CNF que realmente utilizó CYK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseTreeNode {
    Variable {
        name: String,
        children: Vec<ParseTreeNode>,
    },
    Terminal(String),
    Epsilon,
}

impl ParseTreeNode {
    pub fn label(&self) -> &str {
        match self {
            Self::Variable { name, .. } | Self::Terminal(name) => name,
            Self::Epsilon => "ε",
        }
    }

    /// Las hojas terminales deben coincidir exactamente con la entrada.
    pub fn yield_tokens(&self) -> Vec<String> {
        match self {
            Self::Terminal(t) => vec![t.clone()],
            Self::Epsilon => vec![],
            Self::Variable { children, .. } => {
                children.iter().flat_map(Self::yield_tokens).collect()
            }
        }
    }

    fn write_children(&self, f: &mut fmt::Formatter<'_>, prefix: &str) -> fmt::Result {
        if let Self::Variable { children, .. } = self {
            for (i, child) in children.iter().enumerate() {
                let last = i + 1 == children.len();
                writeln!(
                    f,
                    "{prefix}{}{}",
                    if last { "└── " } else { "├── " },
                    child.label()
                )?;
                child.write_children(
                    f,
                    &format!("{prefix}{}", if last { "    " } else { "│   " }),
                )?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for ParseTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.label())?;
        self.write_children(f, "")
    }
}

impl CykResult {
    /// Sigue un único backpointer por variable/celda; no repite el análisis.
    pub fn parse_tree(&self) -> Result<Option<ParseTreeNode>> {
        if !self.accepted {
            return Ok(None);
        }
        if self.tokens.is_empty() {
            return Ok(Some(ParseTreeNode::Variable {
                name: self.start.clone(),
                children: vec![ParseTreeNode::Epsilon],
            }));
        }
        self.build_node(&self.start, 0, self.tokens.len()).map(Some)
    }

    fn build_node(&self, variable: &str, start: usize, length: usize) -> Result<ParseTreeNode> {
        let invalid = || {
            Error(format!(
                "Backpointer inválido para {variable} en posición {start}, longitud {length}"
            ))
        };
        let pointer = length
            .checked_sub(1)
            .and_then(|column| {
                self.table
                    .get(start)
                    .and_then(|row| row.get(column))
                    .and_then(|cell| cell.get(variable))
            })
            .ok_or_else(invalid)?;
        let children = match pointer {
            BackPointer::Terminal(t) => {
                if length != 1 || self.tokens.get(start) != Some(t) {
                    return Err(invalid());
                }
                vec![ParseTreeNode::Terminal(t.clone())]
            }
            BackPointer::Binary { split, left, right } => {
                if *split == 0 || *split >= length {
                    return Err(invalid());
                }
                vec![
                    self.build_node(left, start, *split)?,
                    self.build_node(right, start + split, length - split)?,
                ]
            }
        };
        Ok(ParseTreeNode::Variable {
            name: variable.to_owned(),
            children,
        })
    }
}
