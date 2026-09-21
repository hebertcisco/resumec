use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::constants::APP_NAME;
use crate::model::OutputFormat;

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
