use std::path::PathBuf;

use crate::build::{BuildOptions, build_resume};
use crate::cli::BuildArgs;
use crate::error::AppError;
use crate::json_out::print_json;
use crate::model::{MachineResult, OutputFormat};
use crate::paths::AppPaths;
use crate::preferences_io::load_preferences;
use crate::util::derive_output_name_from_path;

pub fn handle_build(args: BuildArgs) -> Result<(), AppError> {
    let paths = AppPaths::detect()?;
    let preferences = load_preferences(&paths)?;
    let theme_name = args
        .theme
        .or(preferences.default_theme.clone())
        .unwrap_or_default();
    let output_dir = args
        .output_dir
        .or_else(|| preferences.output_dir.clone().map(PathBuf::from))
        .unwrap_or(std::env::current_dir().map_err(|error| AppError::Config(error.to_string()))?);
    let output_name = args
        .output_name
        .unwrap_or_else(|| derive_output_name_from_path(&args.input));
    let options = BuildOptions {
        format: args
            .format
            .or(preferences.default_format)
            .unwrap_or(OutputFormat::Pdf),
        theme_name,
        output_dir,
        output_name,
        overwrite: args.overwrite,
        non_interactive: args.non_interactive,
        json_output: args.json_output,
    };
    let result = build_resume(&args.input, options)?;
    if args.json_output {
        print_json(&MachineResult::success(
            result.clone(),
            result.warnings.clone(),
        ))?;
    } else if !args.quiet {
        for output in &result.outputs {
            println!(
                "generated {:?}: {} ({} bytes)",
                output.format,
                output.path.display(),
                output.bytes
            );
        }
        if args.verbose {
            println!("theme: {}", result.theme);
        }
    }
    Ok(())
}
