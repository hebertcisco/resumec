use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::build::{BuildOptions, build_resume, validate_resume};
use crate::cli::{McpArgs, McpCommand};
use crate::error::AppError;
use crate::model::{MachineResult, OutputFormat};
use crate::paths::AppPaths;
use crate::preferences_io::load_preferences;
use crate::resume_io::load_resume;
use crate::theme_io::list_themes;
use crate::util::derive_output_name_from_path;

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

pub fn handle_mcp(args: McpArgs) -> Result<(), AppError> {
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
