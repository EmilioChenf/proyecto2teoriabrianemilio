use std::collections::BTreeSet;
use std::fmt;

/// Los terminales y las variables son tipos distintos incluso si se parecen.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Nonterminal(String),
    Terminal(String),
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Self::Nonterminal(name) | Self::Terminal(name) => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Production {
    pub lhs: String,
    /// Un vector vacío representa epsilon.
    pub rhs: Vec<Symbol>,
}

impl Production {
    pub fn new(lhs: impl Into<String>, rhs: Vec<Symbol>) -> Self {
        Self {
            lhs: lhs.into(),
            rhs,
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self.rhs.as_slice(), [Symbol::Nonterminal(_)])
    }
}

/// Conjuntos ordenados: salida reproducible y ninguna producción duplicada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grammar {
    pub start: String,
    pub productions: BTreeSet<Production>,
}

impl Grammar {
    pub fn variables(&self) -> BTreeSet<String> {
        let mut result = BTreeSet::from([self.start.clone()]);
        for p in &self.productions {
            result.insert(p.lhs.clone());
            for s in &p.rhs {
                if let Symbol::Nonterminal(v) = s {
                    result.insert(v.clone());
                }
            }
        }
        result
    }

    pub fn terminals(&self) -> BTreeSet<String> {
        self.productions
            .iter()
            .flat_map(|p| &p.rhs)
            .filter_map(|s| {
                if let Symbol::Terminal(t) = s {
                    Some(t.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn start_on_rhs(&self) -> bool {
        self.productions.iter().any(|p| {
            p.rhs
                .iter()
                .any(|s| matches!(s, Symbol::Nonterminal(v) if v == &self.start))
        })
    }

    pub fn accepts_empty(&self) -> bool {
        self.productions
            .contains(&Production::new(&self.start, vec![]))
    }

    /// Incluye terminales para no cambiar su clasificación al guardar la CFG.
    pub(crate) fn fresh_name(&self, base: &str) -> String {
        let mut reserved = self.variables();
        reserved.extend(self.terminals());
        if !reserved.contains(base) {
            return base.to_owned();
        }
        for index in 1_u64.. {
            let candidate = format!("{base}_{index}");
            if !reserved.contains(&candidate) {
                return candidate;
            }
        }
        // El espacio de nombres finito en memoria no puede agotar u64.
        unreachable!("espacio de nombres agotado")
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut variables: Vec<_> = self.variables().into_iter().collect();
        variables.retain(|v| v != &self.start);
        variables.insert(0, self.start.clone());
        if self.productions.is_empty() {
            return writeln!(f, "∅ (lenguaje vacío; inicial: {})", self.start);
        }
        for v in variables {
            let alternatives: Vec<_> = self
                .productions
                .iter()
                .filter(|p| p.lhs == v)
                .map(|p| {
                    if p.rhs.is_empty() {
                        "e".to_owned()
                    } else {
                        p.rhs.iter().map(Symbol::name).collect::<Vec<_>>().join(" ")
                    }
                })
                .collect();
            if !alternatives.is_empty() {
                writeln!(f, "{v} -> {}", alternatives.join(" | "))?;
            }
        }
        Ok(())
    }
}
