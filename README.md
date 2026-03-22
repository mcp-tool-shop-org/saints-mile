<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/saints-mile/readme.png" width="400" alt="Saint's Mile" />
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/actions"><img src="https://github.com/mcp-tool-shop-org/saints-mile/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://mcp-tool-shop-org.github.io/saints-mile/"><img src="https://img.shields.io/badge/Landing_Page-saints--mile-d97706" alt="Landing Page"></a>
</p>

A frontier JRPG for the adults who loved those games first.

Saint's Mile is a turn-based party RPG set in the Cinder Basin — a frontier territory being reshaped by rail, water, and law. You play as Galen Rook, a man whose name gets to town before he does, across four decades of a life lived under a wanted poster someone else wrote.

Built in Rust for the terminal. No graphics bloat. Full focus on deterministic mechanics, party combat, and a story that trusts its audience.

## What This Is

- A **90s-style JRPG** with a 4-slot party, distinct roles, duo techniques, and turn-based combat
- A **frontier western** where reputation is a web, distance changes decisions, and the trail is the dungeon
- A **game for adults** — themes of regret, duty, compromise, aging, loyalty, and starting over
- A **terminal-native experience** — runs in any terminal on earth via [ratatui](https://ratatui.rs/)

## The Story

The game spans almost four decades: from a nineteen-year-old deputy's runner who still thinks law and truth are related, to a hard young gunman carrying someone else's crime, to a fully grown outlaw crossing a dying basin with a party of damaged specialists, to an older man forced to decide whether a life can be redeemed by deeds, by truth, or not at all.

The surface conflict is rail, water, and land. The deeper conflict is who gets to write the story of what happened at Saint's Mile.

## Combat

Standoff tension opens every significant fight — hands hover, nerve is tested, initiative is earned. Then a full party-based JRPG battle system takes over: four active members from a roster of six, each with unique command sets, skill lines that deepen through story and bond, and duo techniques that reward party investment.

The western layer changes the mechanics, not just the flavor: ammo instead of MP, nerve instead of morale, grit instead of defense buffs, wounds that linger between fights.

## The Party

| Character | Role | Battle Identity |
|-----------|------|----------------|
| **Galen Rook** | Gunhand | Precision, called shots, field command. Evolves by age. |
| **Eli Winter** | Grifter | Nerve attacks, disruption, cheap tricks. Loyalty unlocks late. |
| **Dr. Ada Mercer** | Sawbones | Healing, wound management, weakness revelation. |
| **Rosa Varela** | Ranch Hand | Lasso crowd control, front-line tanking, positional pressure. |
| **Rev. Miriam Slate** | Preacher | Channeled buffs, nerve support, crowd management. |
| **Lucien "Fuse" Marr** | Dynamiter | Delayed AOE, environmental destruction, terrain reshaping. |

## Status

**Phase 1 — Production Spine.** Full campaign designed (Prologue + 15 chapters). Build Constitution and Runtime Contracts locked. Opening arc implementation next.

## Threat Model

Saint's Mile is a single-player offline game. It does not:
- Connect to the internet
- Collect telemetry or analytics
- Access files outside its own save directory
- Require any permissions beyond terminal I/O

Save files are stored in RON format in a user-accessible directory.

## Requirements

- Rust 1.75+ (2021 edition)
- Any terminal with 256-color support

## License

MIT

---

Built by <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
