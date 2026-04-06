//! Wave 4 feature tests — validates wound recovery, combo counter,
//! terrain modifiers, save backup, GameSettings, quickstart, and
//! screen transition schema.

mod common;

use saints_mile::types::*;
use saints_mile::combat::types::*;
use saints_mile::combat::engine::*;
use saints_mile::combat::wounds;
use saints_mile::dev::quickstart::JumpPoint;
use saints_mile::state::store::StateStore;
use saints_mile::state::settings::{GameSettings, TEXT_SPEED_MIN, TEXT_SPEED_MAX};
use saints_mile::state::types::GameState;
use saints_mile::scene::types::StateEffect;
use tempfile::TempDir;

// ─── Wound Recovery ─────────────────────────────────────────────

/// recover_wound removes a treatable wound when Ada is present.
#[test]
fn wound_recover_with_ada_removes_wound() {
    let mut wounds_vec = vec![wounds::gunshot_wound(), wounds::exhaustion()];
    assert_eq!(wounds_vec.len(), 2);

    let result = wounds::recover_wound(&mut wounds_vec, 0, true);
    assert!(result, "should succeed with Ada present");
    assert_eq!(wounds_vec.len(), 1);
    assert_eq!(wounds_vec[0].id.0, "exhaustion");
}

/// recover_wound fails without Ada.
#[test]
fn wound_recover_without_ada_fails() {
    let mut wounds_vec = vec![wounds::gunshot_wound()];
    let result = wounds::recover_wound(&mut wounds_vec, 0, false);
    assert!(!result);
    assert_eq!(wounds_vec.len(), 1);
}

/// rest_recovery heals the oldest minor wound only.
#[test]
fn rest_recovery_heals_minor_only() {
    let mut wounds_vec = vec![
        wounds::gunshot_wound(),  // Major
        wounds::exhaustion(),     // Minor (oldest minor)
        wounds::blunt_trauma(),   // Minor
    ];

    let healed = wounds::rest_recovery(&mut wounds_vec);
    assert_eq!(healed, Some(InjuryId::new("exhaustion")));
    assert_eq!(wounds_vec.len(), 2);
    // gunshot and blunt_trauma remain
    assert_eq!(wounds_vec[0].id.0, "gunshot");
    assert_eq!(wounds_vec[1].id.0, "blunt_trauma");
}

/// rest_recovery returns None when only major wounds present.
#[test]
fn rest_recovery_skips_major() {
    let mut wounds_vec = vec![wounds::gunshot_wound(), wounds::nerve_shock()];
    let healed = wounds::rest_recovery(&mut wounds_vec);
    assert_eq!(healed, None);
    assert_eq!(wounds_vec.len(), 2);
}

// ─── Combo Counter ──────────────────────────────────────────────

/// Build a minimal EncounterState for combo testing.
fn combo_test_state() -> EncounterState {
    let encounter = Encounter {
        id: EncounterId::new("combo_test"),
        phases: vec![CombatPhase {
            id: "main".to_string(),
            description: "Combo test".to_string(),
            enemies: vec![EnemyTemplate {
                id: "thug".to_string(),
                name: "Thug".to_string(),
                hp: 20, nerve: 15, damage: 5, accuracy: 50,
                speed: 8, bluff: 10, nerve_threshold: 5,
            }],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None,
        party_slots: 4,
        terrain: Terrain {
            name: "Test".to_string(),
            cover: vec![],
            hazards: vec![],
        },
        objectives: vec![],
        outcome_effects: vec![],
        escapable: true,
    };

    let party = vec![(
        "galen".to_string(), "Galen Rook".to_string(),
        40, 30, 12, 12, 70, 10,
        vec![SkillId::new("quick_draw")],
        vec![],
        vec![],
    )];

    EncounterState::new(&encounter, party)
}

/// First use of a skill line returns 1.0 multiplier (no combo).
#[test]
fn combo_first_use_no_bonus() {
    let mut state = combo_test_state();
    let mult = state.record_combo("galen", "Deadeye");
    assert!((mult - 1.0).abs() < f32::EPSILON, "first use should be 1.0x");
}

/// Second consecutive same-line action returns 1.1 multiplier.
#[test]
fn combo_second_use_gives_bonus() {
    let mut state = combo_test_state();
    state.record_combo("galen", "Deadeye");
    let mult = state.record_combo("galen", "Deadeye");
    assert!((mult - 1.1).abs() < f32::EPSILON, "second consecutive should be 1.1x");
}

/// Third consecutive same-line action returns 1.2 multiplier (max).
#[test]
fn combo_third_use_max_bonus() {
    let mut state = combo_test_state();
    state.record_combo("galen", "Deadeye");
    state.record_combo("galen", "Deadeye");
    let mult = state.record_combo("galen", "Deadeye");
    assert!((mult - 1.2).abs() < f32::EPSILON, "third+ consecutive should be 1.2x");
}

/// Switching skill lines resets the combo.
#[test]
fn combo_resets_on_line_switch() {
    let mut state = combo_test_state();
    state.record_combo("galen", "Deadeye");
    state.record_combo("galen", "Deadeye");
    // Switch to a different line
    let mult = state.record_combo("galen", "Command");
    assert!((mult - 1.0).abs() < f32::EPSILON, "switching lines should reset to 1.0x");
}

/// reset_combo clears state for an actor.
#[test]
fn combo_reset_clears() {
    let mut state = combo_test_state();
    state.record_combo("galen", "Deadeye");
    state.record_combo("galen", "Deadeye");
    state.reset_combo("galen");

    let mult = state.record_combo("galen", "Deadeye");
    assert!((mult - 1.0).abs() < f32::EPSILON, "reset should clear combo state");
}

// ─── Terrain Modifiers ──────────────────────────────────────────

/// All terrain modifier types exist and can be constructed.
#[test]
fn terrain_modifier_types_exist() {
    let _cratered = TerrainModifier::Cratered { source: "explosion".to_string() };
    let _burning = TerrainModifier::Burning { damage_per_turn: 5 };
    let _flooded = TerrainModifier::Flooded { nerve_penalty: 3 };
    let _cleared = TerrainModifier::Cleared { former_cover: "wagon".to_string() };
}

/// Terrain modifiers can be applied and checked.
#[test]
fn terrain_modifiers_apply_and_check() {
    let mut state = combo_test_state();

    state.apply_terrain_modifier(TerrainModifier::Burning { damage_per_turn: 5 });
    state.apply_terrain_modifier(TerrainModifier::Flooded { nerve_penalty: 3 });

    let (hp_dmg, nerve_pen) = state.check_terrain_effects();
    assert_eq!(hp_dmg, 5);
    assert_eq!(nerve_pen, 3);
}

/// Multiple terrain modifiers stack.
#[test]
fn terrain_modifiers_stack() {
    let mut state = combo_test_state();

    state.apply_terrain_modifier(TerrainModifier::Burning { damage_per_turn: 5 });
    state.apply_terrain_modifier(TerrainModifier::Burning { damage_per_turn: 3 });

    let (hp_dmg, _) = state.check_terrain_effects();
    assert_eq!(hp_dmg, 8, "two burning modifiers should stack");
}

// ─── Save Backup ────────────────────────────────────────────────

/// Saving to an existing slot creates a .bak backup file.
#[test]
fn save_backup_created_on_overwrite() {
    let dir = TempDir::new().unwrap();
    let store = StateStore::new_game(dir.path());

    // First save
    let path = store.save("backup_test").unwrap();
    assert!(path.exists());

    // Second save overwrites — should create .bak
    let path2 = store.save("backup_test").unwrap();
    assert!(path2.exists());

    let bak = dir.path().join("backup_test.ron.bak");
    assert!(bak.exists(), "backup file should exist after overwrite");
}

/// First save (no existing file) should NOT create a .bak.
#[test]
fn save_no_backup_on_first_write() {
    let dir = TempDir::new().unwrap();
    let store = StateStore::new_game(dir.path());

    store.save("first_save").unwrap();

    let bak = dir.path().join("first_save.ron.bak");
    assert!(!bak.exists(), "no backup on first save");
}

// ─── GameSettings ───────────────────────────────────────────────

/// Default settings have text_speed_multiplier at 1.0.
#[test]
fn settings_default_text_speed() {
    let s = GameSettings::default();
    assert!((s.text_speed_multiplier - 1.0).abs() < f32::EPSILON);
}

/// Settings round-trip through save/load.
#[test]
fn settings_save_load_round_trip() {
    let dir = TempDir::new().unwrap();
    let settings = GameSettings { text_speed_multiplier: 1.5 };
    settings.save(dir.path()).unwrap();

    let loaded = GameSettings::load(dir.path());
    assert!((loaded.text_speed_multiplier - 1.5).abs() < f32::EPSILON);
}

/// Text speed multiplier is clamped to valid range.
#[test]
fn settings_text_speed_clamped() {
    let mut too_low = GameSettings { text_speed_multiplier: 0.1 };
    too_low.validate();
    assert!((too_low.text_speed_multiplier - TEXT_SPEED_MIN).abs() < f32::EPSILON);

    let mut too_high = GameSettings { text_speed_multiplier: 10.0 };
    too_high.validate();
    assert!((too_high.text_speed_multiplier - TEXT_SPEED_MAX).abs() < f32::EPSILON);
}

/// apply_text_speed correctly adjusts base rate.
#[test]
fn settings_apply_text_speed() {
    let fast = GameSettings { text_speed_multiplier: 2.0 };
    assert_eq!(fast.apply_text_speed(30), 15);

    let slow = GameSettings { text_speed_multiplier: 0.5 };
    assert_eq!(slow.apply_text_speed(30), 60);

    // Crisis pacing (0ms) stays instant regardless of multiplier
    assert_eq!(fast.apply_text_speed(0), 0);
}

// ─── Quickstart / JumpPoint ─────────────────────────────────────

/// Each JumpPoint creates a state with Galen in the party.
#[test]
fn jump_points_always_include_galen() {
    for jp in JumpPoint::all() {
        let state = jp.create_state();
        assert!(state.party.has_member(&CharacterId::new("galen")),
            "{:?} must include Galen", jp);
    }
}

/// JumpPoints that cascade carry earlier flags forward.
#[test]
fn jump_point_cascading_carries_flags() {
    let bitter_cut = JumpPoint::BitterCutFight.create_state();
    // Should carry flags from all earlier jump points
    assert_eq!(
        bitter_cut.flags.get("bandit_camp_done"),
        Some(&FlagValue::Bool(true)),
        "BitterCutFight should carry bandit_camp_done from BitterCutDispatch",
    );
    assert_eq!(
        bitter_cut.flags.get("voss_taught_steady_aim"),
        Some(&FlagValue::Bool(true)),
        "Should carry steady_aim from CedarWakeBanditCamp",
    );
}

/// ConvoyStart transitions to YoungMan age phase.
#[test]
fn jump_point_convoy_age_transition() {
    let convoy = JumpPoint::ConvoyStart.create_state();
    assert_eq!(convoy.age_phase, AgePhase::YoungMan);
    assert_eq!(convoy.chapter.0, "ch2");
}

// ─── Screen Transition Schema ───────────────────────────────────

/// AppScreen variants exist and can be constructed.
/// This validates the screen transition type system is wired.
#[test]
fn screen_variants_constructable() {
    use saints_mile::ui::AppScreen;

    let _title = AppScreen::Title;
    let _scene = AppScreen::Scene {
        chapter_label: "Prologue".to_string(),
        location_label: "Trail".to_string(),
    };
    let _combat = AppScreen::Combat;
    let _standoff = AppScreen::Standoff;
    let _outcome = AppScreen::CombatOutcome;
    let _error = AppScreen::Error {
        message: "test".to_string(),
        return_screen: Box::new(AppScreen::Title),
    };
    let _pause = AppScreen::Pause {
        return_screen: Box::new(AppScreen::Title),
    };
    let _status = AppScreen::Status {
        return_screen: Box::new(AppScreen::Title),
    };
}

/// ConfirmQuit carries the return screen for cancel.
#[test]
fn confirm_quit_carries_return_screen() {
    use saints_mile::ui::AppScreen;

    let scene_screen = AppScreen::Scene {
        chapter_label: "Ch1".to_string(),
        location_label: "Cedar Wake".to_string(),
    };
    let _quit = AppScreen::ConfirmQuit {
        return_screen: Box::new(scene_screen),
    };
    // Construction succeeds — return_screen is boxed correctly
}
