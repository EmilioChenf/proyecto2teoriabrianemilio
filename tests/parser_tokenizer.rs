use proyecto2_cyk::cli::parse_args;
use proyecto2_cyk::grammar::Symbol;
use proyecto2_cyk::parser::{parse_grammar, read_grammar};
use proyecto2_cyk::tokenizer::tokenize;

#[test]
fn reads_professor_file() {
    let g = read_grammar("1.txt").unwrap();
    assert_eq!(g.start, "E");
    assert_eq!(g.productions.len(), 8);
}

#[test]
fn identifies_forward_declared_variables() {
    let g = parse_grammar("S -> NP VP\nVP -> eats\nNP -> she").unwrap();
    assert_eq!(
        g.variables().into_iter().collect::<Vec<_>>(),
        ["NP", "S", "VP"]
    );
    assert!(g.productions.iter().any(|p| p.lhs == "S"
        && p.rhs
            == [
                Symbol::Nonterminal("NP".into()),
                Symbol::Nonterminal("VP".into())
            ]));
}

#[test]
fn identifies_multicharacter_and_punctuation_terminals() {
    let g = read_grammar("1.txt").unwrap();
    assert_eq!(
        g.terminals().into_iter().collect::<Vec<_>>(),
        ["(", ")", "*", "+", "id"]
    );
}

#[test]
fn first_lhs_is_start_even_when_not_alphabetically_first() {
    assert_eq!(parse_grammar("Z -> A\nA -> a").unwrap().start, "Z");
}

#[test]
fn whitespace_comments_bom_and_windows_newlines() {
    let g = parse_grammar("\u{feff}# comentario\r\n\r\n S  ->  A   b | a \r\n A -> a\r\n").unwrap();
    assert_eq!(g.start, "S");
    assert_eq!(g.productions.len(), 3);
}

#[test]
fn repeated_lhs_merges_and_deduplicates() {
    let g = parse_grammar("S -> a | a\nS -> b\nS -> a").unwrap();
    assert_eq!(g.productions.len(), 2);
}

#[test]
fn both_epsilon_spellings_use_empty_rhs() {
    let g = parse_grammar("S -> e | ε").unwrap();
    assert_eq!(g.productions.len(), 1);
    assert!(g.accepts_empty());
    assert!(g.terminals().is_empty());
}

#[test]
fn rejects_empty_grammar() {
    assert!(parse_grammar(" \n# nada")
        .unwrap_err()
        .to_string()
        .contains("vacía"));
}

#[test]
fn rejects_missing_arrow_with_line_number() {
    assert!(parse_grammar("S -> a\nA a")
        .unwrap_err()
        .to_string()
        .contains("Línea 2"));
}

#[test]
fn rejects_multiple_arrows() {
    assert!(parse_grammar("S -> A -> a").is_err());
}

#[test]
fn rejects_invalid_initial_or_lhs() {
    for source in [" -> a", "S A -> a", "1S -> a", "e -> a", "ε -> a", "+ -> a"] {
        assert!(parse_grammar(source).is_err(), "{source}");
    }
}

#[test]
fn rejects_undefined_uppercase_variable() {
    assert!(parse_grammar("S -> A")
        .unwrap_err()
        .to_string()
        .contains("sin producción"));
}

#[test]
fn rejects_empty_alternatives() {
    for source in ["S ->", "S -> | a", "S -> a |", "S -> a || b"] {
        assert!(parse_grammar(source).is_err(), "{source}");
    }
}

#[test]
fn rejects_epsilon_mixed_with_symbols() {
    for source in ["S -> a e", "S -> ε a"] {
        assert!(parse_grammar(source).is_err());
    }
}

#[test]
fn missing_file_is_recoverable() {
    assert!(read_grammar("examples/no-existe-9328.txt")
        .unwrap_err()
        .to_string()
        .contains("No se pudo leer"));
}

#[test]
fn display_preserves_start_and_roundtrips() {
    let g = parse_grammar("Z -> A | e\nA -> a").unwrap();
    assert_eq!(parse_grammar(&g.to_string()).unwrap(), g);
}

#[test]
fn lexer_handles_attached_parentheses_and_operators() {
    let g = read_grammar("1.txt").unwrap();
    assert_eq!(
        tokenize("(id*id)+id", &g).unwrap(),
        ["(", "id", "*", "id", ")", "+", "id"]
    );
}

#[test]
fn lexer_splits_english_words_and_unicode_spaces() {
    let g = read_grammar("examples/english.txt").unwrap();
    assert_eq!(
        tokenize("she\t eats\u{2003}a cake", &g).unwrap(),
        ["she", "eats", "a", "cake"]
    );
}

#[test]
fn lexer_selects_longest_terminal() {
    let g = parse_grammar("S -> == | = | id").unwrap();
    assert_eq!(tokenize("id==id", &g).unwrap(), ["id", "==", "id"]);
}

#[test]
fn lexer_does_not_split_unknown_word_into_known_prefixes() {
    let g = parse_grammar("S -> a | ab | b").unwrap();
    assert_eq!(tokenize("ab", &g).unwrap(), ["ab"]);
    assert!(tokenize("aba", &g).is_err());
}

#[test]
fn lexer_reports_unknown_symbol_and_character_position() {
    let g = parse_grammar("S -> á").unwrap();
    let error = tokenize("á @", &g).unwrap_err().to_string();
    assert!(error.contains("carácter 3"));
}

#[test]
fn lexer_empty_input_is_empty_token_sequence() {
    assert!(tokenize(" \n", &parse_grammar("S -> e").unwrap())
        .unwrap()
        .is_empty());
}

#[test]
fn lexer_is_case_sensitive() {
    assert!(tokenize("She", &read_grammar("examples/english.txt").unwrap()).is_err());
}

#[test]
fn cli_argument_modes_and_end_of_options() {
    let opts = parse_args(["--verbose", "1.txt", "id", "+", "id"].map(String::from)).unwrap();
    assert!(opts.verbose);
    assert_eq!(opts.input.as_deref(), Some("id + id"));
    let opts = parse_args(["--", "1.txt", "-"].map(String::from)).unwrap();
    assert_eq!(opts.input.as_deref(), Some("-"));
    assert!(parse_args(["--desconocida".into()]).is_err());
}

#[test]
fn cli_no_arguments_uses_english_interactively() {
    let opts = parse_args(Vec::new()).unwrap();
    assert_eq!(opts.file.to_str(), Some("examples/english.txt"));
    assert!(opts.input.is_none());
}
