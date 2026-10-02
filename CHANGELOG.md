# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed
- **Scene prose wraps at the terminal width.** It was drawn one logical line per
  row with no wrapping, so at the 100×30 terminal PLAYTEST.md recommends, and the
  80×24 minimum, the end of every long line was cut off at the right edge ("…drinks
  from a nearly dry trough be"). The dialogue widget now word-wraps to the width
  it draws at, keeps the two-space margin on continuation rows, and reveals
  characters across rows that were wrapped from the full line, so words do not
  jump rows as they type out. The auto-scroll counts wrapped rows, and it now
  measures the inside of the border rather than the outer area, which was two
  rows taller and let the newest line slip off the bottom. Found by
  [ai-playtest](https://github.com/mcp-tool-shop-org/ai-playtest) driving the
  release build through a pseudo-terminal.

## [1.0.2] - 2026-04-06

### Added
- Skill registry with age-variant skill progression
- Duo-tech system (paired character abilities)
- NPC ally AI — named characters (Eli, Ada, Cal, etc.) use role-specific combat behavior
- Flee action with escapable encounter flag
- Pressure engine runtime (bar tracking, action processing, threshold resolution)
- Fear cascade — nerve breaks chain to allies on same side
- Combo system — consecutive same-skill uses boost damage (1.1x/1.2x)
- Terrain effects — burning/flooding damage at round boundaries
- Cooldown system — skill cooldowns tick down each combat round
- Cover mechanics with partial cover damage reduction
- Status effects (Bleeding, Stunned, Inspired, Suppressed)
- Wound recovery (Ada-gated healing, rest recovery for minor wounds)
- Evidence/investigation system with relay branch evidence modifiers
- Chapter progression validation
- Game settings (text speed, persistent to settings.ron)
- Save system improvements (auto-save, atomic writes with .bak backup, delete)
- Pause screen (Esc/Ctrl+P), Status screen (Tab), Error screen, Confirm Quit
- Combat HUD footer (ammo/nerve/wounds)
- ARCHITECTURE.md and CONTRIBUTING.md
- GitHub issue/PR templates
- Handbook: FAQ and troubleshooting pages
- verify.sh script
- 508 tests (up from 151)

### Fixed
- All combat systems wired into live UI loop (NPC behavior, cooldowns, fear cascade, combos, terrain)
- Content gaps in Ch5/8/11 relay branches, Ch10 hearing evidence, Ch13-15 memory callbacks
- Chapter count documentation (Prologue + 15 chapters)

## [1.0.1] - 2026-03-25

### Added
- `--version` / `-V` and `--help` / `-h` CLI flags
- CI workflow (cargo check + cargo test on push/PR)
- SECURITY.md
- CHANGELOG.md
- Version alignment test

### Fixed
- SHA-pinned GitHub Actions in release workflow

## [1.0.0] - 2026-03-22

### Added
- Initial release — Prologue + 15 chapters, full combat system, save/load, TUI rendering
- Standoff and combat encounter systems
- Convoy and crowd mechanics
- Environmental effects and reckoning system
