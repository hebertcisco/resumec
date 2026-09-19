# resumec

`resumec` is a cross-platform Rust CLI that compiles ATS-friendly resume data from JSON, TOML, or YAML into final PDF and editable DOCX outputs.

## Highlights

- Single binary CLI for local terminals, CI, n8n, and local MCP use
- Canonical resume schema (`schema_version: 1`) shared across JSON/TOML/YAML
- Native PDF and DOCX generation without mandatory external runtime binaries
- Built-in `classic`, `modern`, and `minimal` themes
- JSON-first automation mode with stable exit codes
- User config and theme storage under the platform-standard config directory

## Installation

```bash
cargo install --path .
```

Build a release binary locally:

```bash
cargo build --release
```

## Resume schema

Schemas are published in:

- `/home/runner/work/resumec/resumec/schemas/resume.schema.json`
- `/home/runner/work/resumec/resumec/schemas/theme.schema.json`

An example resume lives at:

- `/home/runner/work/resumec/resumec/examples/resume.yaml`

Core fields:

- `schema_version`
- `basics`
- `work`
- `education`
- `skills`
- optional `certifications`, `projects`, `languages`, `awards`
- optional metadata `theme`, `locale`, `target_role`

## CLI usage

```text
resumec build <input> [options]
resumec validate <input>
resumec theme list
resumec theme new <name>
resumec theme install <path-or-url>
resumec init [--format json|toml|yaml]
resumec config show
resumec config set <key> <value>
resumec mcp serve
resumec completion <shell>
```

### Build a resume

```bash
resumec build examples/resume.yaml --format both --theme modern --output-dir dist --overwrite
```

### Validate only

```bash
resumec validate examples/resume.yaml
```

### Machine-readable mode

```bash
resumec build examples/resume.yaml --format both --json-output --overwrite
```

Example output:

```json
{
  "status": "success",
  "data": {
    "input_file": "examples/resume.yaml",
    "theme": "modern",
    "outputs": [
      {
        "format": "pdf",
        "path": "dist/resume.pdf",
        "bytes": 84213
      },
      {
        "format": "docx",
        "path": "dist/resume.docx",
        "bytes": 45120
      }
    ],
    "warnings": []
  },
  "warnings": [],
  "errors": []
}
```

## Exit codes

- `0`: success
- `1`: schema validation failure
- `2`: theme not found
- `3`: write/overwrite failure
- `4`: input parsing/read failure
- `5`: invalid theme definition
- `6`: network failure
- `7`: configuration failure
- `8`: MCP protocol failure

## Configuration directory

`resumec` stores application state in the platform config directory via `directories`:

- Windows: `%APPDATA%\resumec\`
- macOS: `~/Library/Application Support/resumec/`
- Linux: `~/.config/resumec/`

Contents:

- `preferences.json`
- `themes/`

Example:

```bash
resumec config set theme modern
resumec config set format both
resumec config show
```

## Themes

Built-in themes:

- `classic`: traditional single-column ATS-safe layout
- `modern`: accent color hierarchy with ATS-safe ordering
- `minimal`: compact presentation with reduced spacing

### Create a new theme

```bash
resumec theme new my-theme
```

This creates `./my-theme/theme.yml`.

### Install a theme

```bash
resumec theme install ./my-theme
resumec theme install https://example.com/theme.yml
```

## GitHub Actions example

Workflow included at `/home/runner/work/resumec/resumec/.github/workflows/resume.yml`.

Behavior:

- Pull requests run `resumec validate`
- Pushes to `main` run `resumec build --format both --json-output`
- Generated files are uploaded as artifacts

## n8n example

Use an **Execute Command** node:

```bash
resumec build /data/resume.yaml --format both --json-output --overwrite
```

Then parse the node stdout as JSON and read `data.outputs` for generated file paths.

## MCP mode

Start the local stdio server:

```bash
resumec mcp serve
```

Supported methods:

- `initialize`
- `tools/list`
- `tools/call`

Supported tools:

- `validate_resume`
- `build_resume`
- `list_themes`
- `get_preferences`

Example request line:

```json
{"id":1,"method":"tools/call","params":{"name":"validate_resume","arguments":{"input":"examples/resume.yaml"}}}
```

## Shell completion

```bash
resumec completion bash > /tmp/resumec.bash
resumec completion zsh > /tmp/_resumec
```

## Tests

```bash
cargo test
```

The test suite covers:

- parsing JSON, TOML, and YAML
- validation failures
- PDF/DOCX generation for all built-in themes
- JSON output behavior

## Notes on PDF generation

PDF output is generated natively with `printpdf`. No external browser or system PDF runtime is required.

## License

Dual-licensed under MIT or Apache-2.0.
