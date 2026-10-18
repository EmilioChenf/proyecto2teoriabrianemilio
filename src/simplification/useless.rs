use crate::grammar::{Grammar, Symbol};
use std::collections::BTreeSet;

/// Variables que derivan alguna secuencia finita de terminales, incluido ε.
pub fn generating_variables(grammar: &Grammar) -> BTreeSet<String> {
    let mut generating = BTreeSet::new();
    loop {
        let before = generating.len();
        for p in &grammar.productions {
            if p.rhs.iter().all(|s| match s {
                Symbol::Terminal(_) => true,
                Symbol::Nonterminal(v) => generating.contains(v),
            }) {
                generating.insert(p.lhs.clone());
            }
        }
        if generating.len() == before {
            return generating;
        }
    }
}

pub fn reachable_variables(grammar: &Grammar) -> BTreeSet<String> {
    let mut reachable = BTreeSet::from([grammar.start.clone()]);
    loop {
        let before = reachable.len();
        for p in &grammar.productions {
            if reachable.contains(&p.lhs) {
                for s in &p.rhs {
                    if let Symbol::Nonterminal(v) = s {
                        reachable.insert(v.clone());
                    }
                }
            }
        }
        if reachable.len() == before {
            return reachable;
        }
    }
}

/// Primero elimina no generadores: hacerlo al revés puede dejar inalcanzables.
pub fn eliminate_useless(grammar: &Grammar) -> Grammar {
    let generating = generating_variables(grammar);
    let productive = Grammar {
        start: grammar.start.clone(),
        productions: grammar
            .productions
            .iter()
            .filter(|p| {
                generating.contains(&p.lhs)
                    && p.rhs.iter().all(|s| match s {
                        Symbol::Terminal(_) => true,
                        Symbol::Nonterminal(v) => generating.contains(v),
                    })
            })
            .cloned()
            .collect(),
    };
    let reachable = reachable_variables(&productive);
    Grammar {
        start: productive.start,
        productions: productive
            .productions
            .into_iter()
            .filter(|p| reachable.contains(&p.lhs))
            .collect(),
    }
}
