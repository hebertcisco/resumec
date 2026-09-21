use serde::{Deserialize, Serialize};

use crate::constants::THEME_SCHEMA_VERSION;
use crate::error::ValidationIssue;
use crate::util::{default_page_margin_pt, default_true, is_hex_color, is_safe_file_stem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub schema_version: u32,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub ats_safe: bool,
    pub palette: ThemePalette,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub layout: ThemeLayout,
    #[serde(default)]
    pub sections: Vec<ThemeSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePalette {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub text: String,
    pub muted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub font_family: String,
    pub base_size_pt: f32,
    pub heading_size_pt: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub section_gap_pt: f32,
    pub item_gap_pt: f32,
    #[serde(default = "default_page_margin_pt")]
    pub page_margin_pt: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeLayout {
    pub columns: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSection {
    pub id: SectionId,
    #[serde(default = "default_true")]
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SectionId {
    Basics,
    Summary,
    Work,
    Education,
    Skills,
    Certifications,
    Projects,
    Languages,
    Awards,
}

impl Theme {
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        if self.schema_version != THEME_SCHEMA_VERSION {
            issues.push(ValidationIssue::new(
                "schema_version",
                format!("expected schema_version {THEME_SCHEMA_VERSION}"),
            ));
        }
        if !is_safe_file_stem(&self.name) {
            issues.push(ValidationIssue::new(
                "name",
                "theme name must contain only ASCII letters, numbers, '-' or '_'",
            ));
        }
        if !(1..=2).contains(&self.layout.columns) {
            issues.push(ValidationIssue::new(
                "layout.columns",
                "theme layout columns must be 1 or 2",
            ));
        }
        for (label, color) in [
            ("palette.primary", &self.palette.primary),
            ("palette.secondary", &self.palette.secondary),
            ("palette.accent", &self.palette.accent),
            ("palette.text", &self.palette.text),
            ("palette.muted", &self.palette.muted),
        ] {
            if !is_hex_color(color) {
                issues.push(ValidationIssue::new(
                    label,
                    "color must be in #RRGGBB format",
                ));
            }
        }
        if self.typography.base_size_pt < 8.0 {
            issues.push(ValidationIssue::new(
                "typography.base_size_pt",
                "base font size must be at least 8pt",
            ));
        }
        if self.typography.heading_size_pt < self.typography.base_size_pt {
            issues.push(ValidationIssue::new(
                "typography.heading_size_pt",
                "heading size must be greater than or equal to base size",
            ));
        }
        issues
    }

    pub(crate) fn visible_sections(&self) -> Vec<SectionId> {
        let mut sections: Vec<SectionId> = self
            .sections
            .iter()
            .filter(|section| section.visible)
            .map(|section| section.id)
            .collect();
        if sections.is_empty() {
            sections = vec![
                SectionId::Basics,
                SectionId::Summary,
                SectionId::Work,
                SectionId::Education,
                SectionId::Skills,
                SectionId::Projects,
                SectionId::Certifications,
                SectionId::Languages,
                SectionId::Awards,
            ];
        }
        sections
    }
}
