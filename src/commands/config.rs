use crate::cli::{ConfigArgs, ConfigCommand};
use crate::error::AppError;
use crate::json_out::print_json;
use crate::model::OutputFormat;
use crate::paths::AppPaths;
use crate::preferences_io::{load_preferences, save_preferences};

pub fn handle_config(args: ConfigArgs) -> Result<(), AppError> {
    let paths = AppPaths::detect()?;
    paths.ensure()?;
    let mut preferences = load_preferences(&paths)?;
    match args.command {
        ConfigCommand::Show => {
            print_json(&preferences)?;
        }
        ConfigCommand::Set { key, value } => {
            match key.as_str() {
                "theme" => preferences.default_theme = Some(value),
                "format" => {
                    preferences.default_format = Some(match value.as_str() {
                        "pdf" => OutputFormat::Pdf,
                        "docx" => OutputFormat::Docx,
                        "both" => OutputFormat::Both,
                        _ => {
                            return Err(AppError::Config(
                                "format must be pdf, docx, or both".to_string(),
                            ));
                        }
                    })
                }
                "locale" => preferences.locale = Some(value),
                "output_dir" => preferences.output_dir = Some(value),
                _ => {
                    return Err(AppError::Config(
                        "supported keys: theme, format, locale, output_dir".to_string(),
                    ));
                }
            }
            save_preferences(&paths, &preferences)?;
            print_json(&preferences)?;
        }
    }
    Ok(())
}
