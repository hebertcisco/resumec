use std::fs;

use crate::cli::{ThemeArgs, ThemeCommand};
use crate::error::AppError;
use crate::paths::AppPaths;
use crate::theme_io::{default_theme_template, install_theme, list_themes};
use crate::util::is_safe_file_stem;

pub fn handle_theme(args: ThemeArgs) -> Result<(), AppError> {
    let paths = AppPaths::detect()?;
    paths.ensure_default_themes()?;
    match args.command {
        ThemeCommand::List => {
            for theme in list_themes(&paths)? {
                println!("{}\t{}", theme.name, theme.source);
            }
        }
        ThemeCommand::New { name } => {
            if !is_safe_file_stem(&name) {
                return Err(AppError::Config(
                    "theme name must contain only ASCII letters, numbers, '-' or '_'".to_string(),
                ));
            }
            let destination = std::env::current_dir()
                .map_err(|error| AppError::Config(error.to_string()))?
                .join(&name);
            let theme_path = destination.join("theme.yml");
            fs::create_dir_all(&destination).map_err(|error| AppError::FileWrite {
                path: destination.display().to_string(),
                message: error.to_string(),
            })?;
            let content = default_theme_template(&name);
            fs::write(&theme_path, content).map_err(|error| AppError::FileWrite {
                path: theme_path.display().to_string(),
                message: error.to_string(),
            })?;
            println!("created {}", theme_path.display());
        }
        ThemeCommand::Install { source } => {
            let theme = install_theme(&paths, &source)?;
            println!("installed theme {}", theme.name);
        }
    }
    Ok(())
}
