---
title: FAQ
description: Frequently asked questions about Saint's Mile
sidebar:
  order: 10
---

## How long is the game?

The full campaign spans a Prologue plus 15 chapters across four life phases (Youth, Young Man, Adult Years, Older Years). A complete playthrough takes roughly **8-12 hours** depending on how much you explore encounters and how quickly you move through combat.

## Can I save anywhere?

Save and load is available between encounters and chapters. The game uses an auto-progression system within scenes, so saves happen at natural break points rather than mid-dialogue or mid-combat.

## Does my relay choice matter?

Yes. Relay decisions affect which story branches open, which evidence you collect, and how the conspiracy unfolds. Some choices gate entire encounters or change party dynamics. The game tracks your decisions through a state system that carries consequences across chapters.

## What are the system requirements?

Saint's Mile runs in the terminal and is extremely lightweight:

- **OS:** Windows 10+, macOS 12+, Linux (any modern distro)
- **Terminal:** Any terminal with 256-color support and UTF-8 (e.g., Windows Terminal, iTerm2, Alacritty, gnome-terminal)
- **Disk:** Under 10 MB
- **RAM:** Minimal (runs as a single Rust binary)
- **Network:** None. The game is fully offline.
- **Rust 1.80+** only needed if building from source

## Is there New Game+?

The current release (v1.0.1) does not include a New Game+ mode. The game is designed as a single narrative arc where choices carry weight precisely because they cannot be undone within a playthrough.

## Can I mod the game?

The game is open source under the MIT license. You can fork the repository and modify anything — chapters, encounters, combat tuning, party members. See [ARCHITECTURE.md](https://github.com/mcp-tool-shop-org/saints-mile/blob/main/ARCHITECTURE.md) for a module map and the [Implementation Guide](/saints-mile/handbook/implementation/) for extension points.

Save files use the human-readable RON format, so you can also edit game state directly if you want to experiment.

## How do I contribute?

Read the [Contributing Guide](https://github.com/mcp-tool-shop-org/saints-mile/blob/main/CONTRIBUTING.md) for setup instructions, code style, and the pull request process. The project uses a domain ownership model — check which files belong to which domain before editing.

Key requirements:
- Rust 1.80+
- Every code change must include tests
- Run `cargo fmt` and `cargo clippy` before submitting
- One logical change per commit

## Does the game connect to the internet?

No. Saint's Mile is completely offline. It does not collect telemetry, phone home, or access any network resources. See the [Threat Model](https://github.com/mcp-tool-shop-org/saints-mile/blob/main/README.md#threat-model) in the README for the full security posture.

## What terminal should I use?

Any modern terminal works. Our recommendations by platform:

- **Windows:** Windows Terminal (built into Windows 11, free on Microsoft Store for Windows 10)
- **macOS:** iTerm2 or Alacritty (Terminal.app works but has limited color fidelity)
- **Linux:** Your default terminal is almost certainly fine. Alacritty and Kitty are excellent alternatives.

See the [Troubleshooting guide](/saints-mile/handbook/troubleshooting/) for details on terminal configuration.
