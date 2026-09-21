# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

### Added

- Continuous integration workflow covering format, Clippy, tests, and example
  validation / build.

## [0.1.0] - 2026-09-21

### Added

- Initial public release of `resumec`: validate and build ATS-friendly resumes
  from JSON, TOML, or YAML into PDF and DOCX.
- Built-in themes: `classic`, `modern`, `minimal`.
- CLI commands: `build`, `validate`, `theme`, `init`, `config`, `mcp`,
  `completion`.
- JSON machine output mode with stable exit codes.
- Local MCP stdio tools for validation, build, themes, and preferences.
- Example resume, JSON schemas, and GitHub Actions workflow for resume builds.

[Unreleased]: https://github.com/hebertcisco/resumec/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/hebertcisco/resumec/releases/tag/v0.1.0
