---
title: Beginner's Guide
description: How to install, run, and start playing Saint's Mile
sidebar:
  order: 99
---

Everything you need to get Saint's Mile running and understand what you are walking into.

## 1. Installation

Saint's Mile is a Rust terminal application. You need a working Rust toolchain (1.80 or newer) and any terminal with 256-color support.

**Install Rust** (if you do not have it):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Clone and build the game:**

```bash
git clone https://github.com/mcp-tool-shop-org/saints-mile.git
cd saints-mile
cargo build --release
```

The release binary lands at `target/release/saints-mile` (or `saints-mile.exe` on Windows).

## 2. Running the Game

```bash
# Run directly through cargo
cargo run --release

# Or run the built binary
./target/release/saints-mile

# CLI flags
saints-mile --version   # Print version and exit
saints-mile --help      # Show usage help
```

The game runs entirely inside your terminal. No browser, no GUI window, no internet connection. It renders through ratatui and crossterm, so any modern terminal (Windows Terminal, iTerm2, Alacritty, kitty, or a basic xterm with 256-color) works.

## 3. Controls

All controls are keyboard-driven. There is no mouse support.

### Universal

| Key | Action |
|-----|--------|
| `Ctrl+Q` | Quit the game from any screen |
| `Ctrl+S` | Quick save (during scenes only) |

### Title Screen

| Key | Action |
|-----|--------|
| `N` | New game |
| `L` | Load a saved game |
| `Q` / `Esc` | Quit |

### Scenes (Dialogue & Exploration)

| Key | Action |
|-----|--------|
| `Enter` / `Space` | Advance text or confirm a choice |
| `Up` / `K` | Move choice cursor up |
| `Down` / `J` | Move choice cursor down |
| `Esc` | Return to title screen |

Text reveals letter by letter. Press Enter or Space to skip the animation and show the full line immediately. Once all text is revealed, the same keys confirm your current choice or advance to the next beat.

### Standoff Phase

| Key | Action |
|-----|--------|
| `Up` / `K` | Cycle posture up |
| `Down` / `J` | Cycle posture down |
| `Tab` | Cycle focus target forward |
| `Shift+Tab` | Cycle focus target backward |
| `Enter` | Confirm posture and begin combat |
| `Esc` | Return to title screen |

### Combat

| Key | Action |
|-----|--------|
| `Up` / `K` | Cycle action up |
| `Down` / `J` | Cycle action down |
| `Tab` | Cycle target forward |
| `Shift+Tab` | Cycle target backward |
| `Enter` | Confirm action |
| `Esc` | Return to title screen |

### Save/Load

| Key | Action |
|-----|--------|
| `Up` / `K` | Move slot cursor up |
| `Down` / `J` | Move slot cursor down |
| `Enter` | Confirm save or load for selected slot |
| `Esc` | Return to title screen |

There are three save slots. Saves are stored in RON format in a `saves/` directory next to the binary.

## 4. Core Concepts

Saint's Mile is a turn-based party RPG set in a frontier territory. Here are the systems you will encounter:

**Standoffs** open every significant human-versus-human fight. You choose a posture (Early Draw, Steady Hand, or Bait) and a focus target. Your posture affects turn order, opening nerve damage, and first-shot accuracy. Not every fight has a standoff -- ambushes and animal encounters skip it.

**Nerve** is a second health bar. When nerve breaks, a character panics. Enemies can break too. Managing nerve is as important as managing HP.

**Ammo** is finite per encounter. You cannot spam your strongest attack forever. The sawbones and support roles matter because they let the damage dealers stretch their ammo further.

**Wounds** persist between encounters. A wound picked up in one fight carries penalties into the next. Dr. Ada Mercer (the sawbones) is load-bearing, not optional.

**Reputation** is tracked as a web across multiple axes (town/law, railroad, rancher), not as a single score. Your standing with one faction does not neatly translate to another.

**Memory objects** are items and moments the game remembers across chapters. A biscuit cloth, a flask, a wanted poster -- these echo later in dialogue and choices.

## 5. Your First Hour

The game opens with the **Prologue at Morrow Crossing**. You are Galen Rook at age 34 -- already wanted, already known. This is a flash-forward preview: you will taste standoff combat, trail resources, a campfire choice, and a town that remembers what you did not do.

After the prologue, the game jumps back to **Chapter 1: Cedar Wake** where Galen is 19. This is the real beginning. Take your time in Cedar Wake. Talk to Molly Breck at the boarding house. Visit the shooting post to learn Steady Aim. Do the jobs Voss assigns. The town is worth caring about before the story asks you to.

The first combat encounters during Cedar Wake (horse thief, barroom scuffle, bandit camp) teach you the systems one at a time. By the bandit camp, you are using the full combat system with an NPC ally.

Then comes **Bitter Cut**. The same skills, the same mechanics, completely different meaning. That shift is the thesis of the entire game.

## 6. Tips for New Players

- **Save often.** Ctrl+S during any scene. There are three save slots -- use them before branching choices.
- **Do not neglect the sawbones.** Wounds carry over. If you leave Ada on the bench, your party degrades over time.
- **Read the standoff.** Early Draw is not always best. Bait is powerful against nervous enemies. Steady Hand is the safe default when you are unsure.
- **Ammo management matters.** Check your ammo count before committing to expensive skills. A gunhand with no bullets is a spectator.
- **Pay attention to NPC dialogue.** The game does not repeat critical information. If Old Cask Fen tells you something about paper and blood, that matters later.
- **Choices have real consequences.** The triage choice at the Saint's Mile Relay (save Tom, save Nella, or save the papers) creates permanent divergence. There is no way to get all three.

## 7. FAQ

**Q: Does the game autosave?**
No. Save manually with Ctrl+S or through the save/load screen. The game stores saves as `.ron` files in the `saves/` directory.

**Q: Can I change the text speed?**
Text reveals at a fixed rate. Press Enter or Space to skip the animation and show the full line immediately.

**Q: How long is the game?**
The full campaign spans a prologue and 15 chapters across four life phases (youth, young man, adult years, older years). A complete playthrough takes many hours. The opening arc alone (Prologue through Chapter 2) runs approximately five hours.

**Q: Is there combat difficulty?**
There is no difficulty selector. Combat is balanced around the assumption that you use the full party and manage resources across encounters. If a fight feels impossible, check your party composition and wound state.

**Q: What are the four age phases?**
Youth (age 19), Young Man (age 24), Adult (30s--40s), and Older (around 50). Galen's command menu changes with age -- he starts fast and eager, and becomes deliberate and commanding. The same skills feel different as years pass.

**Q: Does the game require internet?**
No. Saint's Mile is fully offline. It does not connect to the internet, collect telemetry, or access files outside its own save directory.

**Q: What terminal do I need?**
Any terminal with 256-color support. Windows Terminal, iTerm2, Alacritty, kitty, and most modern Linux terminals work out of the box.
