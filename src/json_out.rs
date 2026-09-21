use std::io;

use serde::Serialize;

use crate::error::AppError;

pub fn print_json<T: Serialize>(value: &T) -> Result<(), AppError> {
    serde_json::to_writer_pretty(io::stdout(), value)
        .map_err(|error| AppError::Config(error.to_string()))?;
    println!();
    Ok(())
}
