# Contributing to resumec

Thanks for your interest in contributing. This document explains how to set up a development environment, propose changes, and keep the project healthy for FOSS collaboration.

## Code of conduct

Participation is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By contributing, you agree to uphold it.

## Development setup

Requirements:

- Rust 1.85 or newer (`rustup` recommended)
- Git

```bash
git clone https://github.com/hebertcisco/resumec.git
cd resumec
cargo build
cargo test
```

Optional quality checks (also run in CI):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Project layout

| Path | Responsibility |
|------|----------------|
| `src/main.rs` | Binary entrypoint and process exit handling |
| `src/lib.rs` | Public library surface and CLI dispatcher |
| `src/cli.rs` | Clap command definitions |
| `src/commands/` | Command handlers (`build`, `validate`, `theme`, …) |
| `src/model/` | Resume, theme, and result data models |
| `src/render/` | Text layout and localization |
| `src/output/` | PDF and DOCX writers |
| `src/mcp.rs` | Local MCP stdio server |
| `schemas/` | JSON Schema for resume and theme |
| `examples/` | Sample resumes |
| `tests/` | Integration / CLI tests |
| `assets/themes/` | Built-in themes |

Prefer small modules with a single responsibility. Keep `main.rs` thin: parse args, call the library, map errors to exit codes.

## Making changes

1. Open an issue for larger features or behavior changes when practical.
2. Create a branch from `main`.
3. Keep commits focused and messages clear (imperative mood, e.g. `fix PDF wrap for long bullets`).
4. Add or update tests for user-visible behavior.
5. Update docs (`README.md`, schemas, or this guide) when behavior or public CLI flags change.
6. Open a pull request against `main`.

## Pull request checklist

- [ ] `cargo test` passes locally
- [ ] `cargo fmt` and `cargo clippy` are clean
- [ ] New CLI flags or schema fields are documented
- [ ] Dual license (MIT OR Apache-2.0) is respected; do not add incompatible dependencies without discussion

## Reporting bugs

Include:

- `resumec` version (`resumec --version`)
- OS and architecture
- Minimal resume / theme input that reproduces the issue
- Exact command line and exit code
- Whether `--json-output` changes the symptom

Security-sensitive reports should follow [SECURITY.md](SECURITY.md) instead of a public issue.

## Licensing of contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in resumec is dual-licensed under MIT OR Apache-2.0, without additional terms.
