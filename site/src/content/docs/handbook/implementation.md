---
title: Implementation Guide
description: Build milestones, success tests, and what to build first
sidebar:
  order: 7
---

## Build Order

The opening arc was the first implementation target. All three milestones below are now complete, along with the full campaign (Chapters 3--15).

### Milestone A — Morrow Crossing Runtime

**What exists after this milestone:**
- Scene runner (dialogue, choices, conditions, state effects)
- Basic state system (flags, reputation axes, party state)
- 4-slot battle architecture with 2 characters active
- Standoff opener as encounter pre-phase
- Turn-order system with nerve, ammo, wounds, cover
- Loaded Deck duo tech functional
- Prologue Beat 5 choice with consequence return
- Trail resource management

**Success test:** The prologue plays. The standoff feels distinct. The choice at the campfire is hard. The return to town is different based on the choice.

### Milestone B — Cedar Wake / Bitter Cut

**What exists after this milestone:**
- Town scenes with NPCs (Molly, Voss, Cal, Renata, Declan)
- Shooting post (Steady Aim unlock through play)
- Trail Eye as exploration mechanic
- 3–4 combat encounters with escalating complexity
- NPC combatants (deputies as uncontrollable allies)
- Obedience tracking
- Bitter Cut moral-target combat
- Combat participation tracking (pulled punches flag)
- Chapter transition with time skip

**Success test:** Players love Cedar Wake. Players feel sick at Bitter Cut. The bandit camp is fun. Steady Aim feels earned. Molly is remembered.

### Milestone C — Convoy / Relay

**What exists after this milestone:**
- Convoy as moving world (formation, day/night, NPC interactions)
- Eli as NPC combatant (uncontrollable, behavior-driven)
- Escort pressure (water cart protection)
- Multi-phase combat (relay 3-phase fight)
- Skill unlocks through narrative gates
- Relay rescue branches with full state divergence
- Witness and evidence state tracking
- Memory objects functional
- Chapter transition into adult timeline

**Success test:** Players argue about Eli after the relay. The triage choice is genuinely hard. Dead Drop feels like a scar. The poster's birth feels earned. Players want to keep going.

## Deferred Systems

The following systems remain out of scope or are schemaed but not fully wired:

- Procgen
- World map sprawl
- Full inventory economy
- Polished UI chrome beyond the current spare typographic style

## Stack

- **Runtime:** Rust, ratatui 0.30, crossterm 0.28
- **Data:** RON for content and saves, serde for serialization
- **Error handling:** anyhow + thiserror
- **Observability:** tracing + tracing-subscriber
- **Architecture:** Library crate (`saints_mile`) + binary entry point, TUI layer, data-driven scenes from RON files

## Where to Start Reading the Code

If you are new to the codebase and want to understand how the game works, this is the recommended reading order:

1. **`src/main.rs`** — The entry point. Sets up the terminal, creates the `App`, and runs the 50ms event loop. Start here to see how input, tick, and render connect.

2. **`src/ui/mod.rs`** — The `App` struct and `AppScreen` enum. This is the central state machine that decides what the player sees: title screen, scene, combat, save/load, etc. Understanding `AppScreen` is understanding the game's flow.

3. **`src/scene/runner.rs`** — The `SceneRunner` executes authored scenes against live game state. This is where dialogue, choices, conditions, and state effects meet. Most of the game's narrative gameplay flows through here.

4. **`src/combat/engine.rs`** — The `EncounterState` machine handles standoff and combat phases. Read this to understand how the battle system works: turn order, skills, duo techs, wounds, and ammo.

5. **`src/state/types.rs`** — The `GameState` struct is the complete memory of a playthrough. Reputation, evidence, witnesses, party state, flags, and chapter progression all live here.

6. **`src/content/prologue.rs`** — A concrete content file. After reading the runtime systems above, look at how authored content (scenes, encounters, choices) is structured. The prologue is the simplest chapter.

7. **`ARCHITECTURE.md`** — The full module tree and runtime contracts. Use this as a reference once you have the mental model from the files above.

The key insight: the game is **data-driven**. Content authors compose scenes and encounters from `Condition` and `StateEffect` enum variants. The runtime systems (`SceneRunner`, `EncounterState`, `StateStore`) interpret them. Understanding that split is understanding the architecture.

## The First Playable's Success Test

The first real build is successful if players:
- Love Cedar Wake
- Feel sick at Bitter Cut
- Argue about Eli after the relay
- Understand why the poster exists
- Want to keep going because the world feels alive and wrong
