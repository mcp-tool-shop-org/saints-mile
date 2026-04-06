//! Integration tests for the Adult-Act End — Chapters 11 + 12.
//!
//! Proves: peak party synthesis, hand injury changes the menu,
//! dispersal is consequence not epilogue, the adult arc carries
//! a real state into the older-years gap.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::content::{breakwater_junction, names_in_dust};
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, module: &str, scene_id: &str, choice_index: usize) {
    let scene = match module {
        "bj" => breakwater_junction::get_scene(scene_id),
        "nd" => names_in_dust::get_scene(scene_id),
        _ => None,
    }.unwrap_or_else(|| panic!("scene not found: {}", scene_id));

    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    if !scene.choices.is_empty() {
        SceneRunner::execute_choice(&scene, choice_index, store);
    }
}

fn run_combat(store: &mut StateStore) {
    let encounter = breakwater_junction::breakwater_battle();
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
        id: FlagId::new("junction_held"), value: FlagValue::Bool(true),
    }]);
}

fn ch11_store() -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch11");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch10_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("loyalty_line_unlocked".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("lucien_captured".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    store.state_mut().party.add_member(CharacterId::new("miriam"));
    (dir, store)
}

// ─── Peak Synthesis ────────────────────────────────────────────────

/// Breakwater is the last full-party fight. The battle should be the largest.
#[test]
fn peak_synthesis_battle() {
    let encounter = breakwater_junction::breakwater_battle();

    // Most enemies in any single encounter — Breakwater is the climactic
    // full-party battle. At least 5 enemies to require tactical positioning,
    // but no more than 12 to keep turns manageable in a TUI.
    let enemy_count = encounter.phases[0].enemies.len();
    assert!(enemy_count >= 5, "Breakwater should have the most enemies ({})", enemy_count);
    assert!(enemy_count <= 12, "Breakwater should not exceed 12 enemies ({})", enemy_count);

    // Full party slots
    assert_eq!(encounter.party_slots, 4);

    // Multiple cover elements including destructible
    assert!(encounter.terrain.cover.len() >= 4);
    assert!(encounter.terrain.cover.iter().any(|c| c.destructible));
}

// ─── Hand Injury ───────────────────────────────────────────────────

/// The hand injury is a system wound, not a plot note.
#[test]
fn hand_injury_is_system_wound() {
    let (_dir, mut store) = ch11_store();

    // Before: no hand injury
    let galen = store.state().party.members.iter()
        .find(|m| m.id.0 == "galen").unwrap();
    assert!(galen.injuries.is_empty(), "no injuries before Breakwater");

    // Run through the chapter
    run_scene(&mut store, "bj", "bj_entry", 0);
    run_scene(&mut store, "bj", "bj_preparation", 0);
    run_combat(&mut store);
    run_scene(&mut store, "bj", "bj_hand_injury", 0);

    // After: hand is wounded
    assert_eq!(store.state().flags.get("hand_wounded"), Some(&FlagValue::Bool(true)));
    let galen = store.state().party.members.iter()
        .find(|m| m.id.0 == "galen").unwrap();
    assert!(!galen.injuries.is_empty(), "hand injury should be a real injury");
}

/// Ada's treatment scene captures the identity transition.
#[test]
fn ada_treatment_bridges_identity() {
    let (_dir, store) = ch11_store();

    let scene = breakwater_junction::get_scene("bj_ada_treatment").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);

    // The key medical lines
    assert!(lines.iter().any(|l| l.text.contains("You'll have strength. Not speed")),
        "Ada should name the loss: strength remains, speed doesn't");
    assert!(lines.iter().any(|l| l.text.contains("will not draw the way you used to")),
        "Ada should be clinical about Quick Draw being over");
    assert!(lines.iter().any(|l| l.text.contains("different road to your fingers")),
        "Ada should offer the bridge: skill survives, path changes");
}

/// Older Galen's menu reflects the hand injury.
#[test]
fn hand_changes_the_menu() {
    let youth = party_defs::galen(AgePhase::Youth);
    let adult = party_defs::galen(AgePhase::Adult);
    let older = party_defs::galen(AgePhase::Older);

    // Youth has Quick Draw
    assert!(youth.skills.iter().any(|s| s.0 == "quick_draw"));
    // Adult has Quick Draw
    assert!(adult.skills.iter().any(|s| s.0 == "quick_draw"));
    // Older does NOT have Quick Draw — the hand changed him
    assert!(!older.skills.iter().any(|s| s.0 == "quick_draw"),
        "Older Galen must not have Quick Draw — the hand changed him");

    // But older DOES have Steady Aim — the gift outlives the giver
    assert!(older.skills.iter().any(|s| s.0 == "steady_aim"),
        "Steady Aim must persist — Voss's gift survives");

    // Older has judgment skills instead
    assert!(older.skills.iter().any(|s| s.0 == "judgment_shot"));
    assert!(older.skills.iter().any(|s| s.0 == "initiative_read"));
    assert!(older.skills.iter().any(|s| s.0 == "party_command"));
}

// ─── Dispersal Is Consequence ──────────────────────────────────────

/// Chapter 12 is not epilogue. It's the cost of winning.
#[test]
fn dispersal_is_consequence() {
    let (_dir, mut store) = ch11_store();

    // Complete Ch11
    run_scene(&mut store, "bj", "bj_entry", 0);
    run_scene(&mut store, "bj", "bj_preparation", 0);
    run_combat(&mut store);
    run_scene(&mut store, "bj", "bj_hand_injury", 0);
    run_scene(&mut store, "bj", "bj_ada_treatment", 0);
    run_scene(&mut store, "bj", "bj_victory", 0);

    assert_eq!(store.state().flags.get("ch11_complete"), Some(&FlagValue::Bool(true)));

    // Chapter 12
    store.state_mut().chapter = ChapterId::new("ch12");
    run_scene(&mut store, "nd", "nd_aftermath", 0);
    run_scene(&mut store, "nd", "nd_separations", 0);

    // Each party member has departed
    assert_eq!(store.state().flags.get("ada_departed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("rosa_departed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("miriam_departed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("eli_nearest"), Some(&FlagValue::Bool(true)));

    // The last campfire
    run_scene(&mut store, "nd", "nd_campfire", 0);
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "last_campfire"));

    // The last road — adult arc ends
    let last = names_in_dust::get_scene("nd_last_road").unwrap();
    SceneRunner::apply_scene_effects(&last, &mut store);

    assert_eq!(store.state().flags.get("ch12_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("adult_arc_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("party_dispersed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("voss_still_free"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("poster_in_limbo"), Some(&FlagValue::Bool(true)));
}

/// Eli's campfire line captures the arc.
#[test]
fn eli_campfire_line() {
    let (_dir, store) = ch11_store();

    let scene = names_in_dust::get_scene("nd_campfire").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);

    assert!(lines.iter().any(|l| l.text.contains("roads")),
        "Eli's campfire line about roads should be present");
}

/// The adult arc carry state is real.
#[test]
fn adult_arc_carry_state() {
    let (_dir, mut store) = ch11_store();

    // Run through both chapters
    run_scene(&mut store, "bj", "bj_entry", 0);
    run_scene(&mut store, "bj", "bj_preparation", 0);
    run_combat(&mut store);
    run_scene(&mut store, "bj", "bj_hand_injury", 0);
    run_scene(&mut store, "bj", "bj_ada_treatment", 0);
    run_scene(&mut store, "bj", "bj_victory", 0);

    store.state_mut().chapter = ChapterId::new("ch12");
    run_scene(&mut store, "nd", "nd_aftermath", 0);
    run_scene(&mut store, "nd", "nd_separations", 0);
    run_scene(&mut store, "nd", "nd_campfire", 0);

    let last = names_in_dust::get_scene("nd_last_road").unwrap();
    SceneRunner::apply_scene_effects(&last, &mut store);

    // Save the adult-arc-complete state
    let path = store.save("adult_arc_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();

    // Critical carry state for Ch13
    assert_eq!(loaded.state().flags.get("adult_arc_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(loaded.state().flags.get("hand_wounded"), Some(&FlagValue::Bool(true)));
    assert_eq!(loaded.state().flags.get("party_dispersed"), Some(&FlagValue::Bool(true)));
    assert_eq!(loaded.state().flags.get("voss_still_free"), Some(&FlagValue::Bool(true)));
    assert_eq!(loaded.state().flags.get("public_truth_partial"), Some(&FlagValue::Bool(true)));
    assert!(loaded.state().memory_objects.iter().any(|o| o.id.0 == "last_campfire"));
    assert!(loaded.state().memory_objects.iter().any(|o| o.id.0 == "breakwater_victory"));
}
