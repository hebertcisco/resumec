use std::fs;

use crate::error::AppError;
use crate::model::Preferences;
use crate::paths::AppPaths;

pub fn load_preferences(paths: &AppPaths) -> Result<Preferences, AppError> {
    if !paths.preferences_path.exists() {
        return Ok(Preferences::default());
    }
    let content =
        fs::read_to_string(&paths.preferences_path).map_err(|error| AppError::FileRead {
            path: paths.preferences_path.display().to_string(),
            message: error.to_string(),
        })?;
    serde_json::from_str(&content).map_err(|error| AppError::Config(error.to_string()))
}

pub fn save_preferences(paths: &AppPaths, preferences: &Preferences) -> Result<(), AppError> {
    paths.ensure()?;
    let content = serde_json::to_string_pretty(preferences)
        .map_err(|error| AppError::Config(error.to_string()))?;
    fs::write(&paths.preferences_path, content).map_err(|error| AppError::FileWrite {
        path: paths.preferences_path.display().to_string(),
        message: error.to_string(),
    })
}
