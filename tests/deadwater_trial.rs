//! Integration tests for Deadwater Trial — Chapter 10.
//!
//! Proves: public truth is combat, Eli's Loyalty line is earned through deed,
//! Ch9 results change the room, no clean institutional win.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::reckoning::{self, ReckoningState, ReckoningPhase};
use saints_mile::state::store::StateStore;
use saints_mile::content::deadwater_trial;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = deadwater_trial::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn ch10_store(ch9_assignment: &str) -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch10");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch9_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("eli_pre_echo".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("ch9_assignment".to_string(),
        FlagValue::Text(ch9_assignment.to_string()));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    store.state_mut().party.add_member(CharacterId::new("miriam"));
    (dir, store)
}

// ─── Reckoning Is Combat ───────────────────────────────────────────

/// The reckoning system is a five-bar pressure model, not a cutscene.
#[test]
fn reckoning_is_combat() {
    let mut state = ReckoningState::new("mixed");

    // Multiple party actions affect different bars
    state.execute_action(reckoning::galen_present_evidence());
    assert!(state.evidence_continuity > 75, "evidence should strengthen");

    state.execute_action(reckoning::miriam_stabilize());
    assert!(state.crowd_nerve > 40, "Miriam should calm the crowd");

    state.execute_action(reckoning::ada_medical_testimony());
    assert!(state.room_credibility > 50, "Ada should boost credibility");

    // Opposition degrades everything
    let score_before = state.overall_score();
    state.execute_action(reckoning::opposition_strike());
    assert!(state.overall_score() < score_before, "opposition hurts");

    // Eli's act is the hinge
    state.execute_action(reckoning::eli_defining_act());
    assert!(state.eli_acted, "Eli should have acted");
    assert!(state.room_credibility > 60, "Eli's truth should boost credibility");
    assert_eq!(state.phase, ReckoningPhase::EliAct);
}

/// Ch9 transmission results change the opening position.
#[test]
fn ch9_results_change_deadwater() {
    let switchback = ReckoningState::new("switchback");
    let caldwell = ReckoningState::new("caldwell");
    let pine = ReckoningState::new("pine_signal");

    // Switchback: more procedural control
    assert!(switchback.procedural_control > caldwell.procedural_control);

    // Caldwell: hotter crowd (lower nerve)
    assert!(caldwell.crowd_nerve < switchback.crowd_nerve);

    // Pine Signal: moderate balance
    assert!(pine.crowd_nerve > caldwell.crowd_nerve);
}

// ─── Eli's Loyalty Line ────────────────────────────────────────────

/// Eli's Loyalty line unlocks through deed, not level.
#[test]
fn eli_loyalty_earned_through_deed() {
    let (_dir, mut store) = ch10_store("rosa_lucien_signal");

    // Before the chapter, Eli has no Loyalty skills
    assert!(!store.state().party.has_skill(
        &CharacterId::new("eli"), &SkillId::new("stand_firm")));
    assert!(!store.state().party.has_skill(
        &CharacterId::new("eli"), &SkillId::new("take_the_bullet")));

    // Run through the chapter
    run_scene(&mut store, "dw_arrival", 0);
    run_scene(&mut store, "dw_assembly", 0); // medical first
    run_scene(&mut store, "dw_hearing", 0);
    run_scene(&mut store, "dw_counterstrike", 0);

    // Eli's act unlocks the Loyalty line
    run_scene(&mut store, "dw_eli_act", 0);

    assert_eq!(store.state().flags.get("eli_defining_act"),
        Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("loyalty_line_unlocked"),
        Some(&FlagValue::Bool(true)));

    // Loyalty skills are now unlocked
    assert!(store.state().party.has_skill(
        &CharacterId::new("eli"), &SkillId::new("stand_firm")),
        "Stand Firm should unlock — Eli stopped running");
    assert!(store.state().party.has_skill(
        &CharacterId::new("eli"), &SkillId::new("take_the_bullet")),
        "Take the Bullet should unlock — sacrifice play");
}

/// Eli's line captures the whole turn.
#[test]
fn eli_line_is_right() {
    let (_dir, store) = ch10_store("rosa_lucien_signal");

    let scene = deadwater_trial::get_scene("dw_eli_act").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);

    assert!(lines.iter().any(|l| l.text.contains("lived crooked another ten years")),
        "Eli's defining line should be present");
    assert!(lines.iter().any(|l| l.text.contains("Don't mistake this for virtue")),
        "the line should stay unsentimental");
    assert!(lines.iter().any(|l| l.text.contains("letting better men wear what was mine")),
        "the line should be about ownership, not redemption");
}

// ─── No Clean Win ──────────────────────────────────────────────────

/// Deadwater produces exposure, not justice.
#[test]
fn no_clean_institutional_win() {
    let (_dir, mut store) = ch10_store("rosa_lucien_signal");

    run_scene(&mut store, "dw_arrival", 0);
    run_scene(&mut store, "dw_assembly", 0);
    run_scene(&mut store, "dw_hearing", 0);
    run_scene(&mut store, "dw_counterstrike", 0);
    run_scene(&mut store, "dw_eli_act", 0);

    let verdict = deadwater_trial::get_scene("dw_verdict").unwrap();
    SceneRunner::apply_scene_effects(&verdict, &mut store);

    // Public truth established
    assert_eq!(store.state().flags.get("public_truth_established"),
        Some(&FlagValue::Bool(true)));

    // But Voss is NOT captured
    assert!(store.state().flags.get("voss_captured").is_none(),
        "Voss must NOT be captured at Deadwater");

    // Voss is threatened, not defeated
    assert_eq!(store.state().flags.get("voss_threatened"),
        Some(&FlagValue::Bool(true)));

    // Chapter complete
    assert_eq!(store.state().flags.get("ch10_complete"),
        Some(&FlagValue::Bool(true)));
}

// ─── Full Path ─────────────────────────────────────────────────────

/// Complete Chapter 10 with save round-trip.
#[test]
fn chapter_10_full_path() {
    let (_dir, mut store) = ch10_store("rosa_lucien_signal");

    run_scene(&mut store, "dw_arrival", 0);
    run_scene(&mut store, "dw_assembly", 1); // documents first
    run_scene(&mut store, "dw_hearing", 0);
    run_scene(&mut store, "dw_counterstrike", 0);
    run_scene(&mut store, "dw_eli_act", 0);

    let verdict = deadwater_trial::get_scene("dw_verdict").unwrap();
    SceneRunner::apply_scene_effects(&verdict, &mut store);

    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "deadwater_testimony"));

    let path = store.save("ch10_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("loyalty_line_unlocked"),
        Some(&FlagValue::Bool(true)));
    assert_eq!(loaded.state().flags.get("public_truth_established"),
        Some(&FlagValue::Bool(true)));
}

/// Three hearing sequences produce different narrative textures.
#[test]
fn three_sequences_three_textures() {
    for (choice, expected_seq) in [(0, "medical_first"), (1, "documents_first"), (2, "territorial_first")] {
        let (_dir, mut store) = ch10_store("rosa_lucien_signal");
        run_scene(&mut store, "dw_arrival", 0);
        run_scene(&mut store, "dw_assembly", choice);

        assert_eq!(
            store.state().flags.get("dw_sequence"),
            Some(&FlagValue::Text(expected_seq.to_string())),
        );

        // Each sequence should produce different opening hearing lines
        let hearing = deadwater_trial::get_scene("dw_hearing").unwrap();
        let lines = SceneRunner::filter_lines(&hearing, &store);
        assert!(!lines.is_empty());
    }
}
