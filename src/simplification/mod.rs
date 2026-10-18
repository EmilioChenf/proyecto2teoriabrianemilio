//! Simplificación por puntos fijos y cierres, preservando el lenguaje.
mod epsilon;
pub use epsilon::{eliminate_epsilon, nullable_variables};
mod unit;
pub use unit::{eliminate_unit, unit_closure};
mod useless;
pub use useless::{eliminate_useless, generating_variables, reachable_variables};

use crate::error::{Error, Result};
use crate::grammar::Grammar;

/// Secuencia pública: epsilon, unitarias y finalmente símbolos inútiles.
pub fn simplify(grammar: &Grammar) -> Result<Grammar> {
    let epsilon_free = eliminate_epsilon(grammar)?;
    let unit_free = eliminate_unit(&epsilon_free)?;
    Ok(eliminate_useless(&unit_free))
}

/// La eliminación de epsilon puede generar exponencialmente muchas reglas.
pub const MAX_PRODUCTIONS: usize = 100_000;

pub(crate) fn check_size(size: usize) -> Result<()> {
    if size > MAX_PRODUCTIONS {
        Err(Error(format!("La transformación supera {MAX_PRODUCTIONS} producciones/combinaciones; reduzca la gramática")))
    } else {
        Ok(())
    }
}
