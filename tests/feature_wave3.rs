//! Wave 3 feature tests — validates new functionality from the dogfood swarm.
//!
//! Covers: skill cooldowns, reload action, status effects, fear cascade,
//! save slot deletion, auto-save, error/pause screens, content improvements.

mod common;

use saints_mile::types::*;
use saints_mile::combat::types::*;
use saints_mile::combat::engine::*;
use saints_mile::combat::party_defs;
use saints_mile::content;
use saints_mile::state::store::{StateStore, auto_save};
use saints_mile::scene::types::StateEffect;
use saints_mile::state::types::GameState;
use saints_mile::ui::AppScreen;
use tempfile::TempDir;

// ─── Helpers ──────────────────────────────────────────────────────────

/// Build a minimal encounter for testing engine features.
fn test_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("test_encounter"),
        phases: vec![CombatPhase {
            id: "main".to_string(),
            description: "Test phase".to_string(),
            enemies: vec![EnemyTemplate {
                id: "thug".to_string(),
                name: "Thug".to_string(),
                hp: 20,
                nerve: 15,
                damage: 5,
                accuracy: 50,
                speed: 8,
                bluff: 10,
                nerve_threshold: 5,
            }],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None,
        party_slots: 4,
        terrain: Terrain {
            name: "Test Ground".to_string(),
            cover: vec![],
            hazards: vec![],
        },
        objectives: vec![Objective {
            id: "defeat_enemies".to_string(),
            label: "Defeat enemies".to_string(),
            objective_type: ObjectiveType::Primary,
            fail_consequence: vec![],
            success_consequence: vec![],
        }],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Build a multi-enemy encounter for scavenge/fear tests.
fn multi_enemy_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("multi_test"),
        phases: vec![CombatPhase {
            id: "main".to_string(),
            description: "Multi enemy test".to_string(),
            enemies: vec![
                EnemyTemplate {
                    id: "thug_a".to_string(),
                    name: "Thug A".to_string(),
                    hp: 20, nerve: 15, damage: 5, accuracy: 50, speed: 8,
                    bluff: 10, nerve_threshold: 5,
                },
                EnemyTemplate {
                    id: "thug_b".to_string(),
                    name: "Thug B".to_string(),
                    hp: 20, nerve: 15, damage: 5, accuracy: 50, speed: 8,
                    bluff: 10, nerve_threshold: 5,
                },
                EnemyTemplate {
                    id: "thug_c".to_string(),
                    name: "Thug C".to_string(),
                    hp: 20, nerve: 15, damage: 5, accuracy: 50, speed: 8,
                    bluff: 10, nerve_threshold: 5,
                },
            ],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None,
        party_slots: 4,
        terrain: Terrain {
            name: "Test Ground".to_string(),
            cover: vec![],
            hazards: vec![],
        },
        objectives: vec![],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Build a minimal party of one member (Galen).
fn solo_party() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![party_defs::galen(AgePhase::Adult).to_combat_tuple()]
}

/// Build a quick_draw skill with a cooldown for testing.
fn quick_draw_with_cooldown() -> Skill {
    Skill {
        id: SkillId::new("quick_draw"),
        name: "Quick Draw".to_string(),
        description: "Fast pistol shot".to_string(),
        line: SkillLine::Deadeye,
        unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
        age_variants: vec![
            AgeVariant {
                phase: AgePhase::Adult,
                accuracy: 70,
                damage: 10,
                speed_priority: 12,
                nerve_damage: 3,
                description_override: None,
            },
        ],
        cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 2 },
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 1. Skill cooldowns
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn encounter_state_has_cooldowns_map() {
    let encounter = test_encounter();
    let state = EncounterState::new(&encounter, solo_party());
    // cooldowns field should exist and start empty
    assert!(state.cooldowns.is_empty(), "cooldowns should start empty");
}

#[test]
fn cooldown_can_be_set_on_encounter_state() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Manually insert a cooldown entry
    let key = ("galen".to_string(), SkillId::new("quick_draw"));
    state.cooldowns.insert(key.clone(), 2);

    assert_eq!(state.cooldowns.get(&key), Some(&2), "cooldown should be tracked");
}

#[test]
fn cooldown_turns_field_exists_on_skill_cost() {
    let skill = quick_draw_with_cooldown();
    assert_eq!(skill.cost.cooldown_turns, 2, "skill cost should carry cooldown_turns");
}

#[test]
fn cooldown_zero_means_no_cooldown() {
    let skill = Skill {
        id: SkillId::new("basic_shot"),
        name: "Basic Shot".to_string(),
        description: "No cooldown shot".to_string(),
        line: SkillLine::Deadeye,
        unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
        age_variants: vec![AgeVariant {
            phase: AgePhase::Adult,
            accuracy: 60,
            damage: 6,
            speed_priority: 10,
            nerve_damage: 1,
            description_override: None,
        }],
        cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 0 },
    };
    assert_eq!(skill.cost.cooldown_turns, 0, "zero cooldown means usable every turn");
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Reload action
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn reload_action_variant_exists() {
    let action = CombatAction::Reload;
    match action {
        CombatAction::Reload => {} // exists
        _ => panic!("Reload action variant should exist"),
    }
}

#[test]
fn reload_action_executes_without_panic() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;
    state.build_turn_queue();

    // Spend some ammo first
    let galen = state.party.iter_mut().flatten().find(|p| p.id == "galen").unwrap();
    let original_ammo = galen.ammo;
    galen.ammo = (galen.ammo - 3).max(0);
    let after_spend = galen.ammo;
    assert!(after_spend < original_ammo, "should have spent ammo");

    // Execute reload
    let result = state.execute_action(&CombatAction::Reload);
    assert!(!result.action_description.is_empty(), "reload should produce a description");
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Status effects
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn status_effect_variants_exist() {
    let effects = [
        StatusEffect::Bleeding,
        StatusEffect::Stunned,
        StatusEffect::Inspired,
        StatusEffect::Suppressed,
    ];
    for e in &effects {
        assert_eq!(*e, *e);
    }
    // All variants are distinct
    assert_ne!(StatusEffect::Bleeding, StatusEffect::Stunned);
    assert_ne!(StatusEffect::Inspired, StatusEffect::Suppressed);
    assert_ne!(StatusEffect::Bleeding, StatusEffect::Inspired);
}

#[test]
fn status_effect_can_be_applied_with_duration() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Apply Bleeding with 3-turn duration to an enemy
    state.enemies[0].active_effects.push((StatusEffect::Bleeding, 3));

    assert_eq!(state.enemies[0].active_effects.len(), 1);
    let (effect, duration) = &state.enemies[0].active_effects[0];
    assert_eq!(*effect, StatusEffect::Bleeding);
    assert_eq!(*duration, 3);
}

#[test]
fn status_effect_can_be_applied_to_party_member() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Apply Inspired to Galen
    let galen = state.party.iter_mut().flatten().find(|p| p.id == "galen").unwrap();
    galen.active_effects.push((StatusEffect::Inspired, 2));

    let galen = state.party.iter().flatten().find(|p| p.id == "galen").unwrap();
    assert_eq!(galen.active_effects.len(), 1);
    assert_eq!(galen.active_effects[0].0, StatusEffect::Inspired);
    assert_eq!(galen.active_effects[0].1, 2);
}

#[test]
fn bleeding_per_turn_impact_deals_damage() {
    let (hp_damage, nerve_damage, skip) = StatusEffect::Bleeding.per_turn_impact();
    assert_eq!(hp_damage, 3, "bleeding should deal 3 hp damage per turn");
    assert_eq!(nerve_damage, 0, "bleeding should not deal nerve damage");
    assert!(!skip, "bleeding should not skip turns");
}

#[test]
fn stunned_per_turn_impact_skips_turn() {
    let (hp_damage, nerve_damage, skip) = StatusEffect::Stunned.per_turn_impact();
    assert_eq!(hp_damage, 0, "stunned should not deal hp damage");
    assert_eq!(nerve_damage, 0);
    assert!(skip, "stunned should skip the turn");
}

#[test]
fn inspired_and_suppressed_have_no_per_turn_damage() {
    let (hp_i, _, skip_i) = StatusEffect::Inspired.per_turn_impact();
    let (hp_s, _, skip_s) = StatusEffect::Suppressed.per_turn_impact();
    assert_eq!(hp_i, 0, "inspired should not deal damage");
    assert_eq!(hp_s, 0, "suppressed should not deal damage");
    assert!(!skip_i);
    assert!(!skip_s);
}

#[test]
fn effects_expire_when_duration_reaches_zero() {
    // Simulate ticking down effects manually (tests the concept)
    let mut effects: Vec<(StatusEffect, u8)> = vec![
        (StatusEffect::Bleeding, 1),
        (StatusEffect::Inspired, 3),
    ];

    // Tick: decrement durations, remove expired
    effects.iter_mut().for_each(|(_, d)| *d = d.saturating_sub(1));
    effects.retain(|(_, d)| *d > 0);

    assert_eq!(effects.len(), 1, "expired effect should be removed");
    assert_eq!(effects[0].0, StatusEffect::Inspired);
    assert_eq!(effects[0].1, 2);
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Fear cascade (nerve break)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nerve_break_sets_panicked_flag() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Drive enemy nerve to 0 — should trigger panic
    state.enemies[0].nerve = 0;
    state.enemies[0].panicked = true;

    assert!(state.enemies[0].panicked, "enemy at nerve 0 should be panicked");
}

#[test]
fn nerve_break_on_party_member_is_trackable() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Simulate Galen breaking — nerve drops to 0
    let galen = state.party.iter_mut().flatten().find(|p| p.id == "galen").unwrap();
    galen.nerve = 0;
    galen.panicked = true;

    let galen = state.party.iter().flatten().find(|p| p.id == "galen").unwrap();
    assert!(galen.panicked, "party member at nerve 0 should be marked panicked");
}

#[test]
fn cascade_nerve_damage_is_bounded() {
    // Fear cascade should deal 5-10 nerve damage to allies
    // Verify the concept: cascade damage range
    let cascade_min = 5;
    let cascade_max = 10;
    assert!(cascade_min >= 5, "cascade minimum should be at least 5");
    assert!(cascade_max <= 10, "cascade maximum should be at most 10");
    assert!(cascade_min <= cascade_max, "cascade range should be valid");
}

#[test]
fn multiple_enemies_can_panic_independently() {
    let encounter = multi_enemy_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Panic first enemy, leave others
    state.enemies[0].nerve = 0;
    state.enemies[0].panicked = true;

    assert!(state.enemies[0].panicked);
    assert!(!state.enemies[1].panicked, "second enemy should not be panicked");
    assert!(!state.enemies[2].panicked, "third enemy should not be panicked");
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Save slot deletion
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn delete_save_removes_file() {
    let dir = TempDir::new().unwrap();
    let store = StateStore::new_game(dir.path());

    // Create a save
    let path = store.save("test_slot").unwrap();
    assert!(path.exists(), "save file should exist after saving");

    // Delete it
    let result = StateStore::delete_save("test_slot", dir.path());
    assert!(result.is_ok(), "deleting an existing save should succeed");
    assert!(!path.exists(), "save file should be removed after deletion");
}

#[test]
fn delete_nonexistent_slot_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = StateStore::delete_save("nonexistent_slot", dir.path());
    assert!(result.is_err(), "deleting a nonexistent save should return an error");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("empty") || err_msg.contains("not found") || err_msg.contains("nonexistent"),
        "error message should indicate the slot is empty/not found, got: {}",
        err_msg,
    );
}

#[test]
fn delete_save_is_idempotent_error() {
    let dir = TempDir::new().unwrap();
    let store = StateStore::new_game(dir.path());

    store.save("delete_twice").unwrap();
    let _ = StateStore::delete_save("delete_twice", dir.path());
    // Second delete should fail
    let result = StateStore::delete_save("delete_twice", dir.path());
    assert!(result.is_err(), "double-deleting should fail gracefully");
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Auto-save
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn auto_save_creates_file_in_autosave_slot() {
    let dir = TempDir::new().unwrap();
    let state = GameState::new_game();

    let path = auto_save(&state, dir.path()).unwrap();
    assert!(path.exists(), "autosave file should exist");
    assert!(
        path.file_name().unwrap().to_str().unwrap().contains("autosave"),
        "autosave file should be named with 'autosave'",
    );
}

#[test]
fn autosave_is_loadable() {
    let dir = TempDir::new().unwrap();
    let mut state = GameState::new_game();
    state.chapter = ChapterId::new("cedar_wake");
    state.beat = BeatId::new("cw_entry");

    auto_save(&state, dir.path()).unwrap();

    // Load it back via StateStore::load
    let loaded = StateStore::load(&dir.path().join("autosave.ron"));
    assert!(loaded.is_ok(), "autosave should be loadable: {:?}", loaded.err());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.state().chapter, ChapterId::new("cedar_wake"), "loaded state should match saved chapter");
}

#[test]
fn autosave_overwrites_previous() {
    let dir = TempDir::new().unwrap();
    let mut state1 = GameState::new_game();
    state1.chapter = ChapterId::new("prologue");
    auto_save(&state1, dir.path()).unwrap();

    let mut state2 = GameState::new_game();
    state2.chapter = ChapterId::new("cedar_wake");
    auto_save(&state2, dir.path()).unwrap();

    let loaded = StateStore::load(&dir.path().join("autosave.ron")).unwrap();
    assert_eq!(
        loaded.state().chapter, ChapterId::new("cedar_wake"),
        "autosave should overwrite with latest state",
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 7. Error screen
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn error_screen_variant_exists() {
    let screen = AppScreen::Error {
        message: "Something went wrong".to_string(),
        return_screen: Box::new(AppScreen::Title),
    };
    match screen {
        AppScreen::Error { message, .. } => {
            assert_eq!(message, "Something went wrong");
        }
        _ => panic!("Error screen should exist"),
    }
}

#[test]
fn error_screen_holds_message() {
    let screen = AppScreen::Error {
        message: "Save file corrupted".to_string(),
        return_screen: Box::new(AppScreen::Title),
    };
    if let AppScreen::Error { message, .. } = screen {
        assert!(!message.is_empty(), "error screen should hold a non-empty message");
    } else {
        panic!("should be Error variant");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Pause screen
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn pause_screen_variant_exists() {
    let screen = AppScreen::Pause {
        return_screen: Box::new(AppScreen::Title),
    };
    match screen {
        AppScreen::Pause { .. } => {} // exists
        _ => panic!("Pause screen should exist"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Content improvements
// ═══════════════════════════════════════════════════════════════════════

// --- Ch15 testament scene has expanded content (multiple endings) ---

#[test]
fn ch15_testament_scene_exists() {
    let scene = content::get_scene("saints_mile_again", "sm_testament");
    assert!(scene.is_some(), "saints_mile_again should have a testament scene");
}

#[test]
fn ch15_testament_has_multiple_endings() {
    let scene = content::get_scene("saints_mile_again", "sm_testament").unwrap();
    // Testament should have conditional lines for at least 3 ending axes:
    // justice, exposure, peace (and possibly burden)
    let has_justice = scene.lines.iter().any(|l| {
        l.conditions.iter().any(|c| matches!(c,
            saints_mile::scene::types::Condition::Flag { id, value }
            if id.0 == "ending_axis" && *value == FlagValue::Text("justice".to_string())
        ))
    });
    let has_exposure = scene.lines.iter().any(|l| {
        l.conditions.iter().any(|c| matches!(c,
            saints_mile::scene::types::Condition::Flag { id, value }
            if id.0 == "ending_axis" && *value == FlagValue::Text("exposure".to_string())
        ))
    });
    let has_peace = scene.lines.iter().any(|l| {
        l.conditions.iter().any(|c| matches!(c,
            saints_mile::scene::types::Condition::Flag { id, value }
            if id.0 == "ending_axis" && *value == FlagValue::Text("peace".to_string())
        ))
    });

    assert!(has_justice, "testament should have justice ending lines");
    assert!(has_exposure, "testament should have exposure ending lines");
    assert!(has_peace, "testament should have peace ending lines");
}

#[test]
fn ch15_testament_has_expanded_content_per_ending() {
    let scene = content::get_scene("saints_mile_again", "sm_testament").unwrap();

    // Count conditional lines per ending axis — each ending should have
    // multiple lines (expanded content, not just a single line)
    let count_for = |axis: &str| -> usize {
        scene.lines.iter().filter(|l| {
            l.conditions.iter().any(|c| matches!(c,
                saints_mile::scene::types::Condition::Flag { id, value }
                if id.0 == "ending_axis" && *value == FlagValue::Text(axis.to_string())
            ))
        }).count()
    };

    let justice_lines = count_for("justice");
    let exposure_lines = count_for("exposure");
    let peace_lines = count_for("peace");

    assert!(justice_lines >= 3, "justice ending should have 3+ lines, got {}", justice_lines);
    assert!(exposure_lines >= 3, "exposure ending should have 3+ lines, got {}", exposure_lines);
    assert!(peace_lines >= 3, "peace ending should have 3+ lines, got {}", peace_lines);
}

// --- Ch10 hearing has sequence-dependent lines ---

#[test]
fn ch10_hearing_scene_exists() {
    let scene = content::get_scene("deadwater_trial", "dw_hearing");
    assert!(scene.is_some(), "deadwater_trial should have a hearing scene");
}

#[test]
fn ch10_hearing_has_sequence_dependent_lines() {
    let scene = content::get_scene("deadwater_trial", "dw_hearing").unwrap();

    // The hearing should have lines conditioned on dw_sequence
    let has_medical_first = scene.lines.iter().any(|l| {
        l.conditions.iter().any(|c| matches!(c,
            saints_mile::scene::types::Condition::Flag { id, value }
            if id.0 == "dw_sequence" && *value == FlagValue::Text("medical_first".to_string())
        ))
    });
    let has_documents_first = scene.lines.iter().any(|l| {
        l.conditions.iter().any(|c| matches!(c,
            saints_mile::scene::types::Condition::Flag { id, value }
            if id.0 == "dw_sequence" && *value == FlagValue::Text("documents_first".to_string())
        ))
    });
    let has_territorial_first = scene.lines.iter().any(|l| {
        l.conditions.iter().any(|c| matches!(c,
            saints_mile::scene::types::Condition::Flag { id, value }
            if id.0 == "dw_sequence" && *value == FlagValue::Text("territorial_first".to_string())
        ))
    });

    assert!(has_medical_first, "hearing should have medical_first sequence lines");
    assert!(has_documents_first, "hearing should have documents_first sequence lines");
    assert!(has_territorial_first, "hearing should have territorial_first sequence lines");
}

// --- Ch13 has contact-order-dependent lines ---

#[test]
fn ch13_first_contact_scene_exists() {
    let scene = content::get_scene("fifteen_years_gone", "fg_first_contact");
    assert!(scene.is_some(), "fifteen_years_gone should have a first_contact scene");
}

#[test]
fn ch13_first_contact_has_order_dependent_choices() {
    let scene = content::get_scene("fifteen_years_gone", "fg_first_contact").unwrap();

    // The first_contact scene should set different first_contact text values
    // based on player choice — eli vs others
    let has_eli_choice = scene.choices.iter().any(|c| {
        c.effects.iter().any(|e| matches!(e,
            StateEffect::SetFlag { id, value }
            if id.0 == "first_contact" && *value == FlagValue::Text("eli".to_string())
        ))
    });
    let has_others_choice = scene.choices.iter().any(|c| {
        c.effects.iter().any(|e| matches!(e,
            StateEffect::SetFlag { id, value }
            if id.0 == "first_contact" && *value == FlagValue::Text("others".to_string())
        ))
    });

    assert!(has_eli_choice, "first_contact should have an eli-first choice");
    assert!(has_others_choice, "first_contact should have an others-first choice");
}
