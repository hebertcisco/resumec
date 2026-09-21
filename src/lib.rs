use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use directories::ProjectDirs;
use docx_rs::{Docx, Paragraph, Run};
use printpdf::{
    BuiltinFont, Color, Mm, Op, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Pt, Rgb,
    TextItem, TextMatrix,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

const RESUME_SCHEMA_VERSION: u32 = 1;
const THEME_SCHEMA_VERSION: u32 = 1;
const APP_NAME: &str = "resumec";
const DEFAULT_PAGE_MARGIN_PT: f32 = 36.0;

const CLASSIC_THEME: &str = include_str!("../assets/themes/classic/theme.yml");
const MODERN_THEME: &str = include_str!("../assets/themes/modern/theme.yml");
const MINIMAL_THEME: &str = include_str!("../assets/themes/minimal/theme.yml");
const EXAMPLE_RESUME_YAML: &str = include_str!("../examples/resume.yaml");
const BUILTIN_THEMES: &[(&str, &str)] = &[
    ("classic", CLASSIC_THEME),
    ("modern", MODERN_THEME),
    ("minimal", MINIMAL_THEME),
];

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
    fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

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
        if self.name.trim().is_empty() {
            issues.push(ValidationIssue::new("name", "theme name is required"));
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

    fn visible_sections(&self) -> Vec<SectionId> {
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

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub preferences_path: PathBuf,
    pub themes_dir: PathBuf,
}

impl AppPaths {
    pub fn detect() -> Result<Self, AppError> {
        let project_dirs = ProjectDirs::from("", "", APP_NAME)
            .ok_or_else(|| AppError::Config("failed to determine config directory".to_string()))?;
        let config_dir = project_dirs.config_dir().to_path_buf();
        let themes_dir = config_dir.join("themes");
        let preferences_path = config_dir.join("preferences.json");
        Ok(Self {
            config_dir,
            preferences_path,
            themes_dir,
        })
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

#[derive(Debug, Parser)]
#[command(name = APP_NAME, version, about = "ATS-friendly resume compiler for PDF and DOCX outputs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Build(BuildArgs),
    Validate(ValidateArgs),
    Theme(ThemeArgs),
    Init(InitArgs),
    Config(ConfigArgs),
    Mcp(McpArgs),
    Completion(CompletionArgs),
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    pub input: PathBuf,
    #[arg(long, value_enum)]
    pub format: Option<OutputFormat>,
    #[arg(long)]
    pub theme: Option<String>,
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    #[arg(long)]
    pub output_name: Option<String>,
    #[arg(long)]
    pub json_output: bool,
    #[arg(long)]
    pub quiet: bool,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long)]
    pub overwrite: bool,
    #[arg(long, alias = "yes")]
    pub non_interactive: bool,
}

#[derive(Debug, Args)]
pub struct ValidateArgs {
    pub input: PathBuf,
    #[arg(long)]
    pub json_output: bool,
}

#[derive(Debug, Args)]
pub struct ThemeArgs {
    #[command(subcommand)]
    pub command: ThemeCommand,
}

#[derive(Debug, Subcommand)]
pub enum ThemeCommand {
    List,
    New { name: String },
    Install { source: String },
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long, value_enum, default_value_t = InitFormat::Yaml)]
    pub format: InitFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InitFormat {
    Json,
    Toml,
    Yaml,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Show,
    Set { key: String, value: String },
}

#[derive(Debug, Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Debug, Subcommand)]
pub enum McpCommand {
    Serve,
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    pub shell: Shell,
}

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

#[derive(Debug, Deserialize)]
struct McpRequest {
    #[serde(default)]
    id: serde_json::Value,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    id: serde_json::Value,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

pub fn run_cli(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Commands::Build(args) => handle_build(args),
        Commands::Validate(args) => handle_validate(args),
        Commands::Theme(args) => handle_theme(args),
        Commands::Init(args) => handle_init(args),
        Commands::Config(args) => handle_config(args),
        Commands::Mcp(args) => handle_mcp(args),
        Commands::Completion(args) => {
            let mut command = Cli::command();
            generate(args.shell, &mut command, APP_NAME, &mut io::stdout());
            Ok(())
        }
    }
}

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

    let parsers: Vec<fn(&str) -> Result<Resume, String>> = match extension {
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
        Err(AppError::Validation { issues })
    }
}

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

fn handle_build(args: BuildArgs) -> Result<(), AppError> {
    let paths = AppPaths::detect()?;
    let preferences = load_preferences(&paths)?;
    let theme_name = args
        .theme
        .or(preferences.default_theme.clone())
        .unwrap_or_else(|| "".to_string());
    let output_dir = args
        .output_dir
        .or_else(|| preferences.output_dir.clone().map(PathBuf::from))
        .unwrap_or(std::env::current_dir().map_err(|error| AppError::Config(error.to_string()))?);
    let output_name = args
        .output_name
        .unwrap_or_else(|| derive_output_name_from_path(&args.input));
    let options = BuildOptions {
        format: args
            .format
            .or(preferences.default_format)
            .unwrap_or(OutputFormat::Pdf),
        theme_name,
        output_dir,
        output_name,
        overwrite: args.overwrite,
        non_interactive: args.non_interactive,
        json_output: args.json_output,
    };
    let result = build_resume(&args.input, options)?;
    if args.json_output {
        print_json(&MachineResult::success(
            result.clone(),
            result.warnings.clone(),
        ))?;
    } else if !args.quiet {
        for output in &result.outputs {
            println!(
                "generated {:?}: {} ({} bytes)",
                output.format,
                output.path.display(),
                output.bytes
            );
        }
        if args.verbose {
            println!("theme: {}", result.theme);
        }
    }
    Ok(())
}

fn handle_validate(args: ValidateArgs) -> Result<(), AppError> {
    let resume = load_resume(&args.input)?;
    validate_resume(&resume)?;
    if args.json_output {
        print_json(&MachineResult::success(
            serde_json::json!({
                "input_file": args.input,
                "schema_version": resume.schema_version,
                "status": "valid"
            }),
            Vec::new(),
        ))?;
    } else {
        println!("{} is valid", args.input.display());
    }
    Ok(())
}

fn handle_theme(args: ThemeArgs) -> Result<(), AppError> {
    let paths = AppPaths::detect()?;
    paths.ensure_default_themes()?;
    match args.command {
        ThemeCommand::List => {
            for theme in list_themes(&paths)? {
                println!("{}\t{}", theme.name, theme.source);
            }
        }
        ThemeCommand::New { name } => {
            let destination = std::env::current_dir()
                .map_err(|error| AppError::Config(error.to_string()))?
                .join(&name);
            let theme_path = destination.join("theme.yml");
            fs::create_dir_all(&destination).map_err(|error| AppError::FileWrite {
                path: destination.display().to_string(),
                message: error.to_string(),
            })?;
            let content = default_theme_template(&name);
            fs::write(&theme_path, content).map_err(|error| AppError::FileWrite {
                path: theme_path.display().to_string(),
                message: error.to_string(),
            })?;
            println!("created {}", theme_path.display());
        }
        ThemeCommand::Install { source } => {
            let theme = install_theme(&paths, &source)?;
            println!("installed theme {}", theme.name);
        }
    }
    Ok(())
}

fn handle_init(args: InitArgs) -> Result<(), AppError> {
    let current_dir =
        std::env::current_dir().map_err(|error| AppError::Config(error.to_string()))?;
    let (file_name, content) = match args.format {
        InitFormat::Yaml => ("resume.yaml", EXAMPLE_RESUME_YAML.to_string()),
        InitFormat::Json => {
            let resume: Resume = serde_yaml::from_str(EXAMPLE_RESUME_YAML)
                .map_err(|error| AppError::Config(error.to_string()))?;
            (
                "resume.json",
                serde_json::to_string_pretty(&resume)
                    .map_err(|error| AppError::Config(error.to_string()))?,
            )
        }
        InitFormat::Toml => {
            let resume: Resume = serde_yaml::from_str(EXAMPLE_RESUME_YAML)
                .map_err(|error| AppError::Config(error.to_string()))?;
            (
                "resume.toml",
                toml::to_string_pretty(&resume)
                    .map_err(|error| AppError::Config(error.to_string()))?,
            )
        }
    };
    let path = current_dir.join(file_name);
    if path.exists() {
        return Err(AppError::OverwriteRequired {
            path: path.display().to_string(),
        });
    }
    fs::write(&path, content).map_err(|error| AppError::FileWrite {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    println!("created {}", path.display());
    Ok(())
}

fn handle_config(args: ConfigArgs) -> Result<(), AppError> {
    let paths = AppPaths::detect()?;
    paths.ensure()?;
    let mut preferences = load_preferences(&paths)?;
    match args.command {
        ConfigCommand::Show => {
            print_json(&preferences)?;
        }
        ConfigCommand::Set { key, value } => {
            match key.as_str() {
                "theme" => preferences.default_theme = Some(value),
                "format" => {
                    preferences.default_format = Some(match value.as_str() {
                        "pdf" => OutputFormat::Pdf,
                        "docx" => OutputFormat::Docx,
                        "both" => OutputFormat::Both,
                        _ => {
                            return Err(AppError::Config(
                                "format must be pdf, docx, or both".to_string(),
                            ));
                        }
                    })
                }
                "locale" => preferences.locale = Some(value),
                "output_dir" => preferences.output_dir = Some(value),
                _ => {
                    return Err(AppError::Config(
                        "supported keys: theme, format, locale, output_dir".to_string(),
                    ));
                }
            }
            save_preferences(&paths, &preferences)?;
            print_json(&preferences)?;
        }
    }
    Ok(())
}

fn handle_mcp(args: McpArgs) -> Result<(), AppError> {
    match args.command {
        McpCommand::Serve => serve_mcp(),
    }
}

fn serve_mcp() -> Result<(), AppError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| AppError::Mcp(error.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }
        let request: McpRequest = serde_json::from_str(&line)
            .map_err(|error| AppError::Mcp(format!("invalid JSON request: {error}")))?;
        let response = process_mcp_request(request);
        let payload =
            serde_json::to_string(&response).map_err(|error| AppError::Mcp(error.to_string()))?;
        writeln!(stdout, "{payload}").map_err(|error| AppError::Mcp(error.to_string()))?;
        stdout
            .flush()
            .map_err(|error| AppError::Mcp(error.to_string()))?;
    }
    Ok(())
}

fn process_mcp_request(request: McpRequest) -> McpResponse {
    let result = match request.method.as_str() {
        "initialize" => Ok(serde_json::json!({
            "protocol": "resumec-mcp/1",
            "capabilities": { "tools": true }
        })),
        "tools/list" => Ok(serde_json::json!({
            "tools": [
                {
                    "name": "validate_resume",
                    "description": "Validate a resume file against the canonical resumec schema.",
                    "inputSchema": {
                        "type": "object",
                        "required": ["input"],
                        "properties": { "input": { "type": "string" } }
                    }
                },
                {
                    "name": "build_resume",
                    "description": "Build PDF and/or DOCX outputs from a resume file.",
                    "inputSchema": {
                        "type": "object",
                        "required": ["input"],
                        "properties": {
                            "input": { "type": "string" },
                            "format": { "enum": ["pdf", "docx", "both"] },
                            "theme": { "type": "string" },
                            "output_dir": { "type": "string" },
                            "output_name": { "type": "string" },
                            "overwrite": { "type": "boolean" }
                        }
                    }
                },
                {
                    "name": "list_themes",
                    "description": "List built-in and installed themes.",
                    "inputSchema": { "type": "object", "properties": {} }
                },
                {
                    "name": "get_preferences",
                    "description": "Return the current resumec preferences.",
                    "inputSchema": { "type": "object", "properties": {} }
                }
            ]
        })),
        "tools/call" => call_mcp_tool(request.params),
        other => Err(AppError::Mcp(format!("unsupported method: {other}"))),
    };

    match result {
        Ok(value) => McpResponse {
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => McpResponse {
            id: request.id,
            result: None,
            error: Some(serde_json::json!({
                "code": error.exit_code(),
                "message": error.to_string(),
                "errors": error.issues()
            })),
        },
    }
}

fn call_mcp_tool(params: serde_json::Value) -> Result<serde_json::Value, AppError> {
    let name = params
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AppError::Mcp("tools/call requires a tool name".to_string()))?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    match name {
        "validate_resume" => {
            let input = arguments
                .get("input")
                .and_then(|value| value.as_str())
                .ok_or_else(|| AppError::Mcp("validate_resume requires input".to_string()))?;
            let resume = load_resume(Path::new(input))?;
            validate_resume(&resume)?;
            Ok(serde_json::json!({
                "status": "success",
                "input_file": input,
                "schema_version": resume.schema_version,
                "warnings": [],
                "errors": []
            }))
        }
        "build_resume" => {
            let input = arguments
                .get("input")
                .and_then(|value| value.as_str())
                .ok_or_else(|| AppError::Mcp("build_resume requires input".to_string()))?;
            let format = match arguments
                .get("format")
                .and_then(|value| value.as_str())
                .unwrap_or("pdf")
            {
                "pdf" => OutputFormat::Pdf,
                "docx" => OutputFormat::Docx,
                "both" => OutputFormat::Both,
                _ => {
                    return Err(AppError::Mcp(
                        "format must be pdf, docx, or both".to_string(),
                    ));
                }
            };
            let theme_name = arguments
                .get("theme")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let output_dir = arguments
                .get("output_dir")
                .and_then(|value| value.as_str())
                .map(PathBuf::from)
                .unwrap_or(
                    std::env::current_dir().map_err(|error| AppError::Config(error.to_string()))?,
                );
            let output_name = arguments
                .get("output_name")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| derive_output_name_from_path(Path::new(input)));
            let overwrite = arguments
                .get("overwrite")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            let result = build_resume(
                Path::new(input),
                BuildOptions {
                    format,
                    theme_name,
                    output_dir,
                    output_name,
                    overwrite,
                    non_interactive: true,
                    json_output: true,
                },
            )?;
            Ok(serde_json::to_value(MachineResult::success(
                result.clone(),
                result.warnings.clone(),
            ))
            .map_err(|error| AppError::Mcp(error.to_string()))?)
        }
        "list_themes" => {
            let paths = AppPaths::detect()?;
            let themes = list_themes(&paths)?;
            Ok(serde_json::to_value(themes).map_err(|error| AppError::Mcp(error.to_string()))?)
        }
        "get_preferences" => {
            let paths = AppPaths::detect()?;
            let preferences = load_preferences(&paths)?;
            Ok(serde_json::to_value(preferences)
                .map_err(|error| AppError::Mcp(error.to_string()))?)
        }
        _ => Err(AppError::Mcp(format!("unsupported tool: {name}"))),
    }
}

#[derive(Debug, Clone, Serialize)]
struct ThemeDescriptor {
    name: String,
    source: String,
}

fn list_themes(paths: &AppPaths) -> Result<Vec<ThemeDescriptor>, AppError> {
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

fn install_theme(paths: &AppPaths, source: &str) -> Result<Theme, AppError> {
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

#[derive(Debug, Clone)]
enum LineKind {
    Heading,
    Body,
}

#[derive(Debug, Clone)]
struct DocumentLine {
    kind: LineKind,
    text: String,
}

fn render_lines(resume: &Resume, theme: &Theme) -> Vec<DocumentLine> {
    let locale = resume.locale.as_deref();
    let mut lines = Vec::new();
    for section in theme.visible_sections() {
        match section {
            SectionId::Basics => {
                lines.push(DocumentLine {
                    kind: LineKind::Heading,
                    text: resume.basics.name.clone(),
                });
                if let Some(headline) = &resume.basics.headline {
                    lines.push(DocumentLine {
                        kind: LineKind::Body,
                        text: headline.clone(),
                    });
                }
                lines.push(DocumentLine {
                    kind: LineKind::Body,
                    text: join_non_empty([
                        Some(resume.basics.email.as_str()),
                        resume.basics.phone.as_deref(),
                        resume.basics.location.as_deref(),
                    ]),
                });
                if !resume.basics.links.is_empty() {
                    lines.push(DocumentLine {
                        kind: LineKind::Body,
                        text: resume
                            .basics
                            .links
                            .iter()
                            .map(|link| format!("{}: {}", link.label, link.url))
                            .collect::<Vec<_>>()
                            .join(" | "),
                    });
                }
                lines.push(blank_line());
            }
            SectionId::Summary => {
                if let Some(summary) = &resume.basics.summary {
                    lines.push(section_heading(localized_label("summary", locale)));
                    lines.push(DocumentLine {
                        kind: LineKind::Body,
                        text: summary.clone(),
                    });
                    lines.push(blank_line());
                }
            }
            SectionId::Work => {
                if !resume.work.is_empty() {
                    lines.push(section_heading(localized_label("work", locale)));
                    for entry in &resume.work {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: format!(
                                "{} — {} ({})",
                                entry.position,
                                entry.company,
                                join_non_empty([
                                    format_date_range(
                                        entry.start_date.as_deref(),
                                        entry.end_date.as_deref()
                                    )
                                    .as_deref(),
                                    entry.location.as_deref(),
                                    None,
                                ])
                            ),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                        for bullet in &entry.highlights {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: format!("• {bullet}"),
                            });
                        }
                        lines.push(blank_line());
                    }
                }
            }
            SectionId::Education => {
                if !resume.education.is_empty() {
                    lines.push(section_heading(localized_label("education", locale)));
                    for entry in &resume.education {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: format!(
                                "{}{} — {}{}",
                                entry.study_type.clone().unwrap_or_default(),
                                if entry.study_type.is_some() { " " } else { "" },
                                entry.area,
                                if entry.institution.is_empty() {
                                    "".to_string()
                                } else {
                                    format!(" | {}", entry.institution)
                                }
                            ),
                        });
                        let details = join_non_empty([
                            format_date_range(
                                entry.start_date.as_deref(),
                                entry.end_date.as_deref(),
                            )
                            .as_deref(),
                            entry.location.as_deref(),
                            entry.score.as_deref(),
                        ]);
                        if !details.is_empty() {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: details,
                            });
                        }
                        lines.push(blank_line());
                    }
                }
            }
            SectionId::Skills => {
                if !resume.skills.is_empty() {
                    lines.push(section_heading(localized_label("skills", locale)));
                    for entry in &resume.skills {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: format!("{}: {}", entry.category, entry.items.join(", ")),
                        });
                    }
                    lines.push(blank_line());
                }
            }
            SectionId::Certifications => {
                if !resume.certifications.is_empty() {
                    lines.push(section_heading(localized_label("certifications", locale)));
                    for entry in &resume.certifications {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.name.as_str()),
                                entry.issuer.as_deref(),
                                entry.date.as_deref(),
                            ]),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                    }
                    lines.push(blank_line());
                }
            }
            SectionId::Projects => {
                if !resume.projects.is_empty() {
                    lines.push(section_heading(localized_label("projects", locale)));
                    for entry in &resume.projects {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.name.as_str()),
                                entry.url.as_deref(),
                                None,
                            ]),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                        for bullet in &entry.highlights {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: format!("• {bullet}"),
                            });
                        }
                        lines.push(blank_line());
                    }
                }
            }
            SectionId::Languages => {
                if !resume.languages.is_empty() {
                    lines.push(section_heading(localized_label("languages", locale)));
                    for entry in &resume.languages {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.name.as_str()),
                                entry.fluency.as_deref(),
                                None,
                            ]),
                        });
                    }
                    lines.push(blank_line());
                }
            }
            SectionId::Awards => {
                if !resume.awards.is_empty() {
                    lines.push(section_heading(localized_label("awards", locale)));
                    for entry in &resume.awards {
                        lines.push(DocumentLine {
                            kind: LineKind::Body,
                            text: join_non_empty([
                                Some(entry.title.as_str()),
                                entry.issuer.as_deref(),
                                entry.date.as_deref(),
                            ]),
                        });
                        if let Some(summary) = &entry.summary {
                            lines.push(DocumentLine {
                                kind: LineKind::Body,
                                text: summary.clone(),
                            });
                        }
                    }
                    lines.push(blank_line());
                }
            }
        }
    }
    while matches!(lines.last(), Some(DocumentLine { text, .. }) if text.is_empty()) {
        lines.pop();
    }
    lines
}

fn write_pdf(
    resume: &Resume,
    theme: &Theme,
    lines: &[DocumentLine],
    options: &BuildOptions,
) -> Result<GeneratedOutput, AppError> {
    let destination = options
        .output_dir
        .join(format!("{}.pdf", options.output_name));
    ensure_can_write(&destination, options)?;
    let mut doc = PdfDocument::new(&resume.basics.name);
    let mut pages = Vec::new();
    let heading_size = theme.typography.heading_size_pt;
    let body_size = theme.typography.base_size_pt;
    let margin_mm = pt_to_mm(theme.spacing.page_margin_pt.max(DEFAULT_PAGE_MARGIN_PT));
    let width_mm = 210.0f32;
    let height_mm = 297.0f32;
    let max_width_pt = Mm(width_mm - (margin_mm * 2.0)).into_pt().0;
    let layout_lines = layout_wrapped_lines(lines, heading_size, body_size, max_width_pt);
    let mut y = height_mm - margin_mm;
    let mut ops = start_text_ops();

    for line in &layout_lines {
        let size = match line.kind {
            LineKind::Heading => heading_size,
            LineKind::Body => body_size,
        };
        let line_height_pt = if line.text.is_empty() {
            theme.spacing.section_gap_pt.max(theme.spacing.item_gap_pt)
        } else {
            (size * 1.15) + theme.spacing.item_gap_pt
        };
        let consumed = pt_to_mm(line_height_pt);
        if y - consumed < margin_mm {
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);
            pages.push(PdfPage::new(Mm(width_mm), Mm(height_mm), ops));
            ops = start_text_ops();
            y = height_mm - margin_mm;
        }
        if !line.text.is_empty() {
            ops.push(Op::SetFillColor {
                col: color_for_line(theme, &line.kind),
            });
            ops.push(Op::SetFont {
                font: font_for_line(&line.kind),
                size: Pt(size),
            });
            // printpdf serializes SetTextCursor as relative Td; use Tm for absolute page coords.
            ops.push(Op::SetTextMatrix {
                matrix: TextMatrix::Translate(Mm(margin_mm).into(), Mm(y).into()),
            });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(line.text.clone())],
            });
        }
        y -= consumed;
    }

    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);
    pages.push(PdfPage::new(Mm(width_mm), Mm(height_mm), ops));

    let document = doc.with_pages(pages);
    let bytes = document.save(&PdfSaveOptions::default(), &mut Vec::new());
    fs::write(&destination, &bytes).map_err(|error| AppError::FileWrite {
        path: destination.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(GeneratedOutput {
        format: OutputFormat::Pdf,
        path: destination,
        bytes: bytes.len() as u64,
    })
}

fn write_docx(
    _resume: &Resume,
    theme: &Theme,
    lines: &[DocumentLine],
    options: &BuildOptions,
) -> Result<GeneratedOutput, AppError> {
    let destination = options
        .output_dir
        .join(format!("{}.docx", options.output_name));
    ensure_can_write(&destination, options)?;
    let file = fs::File::create(&destination).map_err(|error| AppError::FileWrite {
        path: destination.display().to_string(),
        message: error.to_string(),
    })?;
    let mut docx = Docx::new();
    for line in lines {
        let mut run = Run::new().add_text(line.text.clone());
        match line.kind {
            LineKind::Heading => {
                run = run
                    .bold()
                    .size((theme.typography.heading_size_pt * 2.0) as usize)
                    .color(strip_hash(&theme.palette.primary));
            }
            LineKind::Body => {
                run = run
                    .size((theme.typography.base_size_pt * 2.0) as usize)
                    .color(strip_hash(&theme.palette.text));
            }
        }
        docx = docx.add_paragraph(Paragraph::new().add_run(run));
    }
    docx.build()
        .pack(file)
        .map_err(|error| AppError::FileWrite {
            path: destination.display().to_string(),
            message: error.to_string(),
        })?;
    let bytes = fs::metadata(&destination)
        .map_err(|error| AppError::FileRead {
            path: destination.display().to_string(),
            message: error.to_string(),
        })?
        .len();
    Ok(GeneratedOutput {
        format: OutputFormat::Docx,
        path: destination,
        bytes,
    })
}

fn ensure_can_write(path: &Path, options: &BuildOptions) -> Result<(), AppError> {
    if path.exists() && !options.overwrite {
        if options.json_output
            || options.non_interactive
            || !io::stdin().is_terminal()
            || !io::stdout().is_terminal()
        {
            return Err(AppError::OverwriteRequired {
                path: path.display().to_string(),
            });
        }
        print!("{} exists. Overwrite? [y/N]: ", path.display());
        io::stdout().flush().map_err(|error| AppError::FileWrite {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|error| AppError::FileRead {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            return Err(AppError::OverwriteRequired {
                path: path.display().to_string(),
            });
        }
    }
    Ok(())
}

fn start_text_ops() -> Vec<Op> {
    vec![Op::SaveGraphicsState, Op::StartTextSection]
}

fn color_for_line(theme: &Theme, kind: &LineKind) -> Color {
    let hex = match kind {
        LineKind::Heading => &theme.palette.primary,
        LineKind::Body => &theme.palette.text,
    };
    let (r, g, b) = hex_to_rgb(hex);
    Color::Rgb(Rgb::new(r, g, b, None))
}

fn font_for_line(kind: &LineKind) -> PdfFontHandle {
    match kind {
        LineKind::Heading => PdfFontHandle::Builtin(BuiltinFont::HelveticaBold),
        LineKind::Body => PdfFontHandle::Builtin(BuiltinFont::Helvetica),
    }
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

fn builtin_theme_sources() -> &'static [(&'static str, &'static str)] {
    BUILTIN_THEMES
}

fn builtin_theme_source(name: &str) -> Option<&'static str> {
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

fn default_theme_template(name: &str) -> String {
    format!(
        "schema_version: 1\nname: {name}\ndescription: Custom resumec theme\nats_safe: true\npalette:\n  primary: \"#1F2933\"\n  secondary: \"#334E68\"\n  accent: \"#486581\"\n  text: \"#102A43\"\n  muted: \"#627D98\"\ntypography:\n  font_family: Helvetica\n  base_size_pt: 11.0\n  heading_size_pt: 14.0\nspacing:\n  section_gap_pt: 12.0\n  item_gap_pt: 6.0\nlayout:\n  columns: 1\nsections:\n  - id: basics\n    visible: true\n  - id: summary\n    visible: true\n  - id: work\n    visible: true\n  - id: education\n    visible: true\n  - id: skills\n    visible: true\n"
    )
}

fn derive_output_name_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(sanitize_filename)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "resume".to_string())
}

fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character.to_ascii_lowercase(),
            _ => '-',
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn looks_like_url(value: &str) -> bool {
    matches!(Url::parse(value), Ok(url) if matches!(url.scheme(), "http" | "https"))
}

fn join_non_empty<'a, I>(items: I) -> String
where
    I: IntoIterator<Item = Option<&'a str>>,
{
    items
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(" | ")
}

fn format_date_range(start: Option<&str>, end: Option<&str>) -> Option<String> {
    let value = join_non_empty([start, end]);
    if value.is_empty() {
        None
    } else {
        Some(value.replace(" | ", " – "))
    }
}

fn strip_hash(color: &str) -> &str {
    color.strip_prefix('#').unwrap_or(color)
}

fn hex_to_rgb(color: &str) -> (f32, f32, f32) {
    let stripped = strip_hash(color);
    let parse = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&stripped[range], 16).unwrap_or_default() as f32 / 255.0
    };
    (parse(0..2), parse(2..4), parse(4..6))
}

fn is_hex_color(color: &str) -> bool {
    let stripped = strip_hash(color);
    stripped.len() == 6
        && stripped
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn pt_to_mm(pt: f32) -> f32 {
    pt * 0.352_778
}

fn blank_line() -> DocumentLine {
    DocumentLine {
        kind: LineKind::Body,
        text: String::new(),
    }
}

fn section_heading(title: &str) -> DocumentLine {
    DocumentLine {
        kind: LineKind::Heading,
        text: title.to_string(),
    }
}

fn is_portuguese_locale(locale: Option<&str>) -> bool {
    locale
        .map(|value| {
            let normalized = value.to_ascii_lowercase();
            normalized == "pt" || normalized.starts_with("pt-") || normalized.starts_with("pt_")
        })
        .unwrap_or(false)
}

fn localized_label(key: &str, locale: Option<&str>) -> &'static str {
    let portuguese = is_portuguese_locale(locale);
    match (key, portuguese) {
        ("summary", true) => "Resumo",
        ("summary", false) => "Summary",
        ("work", true) => "Experiência Profissional",
        ("work", false) => "Work Experience",
        ("education", true) => "Educação",
        ("education", false) => "Education",
        ("skills", true) => "Habilidades",
        ("skills", false) => "Skills",
        ("certifications", true) => "Certificações",
        ("certifications", false) => "Certifications",
        ("projects", true) => "Projetos",
        ("projects", false) => "Projects",
        ("languages", true) => "Idiomas",
        ("languages", false) => "Languages",
        ("awards", true) => "Realizações",
        ("awards", false) => "Awards",
        _ => "Section",
    }
}

fn layout_wrapped_lines(
    lines: &[DocumentLine],
    heading_size: f32,
    body_size: f32,
    max_width_pt: f32,
) -> Vec<DocumentLine> {
    let mut wrapped = Vec::new();
    for line in lines {
        if line.text.is_empty() {
            wrapped.push(line.clone());
            continue;
        }
        let size = match line.kind {
            LineKind::Heading => heading_size,
            LineKind::Body => body_size,
        };
        let bold = matches!(line.kind, LineKind::Heading);
        for segment in wrap_text(&line.text, max_width_pt, size, bold) {
            wrapped.push(DocumentLine {
                kind: line.kind.clone(),
                text: segment,
            });
        }
    }
    wrapped
}

fn wrap_text(text: &str, max_width_pt: f32, font_size: f32, bold: bool) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    if estimate_text_width_pt(text, font_size, bold) <= max_width_pt {
        return vec![text.to_string()];
    }

    let indent = if text.starts_with('•') {
        "  "
    } else {
        ""
    };
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if estimate_text_width_pt(&candidate, font_size, bold) <= max_width_pt {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(current);
        }
        let continuation = if lines.is_empty() {
            word.to_string()
        } else {
            format!("{indent}{word}")
        };
        if estimate_text_width_pt(&continuation, font_size, bold) <= max_width_pt {
            current = continuation;
        } else {
            // Hard-split oversized tokens so layout never overflows the page width.
            let chunks = split_oversized_token(&continuation, max_width_pt, font_size, bold);
            if let Some((last, rest)) = chunks.split_last() {
                lines.extend(rest.iter().cloned());
                current = last.clone();
            } else {
                current = continuation;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        vec![text.to_string()]
    } else {
        lines
    }
}

fn split_oversized_token(token: &str, max_width_pt: f32, font_size: f32, bold: bool) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for character in token.chars() {
        let candidate = format!("{current}{character}");
        if !current.is_empty()
            && estimate_text_width_pt(&candidate, font_size, bold) > max_width_pt
        {
            chunks.push(current);
            current = character.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn estimate_text_width_pt(text: &str, font_size: f32, bold: bool) -> f32 {
    let average = if bold { 0.55 } else { 0.50 };
    text.chars()
        .map(|character| {
            let factor = match character {
                'i' | 'l' | 'I' | 'j' | 't' | 'f' | 'r' | '.' | ',' | ':' | ';' | '!' | '|'
                | '\'' | '•' => 0.28,
                'm' | 'w' | 'M' | 'W' => 0.85,
                ' ' => 0.28,
                _ => average,
            };
            factor * font_size
        })
        .sum()
}

fn default_true() -> bool {
    true
}

fn default_page_margin_pt() -> f32 {
    DEFAULT_PAGE_MARGIN_PT
}

pub fn print_json<T: Serialize>(value: &T) -> Result<(), AppError> {
    serde_json::to_writer_pretty(io::stdout(), value)
        .map_err(|error| AppError::Config(error.to_string()))?;
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn parses_yaml_resume() {
        let path = repo_root().join("examples/resume.yaml");
        let resume = load_resume(&path).expect("yaml should parse");
        assert_eq!(resume.basics.name, "Ana Souza");
    }

    #[test]
    fn validates_resume_schema_version() {
        let mut resume: Resume =
            serde_yaml::from_str(EXAMPLE_RESUME_YAML).expect("fixture should parse");
        resume.schema_version = 2;
        let issues = resume.validate();
        assert!(issues.iter().any(|issue| issue.field == "schema_version"));
    }

    #[test]
    fn loads_builtin_theme() {
        let paths = AppPaths::detect().expect("paths available");
        let theme = load_theme_by_name("classic", &paths).expect("classic theme should exist");
        assert_eq!(theme.name, "classic");
    }

    #[test]
    fn builds_both_formats_for_all_builtin_themes() {
        let input = repo_root().join("examples/resume.yaml");
        for theme_name in ["classic", "modern", "minimal"] {
            let temp = tempdir().expect("temp dir");
            let result = build_resume(
                &input,
                BuildOptions {
                    format: OutputFormat::Both,
                    theme_name: theme_name.to_string(),
                    output_dir: temp.path().to_path_buf(),
                    output_name: theme_name.to_string(),
                    overwrite: true,
                    non_interactive: true,
                    json_output: true,
                },
            )
            .expect("build should succeed");
            assert_eq!(result.outputs.len(), 2);
            for output in result.outputs {
                assert!(
                    output.path.exists(),
                    "{} should exist",
                    output.path.display()
                );
                assert!(
                    output.bytes > 0,
                    "{} should not be empty",
                    output.path.display()
                );
            }
        }
    }

    #[test]
    fn pdf_keeps_multiline_resume_content() {
        let input = repo_root().join("examples/resume.yaml");
        let temp = tempdir().expect("temp dir");
        let result = build_resume(
            &input,
            BuildOptions {
                format: OutputFormat::Pdf,
                theme_name: "modern".to_string(),
                output_dir: temp.path().to_path_buf(),
                output_name: "matrix-check".to_string(),
                overwrite: true,
                non_interactive: true,
                json_output: true,
            },
        )
        .expect("build should succeed");
        let bytes = fs::read(&result.outputs[0].path).expect("pdf should be readable");
        assert!(bytes.starts_with(b"%PDF"));
        assert!(bytes.len() > 1_000);
        assert_eq!(result.outputs[0].bytes, bytes.len() as u64);
    }

    #[test]
    fn wraps_long_lines_and_localizes_portuguese_labels() {
        assert_eq!(localized_label("work", Some("pt-BR")), "Experiência Profissional");
        assert_eq!(localized_label("work", Some("en-US")), "Work Experience");
        let wrapped = wrap_text(
            "• Desenvolvimento full stack end-to-end com React Native, Laravel e Node.js para produtos white-label em produção.",
            220.0,
            10.5,
            false,
        );
        assert!(wrapped.len() > 1);
        assert!(wrapped[0].starts_with('•'));
        assert!(wrapped[1].starts_with("  "));
    }
}
