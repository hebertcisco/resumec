use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::constants::BUILTIN_THEMES;
use crate::error::AppError;
use crate::model::Theme;
use crate::paths::AppPaths;
use crate::util::looks_like_url;

pub fn builtin_theme_sources() -> &'static [(&'static str, &'static str)] {
    BUILTIN_THEMES
}

pub fn builtin_theme_source(name: &str) -> Option<&'static str> {
    BUILTIN_THEMES
        .iter()
        .find_map(|(theme_name, source)| {
            if *theme_name == name {
                Some(source)
            } else {
                None
            }
        })
        .copied()
}

pub fn load_theme_by_name(name: &str, paths: &AppPaths) -> Result<Theme, AppError> {
    let theme_path = paths.themes_dir.join(name).join("theme.yml");
    if theme_path.exists() {
        return load_theme_from_path(&theme_path);
    }
    if let Some(source) = builtin_theme_source(name) {
        return parse_theme_content(name, source);
    }
    Err(AppError::ThemeNotFound {
        name: name.to_string(),
    })
}

pub fn load_theme_from_path(path: &Path) -> Result<Theme, AppError> {
    let content = fs::read_to_string(path).map_err(|error| AppError::FileRead {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    parse_theme_content(
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("theme"),
        &content,
    )
}

pub fn parse_theme_content(name_hint: &str, content: &str) -> Result<Theme, AppError> {
    let theme: Theme = serde_yaml::from_str(content).map_err(|error| AppError::InvalidTheme {
        name: name_hint.to_string(),
        message: error.to_string(),
    })?;
    let issues = theme.validate();
    if issues.is_empty() {
        Ok(theme)
    } else {
        Err(AppError::InvalidTheme {
            name: name_hint.to_string(),
            message: issues
                .iter()
                .map(|issue| format!("{}: {}", issue.field, issue.message))
                .collect::<Vec<_>>()
                .join("; "),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ThemeDescriptor {
    pub name: String,
    pub source: String,
}

pub fn list_themes(paths: &AppPaths) -> Result<Vec<ThemeDescriptor>, AppError> {
    paths.ensure_default_themes()?;
    let mut themes: Vec<ThemeDescriptor> = builtin_theme_sources()
        .iter()
        .map(|(name, _)| ThemeDescriptor {
            name: (*name).to_string(),
            source: "built-in".to_string(),
        })
        .collect();

    if paths.themes_dir.exists() {
        for entry in fs::read_dir(&paths.themes_dir).map_err(|error| AppError::FileRead {
            path: paths.themes_dir.display().to_string(),
            message: error.to_string(),
        })? {
            let entry = entry.map_err(|error| AppError::FileRead {
                path: paths.themes_dir.display().to_string(),
                message: error.to_string(),
            })?;
            let theme_path = entry.path().join("theme.yml");
            if theme_path.exists() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !themes.iter().any(|theme| theme.name == name) {
                    themes.push(ThemeDescriptor {
                        name,
                        source: theme_path.display().to_string(),
                    });
                }
            }
        }
    }
    themes.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(themes)
}

pub fn install_theme(paths: &AppPaths, source: &str) -> Result<Theme, AppError> {
    paths.ensure_default_themes()?;
    let content = if looks_like_url(source) {
        reqwest::blocking::get(source)
            .map_err(|error| AppError::Network(error.to_string()))?
            .text()
            .map_err(|error| AppError::Network(error.to_string()))?
    } else {
        let source_path = PathBuf::from(source);
        let theme_path = if source_path.is_dir() {
            source_path.join("theme.yml")
        } else {
            source_path
        };
        fs::read_to_string(&theme_path).map_err(|error| AppError::FileRead {
            path: theme_path.display().to_string(),
            message: error.to_string(),
        })?
    };
    let theme = parse_theme_content(source, &content)?;
    let destination_dir = paths.themes_dir.join(&theme.name);
    fs::create_dir_all(&destination_dir).map_err(|error| AppError::FileWrite {
        path: destination_dir.display().to_string(),
        message: error.to_string(),
    })?;
    let destination = destination_dir.join("theme.yml");
    fs::write(&destination, content).map_err(|error| AppError::FileWrite {
        path: destination.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(theme)
}

pub fn default_theme_template(name: &str) -> String {
    format!(
        "schema_version: 1\nname: {name}\ndescription: Custom resumec theme\nats_safe: true\npalette:\n  primary: \"#1F2933\"\n  secondary: \"#334E68\"\n  accent: \"#486581\"\n  text: \"#102A43\"\n  muted: \"#627D98\"\ntypography:\n  font_family: Helvetica\n  base_size_pt: 11.0\n  heading_size_pt: 14.0\nspacing:\n  section_gap_pt: 12.0\n  item_gap_pt: 6.0\nlayout:\n  columns: 1\nsections:\n  - id: basics\n    visible: true\n  - id: summary\n    visible: true\n  - id: work\n    visible: true\n  - id: education\n    visible: true\n  - id: skills\n    visible: true\n"
    )
}
