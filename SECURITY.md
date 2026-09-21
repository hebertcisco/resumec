# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| `0.1.x` | Yes |
| older   | No |

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.

Prefer one of these private channels:

1. GitHub Security Advisories for this repository (if enabled):
   [Report a vulnerability](https://github.com/hebertcisco/resumec/security/advisories/new)
2. Email the maintainer via the address listed on the GitHub profile for
   [@hebertcisco](https://github.com/hebertcisco), with subject
   `resumec security report`.

Include:

- A description of the issue and its impact
- Steps to reproduce (PoC if possible)
- Affected version / commit
- Any suggested fix (optional)

You should receive an acknowledgment within **7 days**. We will keep you informed
of the remediation plan when practical.

## Scope notes

`resumec` is a local CLI. Typical concerns include:

- Path traversal or unexpected file overwrite when writing outputs
- Unsafe handling of untrusted resume / theme YAML, JSON, or TOML
- Network fetch of themes (`theme install <url>`) following attacker-controlled URLs
- MCP stdio protocol confusion that could confuse automation wrappers

Out of scope unless they enable privilege escalation or remote compromise:

- Denial of service via deliberately huge or pathological documents
- Cosmetic PDF/DOCX rendering bugs

## Safe disclosure

We ask that you give us reasonable time to release a fix before public disclosure.
We are happy to credit reporters who wish to be named in the changelog.
