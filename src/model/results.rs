use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::error::ValidationIssue;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Pdf,
    Docx,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedOutput {
    pub format: OutputFormat,
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub input_file: PathBuf,
    pub theme: String,
    pub outputs: Vec<GeneratedOutput>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MachineResult<T>
where
    T: Serialize,
{
    pub status: &'static str,
    pub data: Option<T>,
    pub warnings: Vec<String>,
    pub errors: Vec<ValidationIssue>,
}

impl<T> MachineResult<T>
where
    T: Serialize,
{
    pub fn success(data: T, warnings: Vec<String>) -> Self {
        Self {
            status: "success",
            data: Some(data),
            warnings,
            errors: Vec::new(),
        }
    }

    pub fn error(errors: Vec<ValidationIssue>) -> Self {
        Self {
            status: "error",
            data: None,
            warnings: Vec::new(),
            errors,
        }
    }
}
