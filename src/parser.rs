use crate::error::{Error, Result};
use crate::grammar::{Grammar, Production, Symbol};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Lee UTF-8 sin modificar el archivo fuente.
pub fn read_grammar(path: impl AsRef<Path>) -> Result<Grammar> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)
        .map_err(|e| Error(format!("No se pudo leer '{}': {e}", path.display())))?;
    parse_grammar(&text)
}

fn valid_variable(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_' || c == '\'')
        && name != "e"
        && name != "ε"
}

fn line_error(line: usize, message: impl AsRef<str>) -> Error {
    Error(format!("Línea {line}: {}", message.as_ref()))
}

/// Dos pasadas: todas las variables se conocen antes de clasificar el RHS.
/// El primer lado izquierdo es el símbolo inicial; `e`/`ε` son reservados.
pub fn parse_grammar(text: &str) -> Result<Grammar> {
    let mut lines = Vec::new();
    let mut variables = BTreeSet::new();
    for (index, raw) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let number = index + 1;
        let (lhs, rhs) = line
            .split_once("->")
            .ok_or_else(|| line_error(number, "falta el separador ->"))?;
        let lhs = lhs.trim();
        if !valid_variable(lhs) {
            return Err(line_error(
                number,
                "el lado izquierdo debe ser una variable válida (no e/ε)",
            ));
        }
        if rhs.contains("->") {
            return Err(line_error(
                number,
                "solo se permite un separador -> por línea",
            ));
        }
        variables.insert(lhs.to_owned());
        lines.push((number, lhs.to_owned(), rhs));
    }
    let start = lines
        .first()
        .map(|(_, lhs, _)| lhs.clone())
        .ok_or_else(|| Error("Gramática vacía: se requiere al menos una producción".into()))?;
    let mut productions = BTreeSet::new();
    for (number, lhs, rhs) in lines {
        for alternative in rhs.split('|') {
            let tokens: Vec<_> = alternative.split_whitespace().collect();
            if tokens.is_empty() {
                return Err(line_error(number, "alternativa vacía; use e para epsilon"));
            }
            if tokens == ["e"] || tokens == ["ε"] {
                productions.insert(Production::new(&lhs, vec![]));
                continue;
            }
            let mut symbols = Vec::new();
            for token in tokens {
                if token == "e" || token == "ε" {
                    return Err(line_error(
                        number,
                        "epsilon debe aparecer solo en una alternativa",
                    ));
                }
                if variables.contains(token) {
                    symbols.push(Symbol::Nonterminal(token.to_owned()));
                } else {
                    if token.starts_with(char::is_uppercase) {
                        return Err(line_error(
                            number,
                            format!(
                                "variable '{token}' sin producción; defínala en un lado izquierdo"
                            ),
                        ));
                    }
                    if token.chars().any(char::is_control) {
                        return Err(line_error(number, "terminal con carácter de control"));
                    }
                    symbols.push(Symbol::Terminal(token.to_owned()));
                }
            }
            productions.insert(Production::new(&lhs, symbols));
        }
    }
    Ok(Grammar { start, productions })
}
