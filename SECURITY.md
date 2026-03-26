# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.x     | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Email:** 64996768+mcp-tool-shop@users.noreply.github.com
2. **Do not** open a public issue for security vulnerabilities
3. Include steps to reproduce and impact assessment

We aim to respond within 72 hours and release a fix within 7 days for confirmed vulnerabilities.

## Threat Model

Saint's Mile is a single-player terminal game with no network access. Primary risks:

- **Save file parsing:** RON deserialization of user-editable save files. Mitigated by typed deserialization (serde + RON) with no unsafe blocks.
- **Terminal escape sequences:** Managed by crossterm/ratatui libraries with established security track records.
- **No telemetry, no network, no secrets** — the game runs entirely offline.
