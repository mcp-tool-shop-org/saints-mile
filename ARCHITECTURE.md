# Architecture

Saint's Mile is a frontier JRPG built as a Rust TUI application. It uses Ratatui for rendering, Crossterm for terminal I/O, and RON for save file serialization. The game runs a single-threaded event loop at 50ms tick rate.

## Module Tree

```
src/
├── main.rs                    # Terminal setup, app loop, render dispatch
├── lib.rs                     # Library crate root (re-exports all modules)
├── types.rs                   # Shared ID newtypes and enums (AgePhase, FlagValue, etc.)
│
├── scene/                     # Scene system
│   ├── mod.rs                 #   Module root
│   ├── types.rs               #   Scene, Choice, Condition, StateEffect, SceneTransition
│   └── runner.rs              #   SceneRunner — executes scenes against live state
│
├── combat/                    # Combat system (4-slot party battle)
│   ├── mod.rs                 #   Module root
│   ├── types.rs               #   Encounter, Skill, DuoTech, CombatantState, Wound
│   ├── engine.rs              #   EncounterState machine, SkillRegistry, DuoTechRegistry
│   ├── convoy.rs              #   Convoy escort combat variant
│   ├── crowd.rs               #   Crowd encounter logic
│   ├── environment.rs         #   Terrain and environmental effects
│   ├── party_defs.rs          #   Party member stat definitions
│   ├── reckoning.rs           #   Public reckoning encounter logic
│   ├── split_party.rs         #   Split-party encounter variant
│   └── wounds.rs              #   Wound persistence and treatment
│
├── state/                     # Game state and persistence
│   ├── mod.rs                 #   Module root
│   ├── types.rs               #   GameState, PartyState, ReputationWeb, WitnessState
│   ├── store.rs               #   StateStore — RON save/load with versioned envelope
│   ├── argument.rs            #   Argument/confrontation state tracking
│   ├── evidence.rs            #   Evidence collection and integrity
│   ├── investigation.rs       #   Multi-domain investigation assembly
│   ├── history.rs             #   Play history tracking
│   ├── reassembly.rs          #   Late-game state reassembly
│   ├── ending.rs              #   Ending determination logic
│   └── progression.rs         #   Chapter progression and age transitions
│
├── pressure/                  # Nonstandard encounter types
│   ├── mod.rs                 #   Module root
│   └── types.rs               #   PressureEncounter, PressureType (Escort, Crowd, Reckoning, Transmission)
│
├── content/                   # Authored game content (16 chapters)
│   ├── mod.rs                 #   Scene/encounter dispatcher + chapter_entry_scene()
│   ├── builders.rs            #   Builder helpers for scene and encounter construction
│   ├── prologue.rs            #   Ch0: Prologue
│   ├── cedar_wake.rs          #   Ch1: Cedar Wake
│   ├── saints_mile_convoy.rs  #   Ch2: Saint's Mile Convoy
│   ├── black_willow.rs        #   Ch3: Black Willow
│   ├── ropehouse_blood.rs     #   Ch4: Ropehouse Blood
│   ├── dust_revival.rs        #   Ch5: Dust Revival
│   ├── fuse_country.rs        #   Ch6: Fuse Country
│   ├── iron_ledger.rs         #   Ch7: Iron Ledger
│   ├── burned_mission.rs      #   Ch8: Burned Mission
│   ├── long_wire.rs           #   Ch9: Long Wire
│   ├── deadwater_trial.rs     #   Ch10: Deadwater Trial
│   ├── breakwater_junction.rs #   Ch11: Breakwater Junction
│   ├── names_in_dust.rs       #   Ch12: Names in Dust
│   ├── fifteen_years_gone.rs  #   Ch13: Fifteen Years Gone
│   ├── old_friends.rs         #   Ch14: Old Friends
│   └── saints_mile_again.rs   #   Ch15: Saint's Mile Again
│
├── ui/                        # TUI presentation layer
│   ├── mod.rs                 #   App struct, AppScreen enum, app logic
│   ├── input.rs               #   Input event handling (keyboard → InputResult)
│   ├── theme.rs               #   Color palette and style constants
│   ├── text_reveal.rs         #   Typewriter text reveal effect
│   ├── screens/               #   Screen renderers
│   │   ├── mod.rs             #     Screen module root
│   │   ├── title.rs           #     Title screen
│   │   ├── scene.rs           #     Scene/dialogue screen
│   │   ├── standoff.rs        #     Standoff pre-combat screen
│   │   ├── combat.rs          #     Combat screen
│   │   ├── crowd.rs           #     Crowd encounter screen
│   │   ├── escort.rs          #     Escort encounter screen
│   │   ├── reckoning.rs       #     Public reckoning screen
│   │   ├── save_load.rs       #     Save/load screen
│   │   └── transmission.rs    #     Transmission race screen
│   └── widgets/               #   Reusable UI components
│       ├── mod.rs             #     Widget module root
│       ├── choice_menu.rs     #     Player choice menu
│       ├── dialogue.rs        #     Dialogue box
│       ├── memory_echo.rs     #     Memory object echo display
│       └── stat_bar.rs        #     HP/nerve/ammo bar
│
└── dev/                       # Developer/test infrastructure
    ├── mod.rs                 #   Module root
    ├── quickstart.rs          #   JumpPoint system — skip to any chapter for testing
    ├── event_log.rs           #   Event logging for test verification
    └── fixtures.rs            #   Test fixture builders
```

## Runtime Contracts

The game is organized around four runtime contracts. Each is a self-contained system with defined inputs, outputs, and state interactions.

### 1. Scene Contract

**Owner:** `scene/`

A Scene is a unit of narrative gameplay: dialogue lines, player choices, conditions, and state effects.

- **Input:** A `Scene` struct (authored in `content/`) + current `GameState`
- **Process:** `SceneRunner` filters lines by conditions, presents available choices, applies state effects
- **Output:** `SceneResult` containing displayed lines, the chosen action, and a `SceneTransition`
- **Transitions:** `SceneTransition::Scene(id)` (same chapter), `SceneTransition::Beat(id)` (chapter advance), `SceneTransition::Combat(id)` (enter encounter), `SceneTransition::End` (game over or chapter close)

Conditions gate line visibility and choice availability. State effects fire on scene entry and choice confirmation. Lines carry speaker, emotion, and pacing metadata.

### 2. Combat Contract

**Owner:** `combat/`

Combat is a 4-slot party battle with a standoff pre-phase. Even with 2 active characters, the engine runs full party logic.

- **Input:** An `Encounter` definition (phases, enemies, standoff config, terrain, objectives)
- **Phases:** Standoff → Combat → Resolved
- **Standoff:** Player picks a posture (EarlyDraw / SteadyHand / Bait) and focus target. Outcome sets initiative mods, nerve damage, and broken enemies.
- **Combat:** Turn-based with skills, duo techs, items, cover, wounds, ammo, and nerve. Each `CombatantState` tracks HP, nerve, ammo, wounds, position, and available skills.
- **Output:** `EncounterResult` (Victory / Defeat / Fled / ObjectiveComplete) + `OutcomeEffect` list
- **Registries:** `SkillRegistry` and `DuoTechRegistry` map IDs to definitions at runtime. Skills have `AgeVariant`s that change stats per life phase.

### 3. State Contract

**Owner:** `state/`

GameState is the complete memory of a playthrough. It tracks chapter, beat, age phase, reputation, evidence, witnesses, party, flags, memory objects, resources, and investigation state.

- **Persistence:** RON-backed save/load through `StateStore`. Saves use a versioned `SaveEnvelope` with metadata.
- **Integrity rule:** The save/load layer never "repairs" morally ambiguous state into certainty. If a witness is compromised or evidence is partial, that exact state is preserved.
- **State effects:** All state mutations flow through `StateEffect` enum variants (SetFlag, AdjustReputation, AddPartyMember, UnlockSkill, AddEvidence, etc.). The store applies them atomically.
- **Progression:** `progression.rs` handles chapter transitions and age phase advances.
- **Endings:** `ending.rs` determines the ending based on accumulated state.

### 4. Pressure Contract

**Owner:** `pressure/`

Pressure encounters are nonstandard combat-adjacent gameplay: escort missions, crowd control, public reckonings, witness protection, and transmission races.

- **Schema:** `PressureEncounter` with typed `PressureType` variants, pressure bars, party actions, and success/failure thresholds
- **Types:** Escort (cargo integrity), Crowd (collective nerve, ringleaders), PublicReckoning (5 simultaneous bars), WitnessProtection (integrity drain), TransmissionRace (channels + timer)
- **Output:** State effects on success or failure, determined by `PressureCondition` thresholds

## Key Data Flows

### Scene Runner Flow

```
content::get_scene(chapter, id)
  → Scene struct
  → SceneRunner::prepare(scene, &state)
    → filter lines by conditions
    → evaluate choice availability
    → PreparedScene (displayable)
  → Player makes choice
  → SceneRunner::execute_choice(choice, &mut state)
    → apply StateEffects
    → return SceneTransition
  → App follows transition (next scene, combat, or chapter advance)
```

### Combat Engine Flow

```
content::get_encounter(chapter, id)
  → Encounter struct
  → EncounterState::new(encounter)
    → Standoff phase
      → Player selects posture + focus
      → StandoffResult (initiative, nerve damage, broken enemies)
    → Combat phase
      → Turn queue by speed priority
      → Player selects action + target
      → engine resolves: accuracy, damage, nerve, ammo, wounds, cover
      → Check objectives each turn
    → Resolved phase
      → EncounterResult (Victory/Defeat/Fled/ObjectiveComplete)
      → Apply OutcomeEffects to GameState
  → Return to scene via post_combat_scene
```

### State Persistence Flow

```
App::quick_save()
  → StateStore::save(slot_name)
    → SaveEnvelope::new(state, label)
    → ron::to_string() → write to saves/{slot}.ron

App::load_game(slot)
  → StateStore::load(path)
    → ron::from_str::<SaveEnvelope>()
    → version check
    → restore GameState
  → App::load_scene(beat) → resume play
```

## Key Entry Points

| Entry Point | Location | Purpose |
|-------------|----------|---------|
| App loop | `main.rs::run_loop()` | Poll events at 50ms, handle input, tick, render |
| Render dispatch | `main.rs::render()` | Match on `AppScreen` variant, call screen renderer |
| Input dispatch | `ui/input.rs::handle_event()` | Map keyboard events to `InputResult` per screen |
| Screen management | `ui/mod.rs::App` | Holds `AppScreen` enum, all UI state, game methods |
| Content dispatch | `content/mod.rs::get_scene()` | Route chapter+id to the correct content module |
| Encounter dispatch | `content/mod.rs::get_encounter()` | Route chapter+id to the correct encounter definition |

## Extension Points

### How to Add a New Chapter

1. Create `src/content/new_chapter.rs` with two public functions:
   - `pub fn get_scene(id: &str) -> Option<Scene>` — match scene IDs to Scene structs
   - `pub fn get_encounter(id: &str) -> Option<Encounter>` — match encounter IDs (return `None` for each if no encounters)
2. Add `pub mod new_chapter;` to `src/content/mod.rs`
3. Add a match arm for `"new_chapter"` in `content::get_scene()`
4. Add a match arm for `"new_chapter"` in `content::get_encounter()`
5. Add an entry scene mapping in `content::chapter_entry_scene()`
6. Create `tests/new_chapter.rs` with scene and encounter tests

### How to Add a New Encounter

1. In your chapter's content file, define the encounter using the builder pattern:
   - Create an `Encounter` with phases, enemies, standoff config, terrain, and objectives
   - Each `CombatPhase` has enemy templates, optional NPC allies, and entry conditions
   - Add `OutcomeEffect`s for victory/defeat consequences
2. Register the encounter in your chapter's `get_encounter()` match
3. Reference it from a scene choice using `SceneTransition::Combat(EncounterId::new("encounter_id"))`
4. Write tests that verify the encounter loads and its objectives are achievable

### How to Add a New Skill

1. Create a `Skill` struct with:
   - Unique `SkillId`
   - A `SkillLine` variant (Deadeye, Trailcraft, Command, Hustle, Deceit, Loyalty, Triage, Tonics, Diagnosis, Lariat, Guard, Break, Hymn, Witness, Intercession, Charges, Smoke, Collapse)
   - An `UnlockCondition` (StartOfPhase, StoryEvent, TurningPoint, Bond, Ordeal)
   - `AgeVariant`s for each applicable `AgePhase` (Youth, YoungMan, Adult, Older) with phase-specific accuracy, damage, speed, and nerve stats
   - A `SkillCost` (ammo, nerve, or both)
2. Register the skill in the `SkillRegistry` (see `combat/engine.rs`)
3. Add the `SkillId` to the relevant character's `available_skills` in their party definition
4. If the skill has age variants, verify each variant is present and stats are reasonable
5. Write tests that verify the skill resolves correctly at each age phase

## Design Principles

- **The command menu carries biography.** Skills change name, description, and stats as the character ages. The menu is not just UI -- it reflects who the character has become.
- **Four age phases shape everything.** Youth, YoungMan, Adult, and Older are not cosmetic. They change available skills, stat curves, and narrative tone.
- **Conditions and effects are data.** All game logic flows through `Condition` and `StateEffect` enums. Content authors compose behavior by combining these variants, not by writing imperative code.
- **State integrity over convenience.** The save system preserves moral ambiguity. Partial evidence, compromised witnesses, and unresolved flags are never "fixed" by serialization.
- **Standoff is not decoration.** The pre-combat phase produces real mechanical consequences: initiative, nerve damage, broken enemies. Posture choice matters.
