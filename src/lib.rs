//! resumec library: ATS-friendly resume compiler for PDF and DOCX.

pub mod build;
pub mod cli;
pub mod commands;
pub mod constants;
pub mod error;
pub mod json_out;
pub mod mcp;
pub mod model;
pub mod paths;
pub mod preferences_io;
pub mod render;
pub mod resume_io;
pub mod theme_io;
pub mod util;

mod output;

use clap::CommandFactory;
use clap_complete::generate;
use std::io;

pub use build::{BuildOptions, build_resume, validate_resume};
pub use cli::{Cli, Commands};
pub use constants::EXAMPLE_RESUME_YAML;
pub use error::{AppError, ValidationIssue};
pub use json_out::print_json;
pub use model::{
    Award, Basics, BuildResult, Certification, EducationEntry, GeneratedOutput, LanguageSkill,
    Link, MachineResult, OutputFormat, Preferences, Project, Resume, SkillCategory, Theme,
    ThemeLayout, ThemePalette, ThemeSection, ThemeSpacing, ThemeTypography, WorkExperience,
};
pub use paths::AppPaths;
pub use resume_io::{load_resume, parse_resume_content};
pub use theme_io::{load_theme_by_name, load_theme_from_path, parse_theme_content};

use crate::cli::Commands as Cmd;
use crate::constants::APP_NAME;

/// Run the CLI command tree.
pub fn run_cli(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Cmd::Build(args) => commands::handle_build(args),
        Cmd::Validate(args) => commands::handle_validate(args),
        Cmd::Theme(args) => commands::handle_theme(args),
        Cmd::Init(args) => commands::handle_init(args),
        Cmd::Config(args) => commands::handle_config(args),
        Cmd::Mcp(args) => mcp::handle_mcp(args),
        Cmd::Completion(args) => {
            let mut command = Cli::command();
            generate(args.shell, &mut command, APP_NAME, &mut io::stdout());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
