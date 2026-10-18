use std::process::ExitCode;

fn main() -> ExitCode {
    match proyecto2_cyk::cli::run(std::env::args().skip(1)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::from(2)
        }
    }
}
