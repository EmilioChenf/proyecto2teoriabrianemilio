use crate::cnf::convert;
use crate::cyk::recognize;
use crate::error::{Error, Result};
use crate::parser::read_grammar;
use crate::simplification::{generating_variables, nullable_variables, reachable_variables};
use crate::tokenizer::tokenize;
use std::io::{self, Write};
use std::path::PathBuf;

const HELP: &str = "Proyecto 2 - Algoritmo CYK
Uso: cyk [--verbose] [--ignore-case] [archivo [\"expresión\"]]

Sin archivo: usa examples/english.txt y solicita una frase.
Sin expresión: solicita una línea por consola. Una línea vacía representa ε.
--verbose      Muestra etapas intermedias, conjuntos y tabla CYK.
--ignore-case  Convierte la entrada a minúsculas (gramática en minúsculas).
--help, -h     Muestra esta ayuda.
--             Termina las opciones, para nombres/expresiones que empiezan por -.

Ejemplos:
  cargo run -- 1.txt \"(id*id)+id\"
  cargo run -- examples/english.txt \"she eats a cake with a fork\"
  cargo run -- --ignore-case examples/english.txt \"The cat drinks the beer\"

Códigos de salida: 0 = SÍ/ayuda, 1 = NO, 2 = error de entrada/archivo/gramática.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub file: PathBuf,
    pub input: Option<String>,
    pub verbose: bool,
    pub ignore_case: bool,
    pub help: bool,
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Options> {
    let mut options = Options {
        file: PathBuf::from("examples/english.txt"),
        input: None,
        verbose: false,
        ignore_case: false,
        help: false,
    };
    let mut positional = Vec::new();
    let mut flags = true;
    for arg in args {
        match arg.as_str() {
            "--" if flags => flags = false,
            "--help" | "-h" if flags => options.help = true,
            "--verbose" if flags => options.verbose = true,
            "--ignore-case" if flags => options.ignore_case = true,
            _ if flags && arg.starts_with('-') => {
                return Err(Error(format!("Opción desconocida '{arg}'; consulte --help o use -- para terminar las opciones")));
            }
            _ => positional.push(arg),
        }
    }
    if let Some(file) = positional.first() {
        options.file = PathBuf::from(file);
    }
    if positional.len() > 1 {
        options.input = Some(positional[1..].join(" "));
    }
    Ok(options)
}

/// Orquesta la demostración; los algoritmos también se pueden usar como biblioteca.
pub fn run(args: impl IntoIterator<Item = String>) -> Result<bool> {
    let options = parse_args(args)?;
    if options.help {
        println!("{HELP}");
        return Ok(true);
    }
    let original = read_grammar(&options.file)?;
    println!("========================================\nPROYECTO 2 - ALGORITMO CYK\n========================================");
    println!(
        "\nArchivo: {}\n\nGramática original:\n{original}",
        options.file.display()
    );
    let conversion = convert(&original)?;
    println!(
        "¿La gramática original está en CNF?: {}",
        if conversion.originally_cnf {
            "SÍ"
        } else {
            "NO"
        }
    );
    println!("\nSimplificando gramática...");
    if options.verbose {
        println!("Anulables: {:?}", nullable_variables(&original));
        println!(
            "Generadoras originales: {:?}",
            generating_variables(&original)
        );
        println!(
            "Alcanzables originales: {:?}",
            reachable_variables(&original)
        );
        println!(
            "\nDespués de eliminar epsilon:\n{}",
            conversion.epsilon_free
        );
        println!("Después de eliminar unitarias:\n{}", conversion.unit_free);
    }
    println!("Gramática simplificada:\n{}", conversion.simplified);
    println!("Convirtiendo a CNF...");
    if options.verbose {
        println!("Terminales aislados:\n{}", conversion.terminals_isolated);
    }
    println!("Gramática en CNF (validada):\n{}", conversion.cnf);
    let input = if let Some(input) = options.input {
        input
    } else {
        print!("Ingrese una expresión (línea vacía = ε): ");
        io::stdout()
            .flush()
            .map_err(|e| Error(format!("No se pudo mostrar el prompt: {e}")))?;
        let mut line = String::new();
        let bytes = io::stdin()
            .read_line(&mut line)
            .map_err(|e| Error(format!("No se pudo leer la expresión: {e}")))?;
        if bytes == 0 {
            return Err(Error("No se recibió expresión (fin de entrada)".into()));
        }
        line.trim_end_matches(['\r', '\n']).to_owned()
    };
    let normalized = if options.ignore_case {
        if original.terminals().iter().any(|t| *t != t.to_lowercase()) {
            return Err(Error(
                "--ignore-case requiere terminales de la gramática en minúsculas".into(),
            ));
        }
        input.to_lowercase()
    } else {
        input.clone()
    };
    // Se usa el vocabulario original: un terminal eliminado por inútil sigue
    // siendo una entrada léxica válida, que CYK debe rechazar sintácticamente.
    let tokens = tokenize(&normalized, &original)?;
    let result = recognize(&conversion.cnf, &tokens)?;
    println!(
        "\nExpresión: {}",
        if input.trim().is_empty() {
            "ε"
        } else {
            &input
        }
    );
    println!(
        "Tokens: {}",
        if tokens.is_empty() {
            "ε".into()
        } else {
            tokens.join(" ")
        }
    );
    println!(
        "Resultado CYK: {}",
        if result.accepted { "SÍ" } else { "NO" }
    );
    println!(
        "Tiempo CYK: {:.3} µs",
        result.elapsed.as_secs_f64() * 1_000_000.0
    );
    if options.verbose {
        println!("\nTabla CYK [inicio, longitud] (índices desde 0):");
        for (start, row) in result.table.iter().enumerate() {
            for (column, cell) in row.iter().enumerate() {
                println!(
                    "[{start}, {}]: {{{}}}",
                    column + 1,
                    cell.keys().cloned().collect::<Vec<_>>().join(", ")
                );
            }
        }
    }
    if let Some(tree) = result.parse_tree()? {
        println!("\nParse Tree (gramática CNF):\n{tree}");
    } else {
        println!("Parse Tree: no existe para esta entrada.");
    }
    if tokens.is_empty() && !result.accepted {
        println!("La gramática no genera la expresión vacía.");
    }
    Ok(result.accepted)
}
