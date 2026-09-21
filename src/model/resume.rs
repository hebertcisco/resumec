use serde::{Deserialize, Serialize};
use url::Url;

use crate::constants::RESUME_SCHEMA_VERSION;
use crate::error::ValidationIssue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resume {
    pub schema_version: u32,
    pub basics: Basics,
    #[serde(default)]
    pub work: Vec<WorkExperience>,
    #[serde(default)]
    pub education: Vec<EducationEntry>,
    #[serde(default)]
    pub skills: Vec<SkillCategory>,
    #[serde(default)]
    pub certifications: Vec<Certification>,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub languages: Vec<LanguageSkill>,
    #[serde(default)]
    pub awards: Vec<Award>,
    pub theme: Option<String>,
    pub locale: Option<String>,
    pub target_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Basics {
    pub name: String,
    pub headline: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkExperience {
    pub company: String,
    pub position: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub location: Option<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationEntry {
    pub institution: String,
    pub area: String,
    pub study_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub location: Option<String>,
    pub score: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCategory {
    pub category: String,
    #[serde(default)]
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub name: String,
    pub issuer: Option<String>,
    pub date: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub summary: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSkill {
    pub name: String,
    pub fluency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Award {
    pub title: String,
    pub issuer: Option<String>,
    pub date: Option<String>,
    pub summary: Option<String>,
}

impl Resume {
    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        if self.schema_version != RESUME_SCHEMA_VERSION {
            issues.push(ValidationIssue::new(
                "schema_version",
                format!("expected schema_version {RESUME_SCHEMA_VERSION}"),
            ));
        }
        if self.basics.name.trim().is_empty() {
            issues.push(ValidationIssue::new("basics.name", "name is required"));
        }
        if !self.basics.email.contains('@') {
            issues.push(ValidationIssue::new("basics.email", "email must contain @"));
        }
        for (index, link) in self.basics.links.iter().enumerate() {
            if link.label.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    format!("basics.links[{index}].label"),
                    "link label is required",
                ));
            }
            if Url::parse(&link.url).is_err() {
                issues.push(ValidationIssue::new(
                    format!("basics.links[{index}].url"),
                    "link must be a valid URL",
                ));
            }
        }
        if self.work.is_empty() {
            issues.push(ValidationIssue::new(
                "work",
                "at least one work entry is required",
            ));
        }
        for (index, work) in self.work.iter().enumerate() {
            if work.company.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    format!("work[{index}].company"),
                    "company is required",
                ));
            }
            if work.position.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    format!("work[{index}].position"),
                    "position is required",
                ));
            }
            if work.highlights.is_empty() {
                issues.push(ValidationIssue::new(
                    format!("work[{index}].highlights"),
                    "at least one highlight is required",
                ));
            }
        }
        if self.education.is_empty() {
            issues.push(ValidationIssue::new(
                "education",
                "at least one education entry is required",
            ));
        }
        for (index, skill_group) in self.skills.iter().enumerate() {
            if skill_group.category.trim().is_empty() {
                issues.push(ValidationIssue::new(
                    format!("skills[{index}].category"),
                    "skill category is required",
                ));
            }
            if skill_group.items.is_empty() {
                issues.push(ValidationIssue::new(
                    format!("skills[{index}].items"),
                    "at least one skill item is required",
                ));
            }
        }
        for (index, project) in self.projects.iter().enumerate() {
            if let Some(url) = &project.url {
                if Url::parse(url).is_err() {
                    issues.push(ValidationIssue::new(
                        format!("projects[{index}].url"),
                        "project URL must be valid",
                    ));
                }
            }
        }
        issues
    }
}
