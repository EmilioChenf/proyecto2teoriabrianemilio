use proyecto2_cyk::cnf::{binarize, convert, is_cnf, isolate_terminals, validate_cnf};
use proyecto2_cyk::grammar::{Grammar, Production, Symbol};
use proyecto2_cyk::parser::parse_grammar;
use proyecto2_cyk::simplification::{
    eliminate_epsilon, eliminate_unit, eliminate_useless, generating_variables, nullable_variables,
    reachable_variables, simplify,
};
use std::collections::BTreeSet;

fn g(source: &str) -> Grammar {
    parse_grammar(source).unwrap()
}
fn n(name: &str) -> Symbol {
    Symbol::Nonterminal(name.into())
}
fn t(name: &str) -> Symbol {
    Symbol::Terminal(name.into())
}
fn has(grammar: &Grammar, lhs: &str, rhs: Vec<Symbol>) -> bool {
    grammar.productions.contains(&Production::new(lhs, rhs))
}

#[test]
fn nullable_fixed_point_follows_indirect_rules() {
    let grammar = g("S -> A B\nA -> C\nB -> e\nC -> e\nD -> d");
    assert_eq!(
        nullable_variables(&grammar),
        ["A", "B", "C", "S"].map(String::from).into()
    );
}

#[test]
fn nullable_cycle_without_epsilon_stays_nonnullable() {
    assert!(nullable_variables(&g("S -> A\nA -> S")).is_empty());
}

#[test]
fn nullable_rejects_rhs_with_terminal() {
    assert_eq!(
        nullable_variables(&g("S -> A a\nA -> e")),
        BTreeSet::from(["A".into()])
    );
}

#[test]
fn epsilon_generates_all_distinct_omissions() {
    let out = eliminate_epsilon(&g("S -> A B c\nA -> a | e\nB -> b | e")).unwrap();
    for rhs in [
        vec![n("A"), n("B"), t("c")],
        vec![n("A"), t("c")],
        vec![n("B"), t("c")],
        vec![t("c")],
    ] {
        assert!(has(&out, "S", rhs));
    }
    assert!(out.productions.iter().all(|p| !p.rhs.is_empty()));
}

#[test]
fn epsilon_repeated_occurrences_are_deduplicated() {
    let out = eliminate_epsilon(&g("S -> A A A\nA -> a | e")).unwrap();
    assert_eq!(out.productions.iter().filter(|p| p.lhs == "S").count(), 4);
    assert!(out.accepts_empty());
}

#[test]
fn epsilon_retains_only_start_empty_rule() {
    let out = eliminate_epsilon(&g("S -> A\nA -> e | a")).unwrap();
    assert!(out.accepts_empty());
    assert!(out
        .productions
        .iter()
        .filter(|p| p.rhs.is_empty())
        .all(|p| p.lhs == out.start));
}

#[test]
fn nullable_recursive_start_gets_isolated() {
    let out = eliminate_epsilon(&g("S -> S S | a | e")).unwrap();
    assert_ne!(out.start, "S");
    assert!(!out.start_on_rhs());
    assert!(out.accepts_empty());
    assert!(has(&out, "S", vec![n("S"), n("S")]));
}

#[test]
fn new_start_avoids_existing_variable_and_terminal_names() {
    let out = eliminate_epsilon(&g(
        "S -> S A | e | S_INICIO_1\nS_INICIO -> b\nS_INICIO_1 -> c\nA -> a",
    ))
    .unwrap();
    assert_eq!(out.start, "S_INICIO_2");
}

#[test]
fn epsilon_expansion_limit_returns_error() {
    let mut source = String::from("S ->");
    for i in 0..17 {
        source.push_str(&format!(" A{i}"));
    }
    for i in 0..17 {
        source.push_str(&format!("\nA{i} -> e | a"));
    }
    assert!(eliminate_epsilon(&g(&source))
        .unwrap_err()
        .to_string()
        .contains("supera"));
}

#[test]
fn repeated_nullable_symbols_do_not_overflow_bit_masks() {
    let source = format!("S -> {}\nA -> a | e", vec!["A"; 80].join(" "));
    assert_eq!(
        eliminate_epsilon(&g(&source)).unwrap().productions.len(),
        82
    );
}

#[test]
fn unit_closure_is_reflexive_transitive_and_cycle_safe() {
    let grammar = g("S -> A\nA -> B\nB -> S | b");
    assert_eq!(
        proyecto2_cyk::simplification::unit_closure(&grammar, "S"),
        ["A", "B", "S"].map(String::from).into()
    );
    let out = eliminate_unit(&grammar).unwrap();
    assert!(out.productions.iter().all(|p| !p.is_unit()));
    for variable in ["S", "A", "B"] {
        assert!(has(&out, variable, vec![t("b")]));
    }
}

#[test]
fn unit_closure_preserves_multiple_nonunit_rules() {
    let out = eliminate_unit(&g("S -> A | B\nA -> a\nB -> b")).unwrap();
    assert!(has(&out, "S", vec![t("a")]));
    assert!(has(&out, "S", vec![t("b")]));
}

#[test]
fn unit_cycle_without_base_rule_becomes_empty_language() {
    assert!(simplify(&g("S -> A\nA -> S"))
        .unwrap()
        .productions
        .is_empty());
}

#[test]
fn generating_fixed_point_handles_epsilon_and_cycles() {
    let grammar = g("S -> A B\nA -> a\nB -> e\nC -> C C");
    assert_eq!(
        generating_variables(&grammar),
        ["A", "B", "S"].map(String::from).into()
    );
}

#[test]
fn reachable_starts_from_actual_start() {
    assert_eq!(
        reachable_variables(&g("Z -> A\nA -> a\nB -> b")),
        ["A", "Z"].map(String::from).into()
    );
}

#[test]
fn useless_removes_both_nongenerating_and_unreachable_rules() {
    let out = eliminate_useless(&g("S -> A | B\nA -> a\nB -> B B\nC -> c"));
    assert_eq!(out.variables(), ["A", "S"].map(String::from).into());
}

#[test]
fn reachability_is_recomputed_after_nongenerators_removed() {
    let out = eliminate_useless(&g("S -> a | A B\nA -> a\nB -> B"));
    assert_eq!(
        out.productions,
        BTreeSet::from([Production::new("S", vec![t("a")])])
    );
}

#[test]
fn full_simplification_removes_epsilon_units_and_useless_symbols() {
    let out = simplify(&g("S -> A B | C\nA -> a | e\nB -> b\nC -> C\nD -> d")).unwrap();
    assert!(has(&out, "S", vec![t("b")]));
    assert!(!out.variables().contains("C"));
    assert!(!out.variables().contains("D"));
    assert!(out
        .productions
        .iter()
        .all(|p| !p.is_unit() && !p.rhs.is_empty()));
}

#[test]
fn terminal_isolation_preserves_binary_rules_and_single_terminals() {
    let grammar = g("S -> A B | a B | a\nA -> a\nB -> b");
    let out = isolate_terminals(&grammar).unwrap();
    assert!(has(&out, "S", vec![n("A"), n("B")]));
    assert!(has(&out, "S", vec![t("a")]));
    assert!(out
        .productions
        .iter()
        .filter(|p| p.rhs.len() > 1)
        .all(|p| p.rhs.iter().all(|s| matches!(s, Symbol::Nonterminal(_)))));
}

#[test]
fn terminal_auxiliaries_are_shared_and_avoid_collisions() {
    let out = isolate_terminals(&g("S -> a A | a T_a\nA -> b\nT_a -> c")).unwrap();
    assert!(has(&out, "T_a_1", vec![t("a")]));
    assert!(has(&out, "T_a", vec![t("c")]));
    assert_eq!(
        out.productions.iter().filter(|p| p.rhs == [t("a")]).count(),
        1
    );
}

#[test]
fn binarization_creates_equivalent_right_chain() {
    let out = binarize(&g("S -> A B C D\nA -> a\nB -> b\nC -> c\nD -> d")).unwrap();
    assert!(has(&out, "S", vec![n("A"), n("B_S")]));
    assert!(has(&out, "B_S", vec![n("B"), n("B_S_1")]));
    assert!(has(&out, "B_S_1", vec![n("C"), n("D")]));
    assert!(is_cnf(&out));
}

#[test]
fn binarization_avoids_existing_auxiliary_names() {
    let out = binarize(&g("S -> A B B_S\nA -> a\nB -> b\nB_S -> c")).unwrap();
    assert!(has(&out, "B_S", vec![t("c")]));
    assert!(has(&out, "S", vec![n("A"), n("B_S_1")]));
}

#[test]
fn validator_rejects_unit_long_mixed_and_nonstart_epsilon_rules() {
    for source in [
        "S -> A\nA -> a",
        "S -> a b",
        "S -> A A A\nA -> a",
        "S -> A A\nA -> e",
    ] {
        assert!(validate_cnf(&g(source)).is_err(), "{source}");
    }
}

#[test]
fn validator_checks_start_epsilon_rhs_restriction() {
    assert!(is_cnf(&g("S -> A A | e\nA -> a")));
    assert!(!is_cnf(&g("S -> S S | e")));
    assert!(is_cnf(&g("S -> S S | a")));
}

#[test]
fn validator_rejects_empty_symbol_names_in_public_api() {
    let mut grammar = g("S -> a");
    grammar.start.clear();
    assert!(validate_cnf(&grammar).is_err());
    grammar.start = "S".into();
    grammar
        .productions
        .insert(Production::new("S", vec![t("")]));
    assert!(validate_cnf(&grammar).is_err());
}

#[test]
fn full_conversion_is_deterministic_idempotent_and_really_cnf() {
    let grammar = g("S -> a A b B | e\nA -> A A | a | e\nB -> A | b");
    let first = convert(&grammar).unwrap();
    let second = convert(&grammar).unwrap();
    assert!(is_cnf(&first.cnf));
    assert_eq!(first.cnf, second.cnf);
    assert_eq!(convert(&first.cnf).unwrap().cnf, first.cnf);
}

#[test]
fn conversion_handles_epsilon_only_and_empty_languages() {
    assert!(convert(&g("S -> e")).unwrap().cnf.accepts_empty());
    let out = convert(&g("S -> A\nA -> S")).unwrap().cnf;
    assert!(out.productions.is_empty());
    assert!(is_cnf(&out));
}

#[test]
fn cnf_serialization_can_be_read_and_preserves_start() {
    let out = convert(&g("Z -> Z A | e\nA -> a")).unwrap().cnf;
    assert_eq!(parse_grammar(&out.to_string()).unwrap(), out);
}
