use super::check_size;
use crate::error::Result;
use crate::grammar::{Grammar, Production, Symbol};
use std::collections::BTreeSet;

/// Menor punto fijo de las variables que derivan la cadena vacía.
pub fn nullable_variables(grammar: &Grammar) -> BTreeSet<String> {
    let mut nullable = BTreeSet::new();
    loop {
        let before = nullable.len();
        for p in &grammar.productions {
            if p.rhs
                .iter()
                .all(|s| matches!(s, Symbol::Nonterminal(v) if nullable.contains(v)))
            {
                nullable.insert(p.lhs.clone());
            }
        }
        if nullable.len() == before {
            return nullable;
        }
    }
}

/// Enumera omisiones de cada ocurrencia anulable, sin máscaras de bits.
/// Solo el inicial conserva epsilon; se aísla si también aparece en un RHS.
pub fn eliminate_epsilon(grammar: &Grammar) -> Result<Grammar> {
    let mut nullable = nullable_variables(grammar);
    let start_nullable = nullable.contains(&grammar.start);
    let mut source = grammar.clone();
    if start_nullable && source.start_on_rhs() {
        let new_start = source.fresh_name("S_INICIO");
        source.productions.insert(Production::new(
            &new_start,
            vec![Symbol::Nonterminal(source.start.clone())],
        ));
        nullable.insert(new_start.clone());
        source.start = new_start;
    }
    let mut result = Grammar {
        start: source.start.clone(),
        productions: BTreeSet::new(),
    };
    for p in &source.productions {
        if p.rhs.is_empty() {
            continue;
        }
        let mut variants = BTreeSet::from([Vec::new()]);
        for symbol in &p.rhs {
            let optional = matches!(symbol, Symbol::Nonterminal(v) if nullable.contains(v));
            let mut next = BTreeSet::new();
            for prefix in variants {
                let mut kept = prefix.clone();
                kept.push(symbol.clone());
                next.insert(kept);
                if optional {
                    next.insert(prefix);
                }
                check_size(next.len())?;
            }
            variants = next;
        }
        for rhs in variants {
            if !rhs.is_empty() {
                result.productions.insert(Production::new(&p.lhs, rhs));
            }
        }
        check_size(result.productions.len())?;
    }
    if start_nullable {
        result
            .productions
            .insert(Production::new(&result.start, vec![]));
    }
    Ok(result)
}
