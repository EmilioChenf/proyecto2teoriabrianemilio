//! Simplificación por puntos fijos y cierres, preservando el lenguaje.
mod epsilon;
pub use epsilon::{eliminate_epsilon, nullable_variables};

use crate::error::{Error, Result};

/// La eliminación de epsilon puede generar exponencialmente muchas reglas.
pub const MAX_PRODUCTIONS: usize = 100_000;

pub(crate) fn check_size(size: usize) -> Result<()> {
    if size > MAX_PRODUCTIONS {
        Err(Error(format!("La transformación supera {MAX_PRODUCTIONS} producciones/combinaciones; reduzca la gramática")))
    } else {
        Ok(())
    }
}
