use serde::{Deserialize, Serialize};

use crate::model::results::OutputFormat;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    pub default_theme: Option<String>,
    pub default_format: Option<OutputFormat>,
    pub locale: Option<String>,
    pub output_dir: Option<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_theme: Some("classic".to_string()),
            default_format: Some(OutputFormat::Pdf),
            locale: None,
            output_dir: None,
        }
    }
}
