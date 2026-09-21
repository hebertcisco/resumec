use std::fs;

use crate::cli::{InitArgs, InitFormat};
use crate::constants::EXAMPLE_RESUME_YAML;
use crate::error::AppError;
use crate::model::Resume;

pub fn handle_init(args: InitArgs) -> Result<(), AppError> {
    let current_dir =
        std::env::current_dir().map_err(|error| AppError::Config(error.to_string()))?;
    let (file_name, content) = match args.format {
        InitFormat::Yaml => ("resume.yaml", EXAMPLE_RESUME_YAML.to_string()),
        InitFormat::Json => {
            let resume: Resume = serde_yaml::from_str(EXAMPLE_RESUME_YAML)
                .map_err(|error| AppError::Config(error.to_string()))?;
            (
                "resume.json",
                serde_json::to_string_pretty(&resume)
                    .map_err(|error| AppError::Config(error.to_string()))?,
            )
        }
        InitFormat::Toml => {
            let resume: Resume = serde_yaml::from_str(EXAMPLE_RESUME_YAML)
                .map_err(|error| AppError::Config(error.to_string()))?;
            (
                "resume.toml",
                toml::to_string_pretty(&resume)
                    .map_err(|error| AppError::Config(error.to_string()))?,
            )
        }
    };
    let path = current_dir.join(file_name);
    if path.exists() {
        return Err(AppError::OverwriteRequired {
            path: path.display().to_string(),
        });
    }
    fs::write(&path, content).map_err(|error| AppError::FileWrite {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    println!("created {}", path.display());
    Ok(())
}
