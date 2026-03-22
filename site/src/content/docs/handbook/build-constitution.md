---
title: Build Constitution
description: Anti-drift laws, runtime contracts, and what the code is not allowed to become
sidebar:
  order: 6
---

## What the Code Is NOT Allowed to Become

1. **Not a duel sim.** The standoff is an opener, not the whole combat.
2. **Not interactive fiction with battles.** Combat must be mechanically deep, not decorative.
3. **Not a generic retro RPG.** The western layer must change mechanics, not just flavor text.
4. **Not a survival slog.** Resources create tension, not tedium.
5. **Not a lore-heavy mystery game with thin mechanics.** Systems must carry the narrative weight.
6. **Not a two-character engine that later becomes a party game.** Build 4-slot party architecture from day one.

## Runtime Contracts

Four contracts govern the implementation:

### Scene Contract
How towns, campfires, investigations, and consequence scenes are authored. Every scene has: speaker, text, choices, conditions, state effects, pacing tags, and memory callbacks. Memory objects (biscuit cloth, flask, poster) must be explicitly tagged so they echo later.

### Combat Contract
Built as a 4-slot party battle system from day one. Key requirements:
- Nerve as real second health bar
- Ammo finite per encounter
- Wounds persist between encounters
- Cover is positional and destructible (schemaed from day one)
- Standoff as pre-phase feeding into turn order
- Duo techs in schema even if only one is active early
- NPC combatants as a supported type
- Age variants as first-class data

### State Contract
What the game remembers. Key requirements:
- Relay branch (Tom/Nella/papers) as first-class state axis
- Reputation as a web, not a score
- Evidence with integrity tracking
- Witness states (alive, location, integrity, testified)
- Memory objects as explicit state items
- Hand injury as first-class state field
- Chapter flags supporting "same mechanics, different meaning"

### Pressure Contract
Generalized system for nonstandard encounters. Schema all types now, implement as needed:
- Escort pressure (build for opening arc)
- Crowd pressure (Ch5)
- Public Reckoning pressure (Ch10)
- Transmission race (Ch9)

## The One Thing to Protect Hardest

**The player's command menu must be part of the narrative.**

- Young Galen feels different from adult Galen in the menu
- Eli's grayed-out Loyalty line exists before it opens
- Dead Drop appears as a scar, not just a skill reward
- Age variants are first-class schema, not afterthoughts
- Skill unlock conditions are narrative, not numeric

If the menu cannot carry biography, the game loses one of its best ideas.
