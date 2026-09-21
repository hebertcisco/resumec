use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("schema validation failed")]
    Validation { issues: Vec<ValidationIssue> },
    #[error("failed to parse {path}: {message}")]
    Parse { path: String, message: String },
    #[error("theme '{name}' was not found")]
    ThemeNotFound { name: String },
    #[error("failed to write file '{path}': {message}")]
    FileWrite { path: String, message: String },
    #[error("failed to read file '{path}': {message}")]
    FileRead { path: String, message: String },
    #[error("overwrite required for existing file '{path}'")]
    OverwriteRequired { path: String },
    #[error("invalid theme '{name}': {message}")]
    InvalidTheme { name: String, message: String },
    #[error("network error: {0}")]
    Network(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("mcp protocol error: {0}")]
    Mcp(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation { .. } => 1,
            Self::ThemeNotFound { .. } => 2,
            Self::FileWrite { .. } | Self::OverwriteRequired { .. } => 3,
            Self::Parse { .. } | Self::FileRead { .. } => 4,
            Self::InvalidTheme { .. } => 5,
            Self::Network(_) => 6,
            Self::Config(_) => 7,
            Self::Mcp(_) => 8,
        }
    }

    pub fn issues(&self) -> Vec<ValidationIssue> {
        match self {
            Self::Validation { issues } => issues.clone(),
            Self::Parse { path, message } => {
                vec![ValidationIssue::new(path.clone(), message.clone())]
            }
            Self::ThemeNotFound { name } => vec![ValidationIssue::new(
                "theme",
                format!("theme '{name}' was not found"),
            )],
            Self::FileWrite { path, message } => {
                vec![ValidationIssue::new(path.clone(), message.clone())]
            }
            Self::FileRead { path, message } => {
                vec![ValidationIssue::new(path.clone(), message.clone())]
            }
            Self::OverwriteRequired { path } => vec![ValidationIssue::new(
                path.clone(),
                "destination file already exists".to_string(),
            )],
            Self::InvalidTheme { name, message } => {
                vec![ValidationIssue::new(name.clone(), message.clone())]
            }
            Self::Network(message) | Self::Config(message) | Self::Mcp(message) => {
                vec![ValidationIssue::new("general", message.clone())]
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
}

impl ValidationIssue {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}
