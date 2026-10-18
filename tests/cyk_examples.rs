use proyecto2_cyk::cnf::{convert, is_cnf};
use proyecto2_cyk::cyk::{recognize, BackPointer, CykResult, MAX_TOKENS};
use proyecto2_cyk::grammar::{Grammar, Production, Symbol};
use proyecto2_cyk::parser::{parse_grammar, read_grammar};
use proyecto2_cyk::tokenizer::tokenize;
use proyecto2_cyk::tree::ParseTreeNode;

fn analyze(source: &str, input: &str) -> (Grammar, CykResult) {
    let original = parse_grammar(source).unwrap();
    let tokens = tokenize(input, &original).unwrap();
    let cnf = convert(&original).unwrap().cnf;
    let result = recognize(&cnf, &tokens).unwrap();
    (cnf, result)
}

fn check_tree(node: &ParseTreeNode, grammar: &Grammar) {
    if let ParseTreeNode::Variable { name, children } = node {
        let rhs = children
            .iter()
            .filter_map(|child| match child {
                ParseTreeNode::Variable { name, .. } => Some(Symbol::Nonterminal(name.clone())),
                ParseTreeNode::Terminal(t) => Some(Symbol::Terminal(t.clone())),
                ParseTreeNode::Epsilon => None,
            })
            .collect();
        assert!(
            grammar.productions.contains(&Production::new(name, rhs)),
            "nodo sin producción válida: {node}"
        );
        for child in children {
            check_tree(child, grammar);
        }
    }
}

fn check_professor_cases(file: &str, expected_cnf: bool) {
    let original = read_grammar(file).unwrap();
    assert_eq!(is_cnf(&original), expected_cnf);
    let conversion = convert(&original).unwrap();
    assert_eq!(conversion.originally_cnf, expected_cnf);
    assert!(is_cnf(&conversion.cnf));
    for input in [
        "id",
        "id + id",
        "id * id",
        "id * id + id",
        "(id)",
        "(id + id)",
        "(id * id) + id",
        "((id+id)*id)",
    ] {
        let tokens = tokenize(input, &original).unwrap();
        let result = recognize(&conversion.cnf, &tokens).unwrap();
        assert!(result.accepted, "{file}: {input}");
        let tree = result.parse_tree().unwrap().unwrap();
        assert_eq!(tree.yield_tokens(), tokens);
        check_tree(&tree, &conversion.cnf);
    }
    for input in [
        "+ id",
        "id +",
        "id * + id",
        "( id",
        "id id",
        ")id(",
        "",
        "()",
    ] {
        let tokens = tokenize(input, &original).unwrap();
        let result = recognize(&conversion.cnf, &tokens).unwrap();
        assert!(!result.accepted, "{file}: {input}");
        assert!(result.parse_tree().unwrap().is_none());
    }
}

#[test]
fn professor_cfg_accepts_and_rejects_all_required_cases() {
    check_professor_cases("1.txt", false);
}

#[test]
fn professor_cnf_accepts_and_rejects_all_required_cases() {
    check_professor_cases("1-cnf.txt", true);
}

#[test]
fn english_required_sentences_are_accepted_with_valid_trees() {
    let source = std::fs::read_to_string("examples/english.txt").unwrap();
    for input in ["she eats a cake with a fork", "the cat drinks the beer"] {
        let (cnf, result) = analyze(&source, input);
        assert!(result.accepted, "{input}");
        let tree = result.parse_tree().unwrap().unwrap();
        assert_eq!(
            tree.yield_tokens(),
            input.split_whitespace().collect::<Vec<_>>()
        );
        check_tree(&tree, &cnf);
    }
}

#[test]
fn english_rejects_known_words_in_wrong_order() {
    let source = std::fs::read_to_string("examples/english.txt").unwrap();
    for input in [
        "she a eats cake",
        "the cat the beer drinks",
        "she eats a",
        "a cake with a fork",
        "eats she",
        "",
    ] {
        assert!(!analyze(&source, input).1.accepted, "{input}");
    }
}

#[test]
fn english_handles_intransitive_and_recursive_prepositional_phrases() {
    let source = std::fs::read_to_string("examples/english.txt").unwrap();
    for input in [
        "he cooks",
        "she eats with a spoon",
        "she cuts the meat with a knife in the oven",
    ] {
        assert!(analyze(&source, input).1.accepted, "{input}");
    }
}

#[test]
fn english_tests_syntax_without_imposing_semantics() {
    let source = std::fs::read_to_string("examples/english.txt").unwrap();
    assert!(analyze(&source, "the beer cuts a cat").1.accepted);
}

#[test]
fn cyk_initializes_diagonal_and_full_span() {
    let (_, result) = analyze("S -> A B\nA -> a\nB -> b", "a b");
    assert!(result.accepted);
    assert!(matches!(result.table[0][0].get("A"), Some(BackPointer::Terminal(t)) if t == "a"));
    assert!(
        matches!(result.table[0][1].get("S"), Some(BackPointer::Binary { split: 1, left, right }) if left == "A" && right == "B")
    );
}

#[test]
fn cyk_explores_left_and_right_splits() {
    for source in [
        "S -> A B\nA -> a\nB -> C D\nC -> b\nD -> c",
        "S -> A B\nA -> C D\nC -> a\nD -> b\nB -> c",
    ] {
        assert!(analyze(source, "a b c").1.accepted);
    }
}

#[test]
fn cyk_requires_start_symbol_not_any_full_span_variable() {
    let grammar = parse_grammar("S -> a\nA -> b").unwrap();
    assert!(!recognize(&grammar, &["b".into()]).unwrap().accepted);
}

#[test]
fn cyk_handles_unknown_terminal_in_token_api_by_rejecting() {
    let grammar = parse_grammar("S -> a").unwrap();
    assert!(!recognize(&grammar, &["z".into()]).unwrap().accepted);
}

#[test]
fn cyk_refuses_non_cnf_input() {
    assert!(recognize(
        &parse_grammar("S -> a b").unwrap(),
        &["a".into(), "b".into()]
    )
    .is_err());
}

#[test]
fn cyk_checks_resource_limit_before_table_allocation() {
    let grammar = parse_grammar("S -> a").unwrap();
    assert!(recognize(&grammar, &vec!["a".into(); MAX_TOKENS + 1]).is_err());
}

#[test]
fn cyk_empty_acceptance_and_epsilon_tree() {
    let (cnf, result) = analyze("S -> e", "");
    assert!(result.accepted);
    assert!(result.table.is_empty());
    let tree = result.parse_tree().unwrap().unwrap();
    assert!(tree.yield_tokens().is_empty());
    assert!(tree.to_string().contains('ε'));
    check_tree(&tree, &cnf);
}

#[test]
fn cyk_empty_rejection_is_normal_language_decision() {
    let result = analyze("S -> a", "").1;
    assert!(!result.accepted);
    assert!(result.parse_tree().unwrap().is_none());
}

#[test]
fn cyk_empty_language_rejects_all_inputs() {
    let cnf = convert(&parse_grammar("S -> A\nA -> S").unwrap())
        .unwrap()
        .cnf;
    assert!(!recognize(&cnf, &[]).unwrap().accepted);
    assert!(!recognize(&cnf, &["a".into()]).unwrap().accepted);
}

#[test]
fn nullable_recursive_grammar_preserves_empty_and_nonempty_strings() {
    for size in 0..=20 {
        let input = vec!["a"; size].join(" ");
        let (cnf, result) = analyze("S -> A S | e\nA -> a", &input);
        assert!(result.accepted);
        check_tree(&result.parse_tree().unwrap().unwrap(), &cnf);
    }
}

#[test]
fn ambiguous_grammar_selects_one_deterministic_valid_tree() {
    let (cnf, first) = analyze("S -> S S | a", "a a a a a a");
    let second = recognize(&cnf, &first.tokens).unwrap();
    assert_eq!(first.parse_tree().unwrap(), second.parse_tree().unwrap());
    let tree = first.parse_tree().unwrap().unwrap();
    assert_eq!(tree.yield_tokens(), first.tokens);
    check_tree(&tree, &cnf);
}

#[test]
fn tree_render_has_branch_connectors_and_terminal_leaves() {
    let result = analyze("S -> A B\nA -> a\nB -> b", "a b").1;
    assert_eq!(
        result.parse_tree().unwrap().unwrap().to_string(),
        "S\n├── A\n│   └── a\n└── B\n    └── b\n"
    );
}

#[test]
fn corrupted_backpointer_is_recoverable() {
    let mut result = analyze("S -> A B\nA -> a\nB -> b", "a b").1;
    result.table[0][1].insert(
        "S".into(),
        BackPointer::Binary {
            split: 0,
            left: "A".into(),
            right: "B".into(),
        },
    );
    assert!(result.parse_tree().is_err());
    result.table.clear();
    assert!(result.parse_tree().is_err());
}

#[test]
fn useless_terminal_is_lexed_then_rejected() {
    let original = read_grammar("examples/useless.txt").unwrap();
    let tokens = tokenize("c", &original).unwrap();
    let cnf = convert(&original).unwrap().cnf;
    assert!(!recognize(&cnf, &tokens).unwrap().accepted);
}
