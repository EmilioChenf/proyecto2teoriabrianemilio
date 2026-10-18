use crate::error::{Error, Result};
use crate::grammar::Grammar;

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Máxima coincidencia de terminales, con fronteras de palabra y Unicode.
/// Los símbolos de puntuación pueden estar pegados: `(id+id)` es válido.
pub fn tokenize(input: &str, grammar: &Grammar) -> Result<Vec<String>> {
    let mut terminals: Vec<_> = grammar.terminals().into_iter().collect();
    terminals.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    let mut tokens = Vec::new();
    let mut offset = 0;
    while offset < input.len() {
        let rest = &input[offset..];
        let Some(first) = rest.chars().next() else {
            break;
        };
        if first.is_whitespace() {
            offset += first.len_utf8();
            continue;
        }
        let matched = terminals.iter().find(|t| {
            !t.is_empty()
                && rest.starts_with(t.as_str())
                && !(t.chars().last().is_some_and(word_char)
                    && rest[t.len()..].chars().next().is_some_and(word_char))
        });
        let Some(terminal) = matched else {
            let position = input[..offset].chars().count() + 1;
            let fragment: String = rest.chars().take(20).collect();
            return Err(Error(format!("Error de tokenización en carácter {position}: '{fragment}' no comienza con un terminal reconocido")));
        };
        tokens.push(terminal.clone());
        offset += terminal.len();
    }
    Ok(tokens)
}
