use super::check_size;
use crate::error::Result;
use crate::grammar::{Grammar, Production, Symbol};
use std::collections::{BTreeSet, VecDeque};

/// Cierre reflexivo/transitivo de A -> B. Visitados rompe cualquier ciclo.
pub fn unit_closure(grammar: &Grammar, variable: &str) -> BTreeSet<String> {
    let mut visited = BTreeSet::from([variable.to_owned()]);
    let mut pending = VecDeque::from([variable.to_owned()]);
    while let Some(current) = pending.pop_front() {
        for p in grammar.productions.iter().filter(|p| p.lhs == current) {
            if let [Symbol::Nonterminal(next)] = p.rhs.as_slice() {
                if visited.insert(next.clone()) {
                    pending.push_back(next.clone());
                }
            }
        }
    }
    visited
}

/// Cada variable hereda las reglas no unitarias de todo su cierre.
pub fn eliminate_unit(grammar: &Grammar) -> Result<Grammar> {
    let mut productions = BTreeSet::new();
    for variable in grammar.variables() {
        let closure = unit_closure(grammar, &variable);
        for p in &grammar.productions {
            if closure.contains(&p.lhs) && !p.is_unit() {
                productions.insert(Production::new(&variable, p.rhs.clone()));
                check_size(productions.len())?;
            }
        }
    }
    Ok(Grammar {
        start: grammar.start.clone(),
        productions,
    })
}
