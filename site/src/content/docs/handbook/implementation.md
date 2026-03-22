---
title: Implementation Guide
description: Build milestones, success tests, and what to build first
sidebar:
  order: 7
---

## Build Order

The opening arc only. Not the full campaign.

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

## What Not to Build Yet

- Procgen
- World map sprawl
- Full inventory economy
- Chapters 3–15 content
- Polished UI chrome
- Optional systems that don't improve the opening arc
- Crowd battles, split-party operations, Public Reckoning Pressure (schema only)
- Environmental destruction (schema only)

## Stack

- **Runtime:** Rust, ratatui 0.30, crossterm 0.28
- **Data:** RON for content and saves, serde for serialization
- **Error handling:** anyhow + thiserror
- **Observability:** tracing + tracing-subscriber
- **Architecture:** Core logic crate (later), TUI layer, data-driven scenes from RON files

## The First Playable's Success Test

The first real build is successful if players:
- Love Cedar Wake
- Feel sick at Bitter Cut
- Argue about Eli after the relay
- Understand why the poster exists
- Want to keep going because the world feels alive and wrong
