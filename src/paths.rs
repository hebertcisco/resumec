use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;

use crate::constants::APP_NAME;
use crate::error::AppError;
use crate::theme_io::builtin_theme_sources;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub preferences_path: PathBuf,
    pub themes_dir: PathBuf,
}

impl AppPaths {
    pub fn detect() -> Result<Self, AppError> {
        if let Some(config_dir) = std::env::var_os("RESUMEC_CONFIG_DIR") {
            return Ok(Self::from_config_dir(config_dir));
        }
        let project_dirs = ProjectDirs::from("", "", APP_NAME)
            .ok_or_else(|| AppError::Config("failed to determine config directory".to_string()))?;
        Ok(Self::from_config_dir(project_dirs.config_dir()))
    }

    pub fn from_config_dir(config_dir: impl Into<PathBuf>) -> Self {
        let config_dir = config_dir.into();
        let themes_dir = config_dir.join("themes");
        let preferences_path = config_dir.join("preferences.json");
        Self {
            config_dir,
            preferences_path,
            themes_dir,
        }
    }

    pub fn ensure(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.themes_dir)
            .map_err(|error| AppError::Config(error.to_string()))?;
        Ok(())
    }

    pub fn ensure_default_themes(&self) -> Result<(), AppError> {
        self.ensure()?;
        for (name, content) in builtin_theme_sources() {
            let dir = self.themes_dir.join(name);
            let path = dir.join("theme.yml");
            if !path.exists() {
                fs::create_dir_all(&dir).map_err(|error| AppError::Config(error.to_string()))?;
                fs::write(&path, content).map_err(|error| AppError::FileWrite {
                    path: path.display().to_string(),
                    message: error.to_string(),
                })?;
            }
        }
        Ok(())
    }
}
