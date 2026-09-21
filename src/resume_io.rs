use std::fs;
use std::path::Path;

use crate::error::AppError;
use crate::model::Resume;

type ResumeParser = fn(&str) -> Result<Resume, String>;

pub fn load_resume(path: &Path) -> Result<Resume, AppError> {
    let content = fs::read_to_string(path).map_err(|error| AppError::FileRead {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    parse_resume_content(path, &content)
}

pub fn parse_resume_content(path: &Path, content: &str) -> Result<Resume, AppError> {
    let path_display = path.display().to_string();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    let mut attempts = Vec::new();

    let parsers: Vec<ResumeParser> = match extension {
        "json" => vec![parse_json_resume, parse_yaml_resume, parse_toml_resume],
        "yaml" | "yml" => vec![parse_yaml_resume, parse_json_resume, parse_toml_resume],
        "toml" => vec![parse_toml_resume, parse_json_resume, parse_yaml_resume],
        _ => vec![parse_json_resume, parse_yaml_resume, parse_toml_resume],
    };

    for parser in parsers {
        match parser(content) {
            Ok(resume) => return Ok(resume),
            Err(message) => attempts.push(message),
        }
    }

    Err(AppError::Parse {
        path: path_display,
        message: attempts.join(" | "),
    })
}

fn parse_json_resume(content: &str) -> Result<Resume, String> {
    serde_json::from_str(content).map_err(|error| error.to_string())
}

fn parse_yaml_resume(content: &str) -> Result<Resume, String> {
    serde_yaml::from_str(content).map_err(|error| error.to_string())
}

fn parse_toml_resume(content: &str) -> Result<Resume, String> {
    toml::from_str(content).map_err(|error| error.to_string())
}
