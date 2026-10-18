use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cyk"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn accepted_cli_reports_yes_timing_and_tree() {
    let output = run(&["1.txt", "id * id + id"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Resultado CYK: SÍ",
        "Tiempo CYK:",
        "Parse Tree",
        "CNF (validada)",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn rejected_cli_reports_no_and_exit_one() {
    let output = run(&["1.txt", "+ id"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Resultado CYK: NO"));
}

#[test]
fn invalid_lexeme_is_an_error_not_silent_rejection() {
    let output = run(&["1.txt", "id @ id"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("tokenización"));
}

#[test]
fn missing_file_has_readable_error_and_exit_two() {
    let output = run(&["no-existe-723891.txt", "id"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("No se pudo leer"));
}

#[test]
fn verbose_reports_intermediate_steps_and_table() {
    let output = run(&["--verbose", "1.txt", "id"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Anulables:",
        "Después de eliminar epsilon",
        "Después de eliminar unitarias",
        "Terminales aislados",
        "Tabla CYK",
    ] {
        assert!(text.contains(expected));
    }
}

#[test]
fn english_capitalized_input_works_with_explicit_case_option() {
    let output = run(&[
        "--ignore-case",
        "examples/english.txt",
        "She eats a cake with a fork",
    ]);
    assert!(output.status.success());
}

#[test]
fn interactive_mode_reads_one_line() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cyk"))
        .arg("1.txt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"(id+id)\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("Ingrese una expresión"));
}

#[test]
fn default_interactive_mode_uses_english_grammar() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cyk"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"the cat drinks the beer\n")
        .unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
}

#[test]
fn empty_interactive_line_accepts_epsilon() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cyk"))
        .arg("examples/epsilon.txt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"\n").unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
}

#[test]
fn eof_is_reported_as_missing_expression() {
    let output = run(&["1.txt"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("fin de entrada"));
}

#[test]
fn help_succeeds_and_unknown_option_fails() {
    assert!(run(&["--help"]).status.success());
    assert_eq!(run(&["--inexistente"]).status.code(), Some(2));
}
