use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::model::{BuildResult, OutputFormat, Resume};
use crate::output::{write_docx, write_pdf};
use crate::paths::AppPaths;
use crate::preferences_io::load_preferences;
use crate::render::render_lines;
use crate::resume_io::load_resume;
use crate::theme_io::load_theme_by_name;

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub format: OutputFormat,
    pub theme_name: String,
    pub output_dir: PathBuf,
    pub output_name: String,
    pub overwrite: bool,
    pub non_interactive: bool,
    pub json_output: bool,
}

pub fn validate_resume(resume: &Resume) -> Result<(), AppError> {
    let issues = resume.validate();
    if issues.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation { issues })
    }
}

pub fn build_resume(input_path: &Path, mut options: BuildOptions) -> Result<BuildResult, AppError> {
    let paths = AppPaths::detect()?;
    paths.ensure_default_themes()?;
    let resume = load_resume(input_path)?;
    validate_resume(&resume)?;

    let preferences = load_preferences(&paths)?;
    if options.theme_name.is_empty() {
        options.theme_name = resume
            .theme
            .clone()
            .or(preferences.default_theme)
            .unwrap_or_else(|| "classic".to_string());
    }
    let theme = load_theme_by_name(&options.theme_name, &paths)?;
    let lines = render_lines(&resume, &theme);
    fs::create_dir_all(&options.output_dir).map_err(|error| AppError::FileWrite {
        path: options.output_dir.display().to_string(),
        message: error.to_string(),
    })?;

    let mut outputs = Vec::new();
    match options.format {
        OutputFormat::Pdf => {
            outputs.push(write_pdf(&resume, &theme, &lines, &options)?);
        }
        OutputFormat::Docx => {
            outputs.push(write_docx(&resume, &theme, &lines, &options)?);
        }
        OutputFormat::Both => {
            outputs.push(write_pdf(&resume, &theme, &lines, &options)?);
            outputs.push(write_docx(&resume, &theme, &lines, &options)?);
        }
    }

    Ok(BuildResult {
        input_file: input_path.to_path_buf(),
        theme: theme.name,
        outputs,
        warnings: if theme.layout.columns == 2 {
            vec!["two-column themes may reduce ATS readability".to_string()]
        } else {
            Vec::new()
        },
    })
}
