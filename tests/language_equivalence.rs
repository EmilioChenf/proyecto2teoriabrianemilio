//! Oráculo independiente: menor punto fijo de palabras acotadas de cada CFG.
//! No usa CNF ni CYK para obtener el lenguaje esperado.
use proyecto2_cyk::cnf::convert;
use proyecto2_cyk::cyk::recognize;
use proyecto2_cyk::grammar::{Grammar, Symbol};
use proyecto2_cyk::parser::parse_grammar;
use std::collections::{BTreeMap, BTreeSet};

type Words = BTreeSet<Vec<String>>;

fn bounded_language(grammar: &Grammar, bound: usize) -> Words {
    let mut languages: BTreeMap<_, Words> = grammar
        .variables()
        .into_iter()
        .map(|v| (v, Words::new()))
        .collect();
    loop {
        let before = languages.clone();
        for p in &grammar.productions {
            let mut words = Words::from([Vec::new()]);
            for s in &p.rhs {
                let choices = match s {
                    Symbol::Terminal(t) => Words::from([vec![t.clone()]]),
                    Symbol::Nonterminal(v) => before[v].clone(),
                };
                let mut concatenations = Words::new();
                for prefix in &words {
                    for suffix in &choices {
                        if prefix.len() + suffix.len() <= bound {
                            let mut word = prefix.clone();
                            word.extend(suffix.clone());
                            concatenations.insert(word);
                        }
                    }
                }
                words = concatenations;
            }
            languages.entry(p.lhs.clone()).or_default().extend(words);
        }
        if languages == before {
            return languages.remove(&grammar.start).unwrap();
        }
    }
}

fn all_binary_words(bound: usize) -> Words {
    let mut words = Words::from([Vec::new()]);
    let mut level = words.clone();
    for _ in 0..bound {
        let mut next = Words::new();
        for word in level {
            for terminal in ["a", "b"] {
                let mut extended = word.clone();
                extended.push(terminal.into());
                next.insert(extended);
            }
        }
        words.extend(next.clone());
        level = next;
    }
    words
}

fn verify(source: &str, bound: usize) {
    let original = parse_grammar(source).unwrap();
    let cnf = convert(&original).unwrap().cnf;
    let expected = bounded_language(&original, bound);
    assert_eq!(
        expected,
        bounded_language(&cnf, bound),
        "conversión cambió el lenguaje: {source}"
    );
    for tokens in all_binary_words(bound) {
        let result = recognize(&cnf, &tokens).unwrap();
        assert_eq!(
            result.accepted,
            expected.contains(&tokens),
            "CYK discrepó: {tokens:?} en {source}"
        );
        if let Some(tree) = result.parse_tree().unwrap() {
            assert_eq!(tree.yield_tokens(), tokens);
        }
    }
}

#[test]
fn exhaustive_small_languages_preserve_nullable_cycles_and_mixed_rules() {
    for source in [
        "S -> S S | a | b | e",
        "S -> A B\nA -> a A | e\nB -> b B | e",
        "S -> a S b | e",
        "S -> A | a\nA -> B\nB -> S | b | e",
        "S -> A A A\nA -> a | b | e",
        "S -> A B | a\nA -> A\nB -> b",
        "S -> A\nA -> S",
        "S -> a B a A b\nA -> e | a\nB -> b | A",
    ] {
        verify(source, 5);
    }
}

#[test]
fn deterministic_generated_cfgs_agree_with_independent_oracle() {
    let options = [
        "a", "b", "e", "A", "B", "S", "A B", "B A", "a S", "S b", "A A B",
    ];
    let mut state = 2317_u64;
    for _ in 0..24 {
        let mut lines = Vec::new();
        for variable in ["S", "A", "B"] {
            let mut alternatives = Vec::new();
            for _ in 0..3 {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                alternatives.push(options[(state >> 32) as usize % options.len()]);
            }
            lines.push(format!("{variable} -> {}", alternatives.join(" | ")));
        }
        verify(&lines.join("\n"), 4);
    }
}
