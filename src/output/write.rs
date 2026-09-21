use std::io::{self, IsTerminal, Write};
use std::path::Path;

use crate::build::BuildOptions;
use crate::error::AppError;

pub fn ensure_can_write(path: &Path, options: &BuildOptions) -> Result<(), AppError> {
    if path.exists() && !options.overwrite {
        if options.json_output
            || options.non_interactive
            || !io::stdin().is_terminal()
            || !io::stdout().is_terminal()
        {
            return Err(AppError::OverwriteRequired {
                path: path.display().to_string(),
            });
        }
        print!("{} exists. Overwrite? [y/N]: ", path.display());
        io::stdout().flush().map_err(|error| AppError::FileWrite {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|error| AppError::FileRead {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            return Err(AppError::OverwriteRequired {
                path: path.display().to_string(),
            });
        }
    }
    Ok(())
}
