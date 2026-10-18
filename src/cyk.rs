use crate::cnf::validate_cnf;
use crate::error::{Error, Result};
use crate::grammar::{Grammar, Symbol};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// Límite explícito de recursos; no modifica los resultados dentro del límite.
pub const MAX_TOKENS: usize = 512;

/// La clave de la celda es el LHS; estos datos identifican el RHS y sus hijos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackPointer {
    Terminal(String),
    Binary {
        split: usize,
        left: String,
        right: String,
    },
}

pub type Cell = BTreeMap<String, BackPointer>;

#[derive(Debug, Clone)]
pub struct CykResult {
    pub accepted: bool,
    pub elapsed: Duration,
    /// table[inicio][longitud - 1]; solo se asignan las celdas triangulares.
    pub table: Vec<Vec<Cell>>,
    pub tokens: Vec<String>,
    pub start: String,
}

/// CYK ascendente: longitudes, posiciones, divisiones y reglas binarias.
/// El cronómetro excluye validación, tokenización y reconstrucción del árbol.
pub fn recognize(grammar: &Grammar, tokens: &[String]) -> Result<CykResult> {
    validate_cnf(grammar)?;
    let n = tokens.len();
    if n > MAX_TOKENS {
        return Err(Error(format!(
            "La entrada tiene {n} tokens; el máximo de CYK es {MAX_TOKENS}"
        )));
    }
    let timer = Instant::now();
    let mut terminal_index: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut binary_rules = Vec::new();
    for p in &grammar.productions {
        match p.rhs.as_slice() {
            [Symbol::Terminal(t)] => terminal_index.entry(t).or_default().push(&p.lhs),
            [Symbol::Nonterminal(left), Symbol::Nonterminal(right)] => {
                binary_rules.push((p.lhs.as_str(), left.as_str(), right.as_str()));
            }
            _ => {}
        }
    }
    let mut table: Vec<Vec<Cell>> = (0..n).map(|i| vec![Cell::new(); n - i]).collect();
    for (i, token) in tokens.iter().enumerate() {
        if let Some(variables) = terminal_index.get(token.as_str()) {
            for variable in variables {
                table[i][0].insert((*variable).to_owned(), BackPointer::Terminal(token.clone()));
            }
        }
    }
    for length in 2..=n {
        for start in 0..=n - length {
            let mut cell = Cell::new();
            for split in 1..length {
                let left_cell = &table[start][split - 1];
                let right_cell = &table[start + split][length - split - 1];
                for &(lhs, left, right) in &binary_rules {
                    if left_cell.contains_key(left)
                        && right_cell.contains_key(right)
                        && !cell.contains_key(lhs)
                    {
                        cell.insert(
                            lhs.to_owned(),
                            BackPointer::Binary {
                                split,
                                left: left.to_owned(),
                                right: right.to_owned(),
                            },
                        );
                    }
                }
            }
            table[start][length - 1] = cell;
        }
    }
    let accepted = if n == 0 {
        grammar.accepts_empty()
    } else {
        table[0][n - 1].contains_key(&grammar.start)
    };
    let elapsed = timer.elapsed();
    Ok(CykResult {
        accepted,
        elapsed,
        table,
        tokens: tokens.to_vec(),
        start: grammar.start.clone(),
    })
}
