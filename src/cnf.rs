use crate::error::{Error, Result};
use crate::grammar::{Grammar, Production, Symbol};
use crate::simplification::{check_size, eliminate_epsilon, eliminate_unit, eliminate_useless};
use std::collections::BTreeMap;

/// Todas las etapas son inspeccionables sin volver a ejecutar conversiones.
#[derive(Debug, Clone)]
pub struct Conversion {
    pub originally_cnf: bool,
    pub epsilon_free: Grammar,
    pub unit_free: Grammar,
    pub simplified: Grammar,
    pub terminals_isolated: Grammar,
    pub cnf: Grammar,
}

/// CNF: A -> BC, A -> a; S -> ε únicamente si S no aparece a la derecha.
/// Sin epsilon se admite S en RHS, como en el archivo 1-cnf del profesor.
pub fn validate_cnf(grammar: &Grammar) -> Result<()> {
    if grammar.start.is_empty() {
        return Err(Error("Símbolo inicial inválido: nombre vacío".into()));
    }
    if grammar.accepts_empty() && grammar.start_on_rhs() {
        return Err(Error(
            "CNF inválida: el inicial con epsilon aparece en un lado derecho".into(),
        ));
    }
    for p in &grammar.productions {
        if p.lhs.is_empty() || p.rhs.iter().any(|s| s.name().is_empty()) {
            return Err(Error("CNF inválida: símbolo vacío".into()));
        }
        let valid = match p.rhs.as_slice() {
            [] => p.lhs == grammar.start,
            [Symbol::Terminal(_)] => true,
            [Symbol::Nonterminal(_), Symbol::Nonterminal(_)] => true,
            _ => false,
        };
        if !valid {
            return Err(Error(format!(
                "La producción {} -> {} no está en CNF",
                p.lhs,
                if p.rhs.is_empty() {
                    "e".into()
                } else {
                    p.rhs.iter().map(Symbol::name).collect::<Vec<_>>().join(" ")
                }
            )));
        }
    }
    Ok(())
}

pub fn is_cnf(grammar: &Grammar) -> bool {
    validate_cnf(grammar).is_ok()
}

fn terminal_base(terminal: &str) -> String {
    let mut name = "T_".to_owned();
    for c in terminal.chars() {
        if c.is_alphanumeric() || c == '_' {
            name.push(c);
        } else {
            name.push_str(&format!("u{:x}_", u32::from(c)));
        }
    }
    name
}

/// Una variable auxiliar compartida por terminal en RHS de longitud >= 2.
pub fn isolate_terminals(grammar: &Grammar) -> Result<Grammar> {
    let mut result = grammar.clone();
    let mut replacements: BTreeMap<String, String> = BTreeMap::new();
    for p in &grammar.productions {
        if p.rhs.len() < 2 {
            continue;
        }
        let mut rhs = Vec::new();
        for s in &p.rhs {
            if let Symbol::Terminal(t) = s {
                let variable = if let Some(v) = replacements.get(t) {
                    v.clone()
                } else {
                    let v = result.fresh_name(&terminal_base(t));
                    result
                        .productions
                        .insert(Production::new(&v, vec![s.clone()]));
                    replacements.insert(t.clone(), v.clone());
                    v
                };
                rhs.push(Symbol::Nonterminal(variable));
            } else {
                rhs.push(s.clone());
            }
        }
        result.productions.remove(p);
        result.productions.insert(Production::new(&p.lhs, rhs));
        check_size(result.productions.len())?;
    }
    Ok(result)
}

/// Descomposición por la derecha en cadenas binarias deterministas.
pub fn binarize(grammar: &Grammar) -> Result<Grammar> {
    let mut result = grammar.clone();
    for p in &grammar.productions {
        if p.rhs.len() <= 2 {
            continue;
        }
        let mut lhs = p.lhs.clone();
        for s in &p.rhs[..p.rhs.len() - 2] {
            let auxiliary = result.fresh_name(&format!("B_{}", p.lhs));
            result.productions.insert(Production::new(
                &lhs,
                vec![s.clone(), Symbol::Nonterminal(auxiliary.clone())],
            ));
            lhs = auxiliary;
            check_size(result.productions.len())?;
        }
        result
            .productions
            .insert(Production::new(lhs, p.rhs[p.rhs.len() - 2..].to_vec()));
        result.productions.remove(p);
        check_size(result.productions.len())?;
    }
    Ok(result)
}

/// Transformación completa; el validador verifica la postcondición CNF.
pub fn convert(grammar: &Grammar) -> Result<Conversion> {
    let originally_cnf = is_cnf(grammar);
    let epsilon_free = eliminate_epsilon(grammar)?;
    let unit_free = eliminate_unit(&epsilon_free)?;
    let simplified = eliminate_useless(&unit_free);
    let terminals_isolated = isolate_terminals(&simplified)?;
    let cnf = binarize(&terminals_isolated)?;
    validate_cnf(&cnf)?;
    Ok(Conversion {
        originally_cnf,
        epsilon_free,
        unit_free,
        simplified,
        terminals_isolated,
        cnf,
    })
}
