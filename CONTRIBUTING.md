# Contributing to Saint's Mile

Thank you for your interest in contributing to Saint's Mile, a frontier JRPG built as a Rust TUI application using Ratatui.

## Prerequisites

- **Rust 1.80+** (install via [rustup](https://rustup.rs/))
- **cargo** (included with Rust)
- A terminal that supports 256 colors (most modern terminals do)

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

Pass `--help` for usage or `--version` for version info.

## Testing

```bash
cargo test
```

Every pull request must include tests for the code it touches. There are no exceptions to this rule. Tests live in the `tests/` directory as integration tests.

## Code Structure

See [ARCHITECTURE.md](ARCHITECTURE.md) for a full breakdown of the module tree, runtime contracts, data flows, and extension points.

At a glance:

```
src/
  main.rs          # App loop, terminal setup, render dispatch
  lib.rs           # Library crate root (re-exports all modules)
  types.rs         # Shared ID newtypes and enums
  combat/          # 4-slot party battle engine
  content/         # Authored chapter content (scenes, encounters)
  scene/           # Scene schema and runner
  state/           # Game state, persistence, progression
  pressure/        # Nonstandard encounter types (escort, crowd, reckoning)
  ui/              # TUI presentation layer (screens, widgets, input)
  dev/             # Test infrastructure (quickstart, fixtures, event log)
tests/             # Integration tests (one file per chapter + system tests)
```

## File Ownership Domains

This project uses a swarm protocol where parallel contributors have exclusive file ownership to avoid merge conflicts. The domains are:

| Domain | Owns | Description |
|--------|------|-------------|
| **Engine** | `src/combat/`, `src/scene/`, `src/pressure/` | Combat engine, scene runner, pressure encounters |
| **Content** | `src/content/` | Authored chapter content (scenes, encounters, dialogue) |
| **State** | `src/state/`, `src/types.rs` | Game state, persistence, progression, shared types |
| **Tests** | `tests/` | All integration tests |
| **CI/Docs** | `.github/`, `*.md` (root), `Cargo.toml`, `site/`, `.gitignore` | CI workflows, documentation, packaging |

If you are working in a swarm, only edit files in your assigned domain. If you need changes in another domain, coordinate with that domain's owner.

## Pull Request Process

1. Create a feature branch from `main`.
2. Make your changes in your owned files only.
3. Run `cargo fmt` to format your code.
4. Run `cargo test` to verify all tests pass.
5. Run `cargo clippy` to check for common issues.
6. Write or update tests for every code change.
7. Push your branch and open a pull request against `main`.
8. Describe what changed and why in the PR description.

## Commit Message Conventions

- Use the imperative mood: "Add escort pressure", not "Added escort pressure"
- Keep the first line under 72 characters
- Reference the domain or chapter when relevant: "Add Ch7 iron ledger encounters"
- One logical change per commit

Examples:
```
Add standoff posture selection to combat UI
Fix nerve calculation overflow in multi-phase encounters
Wire relay branch gating to evidence collection
```

## Code Style

- Follow existing patterns in the codebase. The project favors explicit, readable Rust over clever abstractions.
- Run `cargo fmt` before every commit.
- All public types and modules have doc comments (`//!` for modules, `///` for items).
- ID types use the `id_type!` macro defined in `types.rs` -- do not hand-roll ID wrappers.
- Scene and encounter definitions use builder functions (see `src/content/builders.rs`).
- Conditions and state effects are data, not code -- they are enum variants in `scene::types`, evaluated by the scene runner and state store.

## Adding Content

If you are contributing new story content (chapters, scenes, encounters), see the "Extension Points" section in [ARCHITECTURE.md](ARCHITECTURE.md) for step-by-step guides on:

- Adding a new chapter
- Adding a new encounter
- Adding a new skill

## License

By contributing, you agree that your contributions will be licensed under the MIT license (see [LICENSE](LICENSE)).
