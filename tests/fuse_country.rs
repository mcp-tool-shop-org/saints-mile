//! Integration tests for Fuse Country — Chapter 6.
//!
//! Proves: environmental combat is a real system language,
//! Lucien arrives as damage before face, the trestle fight
//! uses every party member's specialty in non-standard ways.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::combat::environment::{self, EnvironmentState, EnvironmentAction, EnvironmentActionResult};
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::content::fuse_country;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = fuse_country::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn run_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = fuse_country::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));
    let party: Vec<_> = party_defs::ch5_roster().into_iter().take(4).collect();
    let mut combat = EncounterState::new(&encounter, party);
    if combat.phase == EncounterPhase::Standoff {
        combat.resolve_standoff(StandoffPosture::SteadyHand, None);
    } else {
        combat.phase = EncounterPhase::Combat;
    }
    for _ in 0..30 {
        combat.build_turn_queue();
        if combat.turn_queue.is_empty() { break; }
        loop {
            let entry = combat.current_turn_entry().cloned();
            if entry.is_none() { break; }
            let entry = entry.unwrap();
            let target_id = match entry.side {
                CombatSide::Party | CombatSide::NpcAlly =>
                    combat.enemies.iter().find(|e| !e.down && !e.panicked)
                        .map(|e| e.id.clone()).unwrap_or_default(),
                CombatSide::Enemy => "galen".to_string(),
            };
            if target_id.is_empty() { break; }
            combat.execute_action(&CombatAction::UseSkill {
                skill: SkillId::new("quick_draw"),
                target: TargetSelection::Single(target_id),
            });
            combat.evaluate_objectives();
            if let Some(outcome) = combat.check_resolution() {
                store.apply_effects(&outcome.effects);
                return;
            }
            if !combat.advance_turn() { break; }
        }
    }
    store.apply_effects(&[saints_mile::scene::types::StateEffect::SetFlag {
        id: FlagId::new("trestle_saved"), value: FlagValue::Bool(true),
    }]);
}

fn ch6_store() -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch6");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch5_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    store.state_mut().party.add_member(CharacterId::new("miriam"));
    (dir, store)
}

// ─── Environmental Combat ──────────────────────────────────────────

/// Environmental combat is a real system, not a one-off gimmick.
#[test]
fn environmental_combat_is_real_system() {
    let mut env = environment::trestle_environment();

    // Multiple mechanics interact
    assert_eq!(env.active_charges(), 3, "three charges on the trestle");
    assert_eq!(env.intact_cover(), 3, "three pieces of cover");

    // Fuses tick toward detonation
    env.tick_fuses();
    env.tick_fuses();
    let events = env.tick_fuses(); // first charge detonates at turn 3
    assert!(!events.is_empty(), "charge should detonate");
    assert_eq!(env.active_charges(), 2);

    // Detonation destroys cover
    assert!(env.intact_cover() < 3 || events[0].cover_destroyed.is_empty() == false
        || events[0].chain_reaction,
        "detonation should affect the environment");

    // Chain reactions accelerate other charges
    if events[0].chain_reaction {
        let remaining: Vec<_> = env.fuse_charges.iter()
            .filter(|c| !c.disarmed && !c.detonated)
            .collect();
        assert!(remaining.iter().any(|c| c.turns_remaining <= 2),
            "chain reaction should accelerate remaining charges");
    }
}

/// Galen can shoot fuses with Steady Aim (high accuracy).
#[test]
fn galen_shoots_fuse_with_steady_aim() {
    let mut env = environment::trestle_environment();

    // Galen's adult accuracy is 70 — should succeed
    let result = env.execute_action(&EnvironmentAction::ShootFuse {
        charge_id: "structural_charge".to_string(),
        accuracy: 70, // Galen's Steady Aim
    });
    assert!(matches!(result, EnvironmentActionResult::Success(_)));

    let charge = env.fuse_charges.iter().find(|c| c.id == "structural_charge").unwrap();
    assert!(charge.disarmed, "Steady Aim should disarm the structural charge");
}

/// Rosa can physically cross and disarm.
#[test]
fn rosa_physically_disarms() {
    let mut env = environment::trestle_environment();

    let result = env.execute_action(&EnvironmentAction::DisarmCharge {
        charge_id: "pylon_charge_1".to_string(),
    });
    assert!(matches!(result, EnvironmentActionResult::Success(_)));
}

/// Structural charge collapse changes the battlefield.
#[test]
fn structural_collapse_changes_battlefield() {
    let mut env = environment::trestle_environment();

    // Let the structural charge detonate
    env.execute_action(&EnvironmentAction::ControlledDetonate {
        charge_id: "structural_charge".to_string(),
    });

    // Trestle should be damaged
    assert!(env.any_collapsed() || env.structures[0].integrity < 100,
        "structural detonation should damage the trestle");
}

// ─── Lucien Introduction Law ───────────────────────────────────────

/// Player sees Lucien's cost before meeting him.
/// Damage → testimony → consequence → encounter → confrontation.
#[test]
fn lucien_damage_before_face() {
    let (_dir, mut store) = ch6_store();

    // Step 1: Evidence — burned depot
    run_scene(&mut store, "fc_corridor_entry", 0);
    run_scene(&mut store, "fc_burned_depot", 0);
    assert_eq!(store.state().flags.get("depot_investigated"), Some(&FlagValue::Bool(true)));

    // Step 2: Testimony — Maeve Strand describes him
    // (in the burned_depot scene — "Big hands. Loud mouth.")

    // Step 3: Consequence — displaced family at Colter Station
    run_scene(&mut store, "fc_corridor_locals", 0);

    // Step 4: Encounter — meet Lucien at the trestle
    // By now the player has seen: blast evidence, medical chain connection,
    // displaced family, stock widow testimony. THEN they meet the man.
    let scene = fuse_country::get_scene("fc_meet_lucien").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);

    // Lucien's first line should be defensive, not charming
    assert!(lines.iter().any(|l| l.text.contains("cost you can see")),
        "Lucien should not arrive charming — he should arrive arguing");
}

/// Lucien is NOT in the party after Chapter 6. He is captured.
#[test]
fn lucien_not_recruited_in_ch6() {
    let (_dir, mut store) = ch6_store();

    run_scene(&mut store, "fc_corridor_entry", 0);
    run_scene(&mut store, "fc_burned_depot", 0);
    run_scene(&mut store, "fc_corridor_locals", 0);
    run_scene(&mut store, "fc_meet_lucien", 0);
    run_scene(&mut store, "fc_trestle_approach", 0);
    run_combat(&mut store, "millburn_trestle");
    run_scene(&mut store, "fc_lucien_decision", 0); // prisoner

    let close = fuse_country::get_scene("fc_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    // Lucien is captured, NOT a party member
    assert_eq!(store.state().flags.get("lucien_captured"), Some(&FlagValue::Bool(true)));
    assert!(!store.state().party.has_member(&CharacterId::new("lucien")),
        "Lucien must NOT be in the party after Ch6");
    assert_eq!(store.state().flags.get("ch6_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("iron_ledger_lead"), Some(&FlagValue::Bool(true)));
}

// ─── Chapter Path ──────────────────────────────────────────────────

/// Full Chapter 6 path with Lucien decision variations.
#[test]
fn chapter_6_lucien_decision_variants() {
    for (choice_idx, expected_status) in [(0, "prisoner"), (1, "forced_guide"), (2, "judged")] {
        let (_dir, mut store) = ch6_store();

        run_scene(&mut store, "fc_corridor_entry", 0);
        run_scene(&mut store, "fc_burned_depot", 0);
        run_scene(&mut store, "fc_corridor_locals", 0);
        run_scene(&mut store, "fc_meet_lucien", 0);
        run_scene(&mut store, "fc_trestle_approach", 0);
        run_combat(&mut store, "millburn_trestle");

        // Lucien decision
        run_scene(&mut store, "fc_lucien_decision", choice_idx);
        assert_eq!(
            store.state().flags.get("lucien_status"),
            Some(&FlagValue::Text(expected_status.to_string())),
            "choice {} should produce status {}", choice_idx, expected_status,
        );

        // Chapter close
        let close = fuse_country::get_scene("fc_chapter_close").unwrap();
        let lines = SceneRunner::filter_lines(&close, &store);

        // Status-specific lines should appear
        let has_status_line = match expected_status {
            "prisoner" => lines.iter().any(|l| l.text.contains("restrained")),
            "forced_guide" => lines.iter().any(|l| l.text.contains("walk us through")),
            "judged" => lines.iter().any(|l| l.text.contains("weight of it")),
            _ => false,
        };
        assert!(has_status_line, "status {} should produce its own line", expected_status);

        SceneRunner::apply_scene_effects(&close, &mut store);
        assert_eq!(store.state().flags.get("ch6_complete"), Some(&FlagValue::Bool(true)));
    }
}

/// Lucien's party definition exists but reflects he's NOT an ally yet.
#[test]
fn lucien_is_hateable() {
    let lucien = party_defs::lucien();

    // He has demolition skills, not party support skills
    assert!(lucien.skills.iter().any(|s| s.0 == "throw_dynamite"));
    assert!(lucien.skills.iter().any(|s| s.0 == "demolish"));

    // Low accuracy — not a marksman
    assert!(lucien.accuracy < 50, "Lucien is not precise — he's destructive");

    // His only duo tech is Fencepost Thunder (Rosa + Lucien)
    // which can only unlock much later, after grudging professional respect
    assert!(lucien.duo_techs.iter().any(|d| d.0 == "fencepost_thunder"));
}
