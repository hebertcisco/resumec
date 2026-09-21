# Architecture

`resumec` is a Rust workspace-style single crate with a thin binary and a
library that owns all domain logic. The goal is clean separation of
responsibilities so CLI, MCP, and future API surfaces can share the same core.

## Layers

```text
┌─────────────────────────────────────────────┐
│  main.rs  (parse CLI, print errors, exit)   │
└─────────────────────┬───────────────────────┘
                      │
┌─────────────────────▼───────────────────────┐
│  lib.rs / cli.rs / commands/* / mcp.rs      │
│  orchestration & adapters                   │
└─────────────────────┬───────────────────────┘
                      │
┌─────────────────────▼───────────────────────┐
│  build / resume_io / theme_io / preferences │
│  domain workflows                           │
└───────────────┬─────────────┬───────────────┘
                │             │
┌───────────────▼──┐   ┌──────▼───────────────┐
│  model/*         │   │  render/* → output/* │
│  data + validate │   │  layout → PDF/DOCX   │
└──────────────────┘   └──────────────────────┘
```

## Module map

| Module | Role |
|--------|------|
| `error` | `AppError`, `ValidationIssue`, exit codes |
| `model` | Resume / theme / preferences / machine results |
| `cli` | Clap definitions only |
| `commands` | Side-effecting command handlers |
| `build` | Validate + render + write orchestration |
| `resume_io` / `theme_io` / `preferences_io` | Load / parse / persist |
| `render` | Localized section text + wrapping |
| `output` | Format writers (`pdf`, `docx`) and overwrite policy |
| `mcp` | JSON-lines MCP stdio adapter |
| `paths` / `constants` / `util` | Shared infrastructure |

## Design rules

1. **Binary stays thin** — no domain logic in `main.rs`.
2. **Models do not I/O** — validation is pure; loaders live in `*_io` modules.
3. **Rendering is format-agnostic** — produce `DocumentLine`s, then write PDF/DOCX.
4. **Stable automation contract** — `--json-output` and exit codes are part of the public surface; change them carefully and document in `CHANGELOG.md`.
5. **Themes are data** — built-ins ship under `assets/themes/`; user themes live in the platform config directory.
6. **Generated paths stay contained** — output and theme names are portable file stems, never arbitrary paths.

## Testing strategy

- Unit tests next to library code (`src/tests.rs` and module tests as needed)
- Integration tests in `tests/cli.rs` exercising the real binary
- CI runs `fmt`, `clippy`, `test`, validate, and a sample build
