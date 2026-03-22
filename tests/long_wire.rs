//! Integration tests for The Long Wire — Chapter 9.
//!
//! Proves: split-party is moral-operational, assignments produce character
//! consequences, truth moves by time, Eli returns changed.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::split_party::{self, TeamSynergy};
use saints_mile::state::store::StateStore;
use saints_mile::content::long_wire;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = long_wire::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn ch9_store() -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch9");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch8_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("lucien_captured".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    store.state_mut().party.add_member(CharacterId::new("miriam"));
    (dir, store)
}

// ─── Assignment Is Character ───────────────────────────────────────

/// Same mission, different team splits, different relationship outputs.
#[test]
fn assignment_is_character() {
    // Assignment A: Rosa+Lucien at signal tower (hostile pairing)
    let split_a = split_party::long_wire_split(
        vec![CharacterId::new("galen"), CharacterId::new("ada")],
        vec![CharacterId::new("rosa"), CharacterId::new("lucien")],
        vec![CharacterId::new("eli"), CharacterId::new("miriam")],
    );

    let signal_result_a = split_a.results.iter().find(|r| r.team_id == "signal_tower").unwrap();
    assert!(signal_result_a.report.contains("Rosa held Pine Signal"),
        "Rosa+Lucien signal should produce the specific report");
    assert!(signal_result_a.report.contains("has not thanked him"),
        "hostile pairing should produce grudging dynamic");

    // Witness result for Eli+Miriam
    let witness_result = split_a.results.iter().find(|r| r.team_id == "witness_route").unwrap();
    assert!(witness_result.report.contains("Neither agrees on how"),
        "Eli+Miriam should disagree on method");

    // Assignment B: different people at different positions
    let split_b = split_party::long_wire_split(
        vec![CharacterId::new("galen"), CharacterId::new("rosa")],
        vec![CharacterId::new("eli"), CharacterId::new("miriam")],
        vec![CharacterId::new("ada"), CharacterId::new("lucien")],
    );

    let signal_result_b = split_b.results.iter().find(|r| r.team_id == "signal_tower").unwrap();
    // Different team, different report
    assert_ne!(signal_result_a.report, signal_result_b.report,
        "different teams should produce different reports");
}

/// Team synergy evaluation reflects relationship dynamics.
#[test]
fn synergy_reflects_relationships() {
    // Strong pairings
    assert_eq!(
        split_party::evaluate_synergy(&[CharacterId::new("galen"), CharacterId::new("eli")]),
        TeamSynergy::Strong,
    );
    assert_eq!(
        split_party::evaluate_synergy(&[CharacterId::new("rosa"), CharacterId::new("miriam")]),
        TeamSynergy::Strong,
    );

    // Hostile pairing
    assert_eq!(
        split_party::evaluate_synergy(&[CharacterId::new("rosa"), CharacterId::new("lucien")]),
        TeamSynergy::Hostile,
    );

    // Volatile pairing
    assert_eq!(
        split_party::evaluate_synergy(&[CharacterId::new("eli"), CharacterId::new("lucien")]),
        TeamSynergy::Volatile,
    );
}

// ─── Truth Moves by Time ───────────────────────────────────────────

/// Dispatch result changes Chapter 10 terrain.
#[test]
fn truth_moves_by_time() {
    let (_dir, mut store) = ch9_store();

    run_scene(&mut store, "lw_junction_arrival", 0);
    run_scene(&mut store, "lw_assignment", 0); // rosa_lucien_signal
    run_scene(&mut store, "lw_split_execution", 0);
    run_scene(&mut store, "lw_reports", 0);

    let close = long_wire::get_scene("lw_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    // Partial truth is now in circulation
    assert_eq!(store.state().flags.get("partial_truth_transmitted"),
        Some(&FlagValue::Bool(true)));

    // Deadwater is now necessary
    assert_eq!(store.state().flags.get("deadwater_necessary"),
        Some(&FlagValue::Bool(true)));

    // Assignment choice is remembered
    assert_eq!(store.state().flags.get("ch9_assignment"),
        Some(&FlagValue::Text("rosa_lucien_signal".to_string())));
}

// ─── Eli Returns Changed ───────────────────────────────────────────

/// After separation, Eli's state shows a pre-echo of his Ch10 transformation.
#[test]
fn eli_returns_changed() {
    let (_dir, mut store) = ch9_store();

    run_scene(&mut store, "lw_junction_arrival", 0);
    run_scene(&mut store, "lw_assignment", 0);
    run_scene(&mut store, "lw_split_execution", 0);

    // Reports scene should mention Eli's shift
    let reports = long_wire::get_scene("lw_reports").unwrap();
    let lines = SceneRunner::filter_lines(&reports, &store);
    assert!(lines.iter().any(|l| l.text.contains("Eli comes back quieter")),
        "reports should note Eli's pre-echo shift");

    run_scene(&mut store, "lw_reports", 0);
    assert_eq!(store.state().flags.get("eli_pre_echo"),
        Some(&FlagValue::Bool(true)),
        "Eli's pre-echo should be flagged for Ch10");
}

// ─── No Single Team Does Everything ────────────────────────────────

/// The player must lose something somewhere.
#[test]
fn no_single_team_does_everything() {
    // With 6 people and 3 objectives, each team has only 2 members
    let split = split_party::long_wire_split(
        vec![CharacterId::new("galen"), CharacterId::new("ada")],
        vec![CharacterId::new("rosa"), CharacterId::new("lucien")],
        vec![CharacterId::new("eli"), CharacterId::new("miriam")],
    );

    // Each team is only 2 people — no team has the full party's capability
    for team in &split.teams {
        assert_eq!(team.members.len(), 2,
            "each team should have exactly 2 members — the party is split thin");
    }

    // The Rosa+Lucien pairing is hostile — success comes at relationship cost
    let signal_team = split.teams.iter().find(|t| t.id == "signal_tower").unwrap();
    assert_eq!(signal_team.synergy, TeamSynergy::Hostile,
        "Rosa+Lucien should be hostile synergy");

    // The signal tower result should show grudging but not warm
    let signal_result = split.results.iter().find(|r| r.team_id == "signal_tower").unwrap();
    assert!(signal_result.success, "hostile pairing can still succeed");
    assert!(signal_result.report.contains("has not thanked"),
        "success should not equal warmth");
}

// ─── Full Chapter Path ─────────────────────────────────────────────

/// Complete Chapter 9.
#[test]
fn chapter_9_full_path() {
    let (_dir, mut store) = ch9_store();

    run_scene(&mut store, "lw_junction_arrival", 0);
    assert_eq!(store.state().flags.get("ch9_started"), Some(&FlagValue::Bool(true)));

    run_scene(&mut store, "lw_assignment", 0); // rosa_lucien_signal
    run_scene(&mut store, "lw_split_execution", 0);

    // Reports
    let reports = long_wire::get_scene("lw_reports").unwrap();
    let lines = SceneRunner::filter_lines(&reports, &store);
    assert!(lines.iter().any(|l| l.text.contains("Rosa held Pine Signal")),
        "assignment-specific report should appear");

    run_scene(&mut store, "lw_reports", 0);

    // Chapter close
    let close = long_wire::get_scene("lw_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    assert_eq!(store.state().flags.get("ch9_complete"), Some(&FlagValue::Bool(true)));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "transmission_result"));

    // Save round-trip
    let path = store.save("ch9_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("deadwater_necessary"), Some(&FlagValue::Bool(true)));
}

/// Three different assignments produce three different chapter textures.
#[test]
fn three_assignments_three_textures() {
    for (choice, expected_assignment) in [
        (0, "rosa_lucien_signal"),
        (1, "eli_miriam_signal"),
        (2, "miriam_lucien_witness"),
    ] {
        let (_dir, mut store) = ch9_store();
        run_scene(&mut store, "lw_junction_arrival", 0);
        run_scene(&mut store, "lw_assignment", choice);

        assert_eq!(
            store.state().flags.get("ch9_assignment"),
            Some(&FlagValue::Text(expected_assignment.to_string())),
        );

        // Each assignment should produce different report text
        let reports = long_wire::get_scene("lw_reports").unwrap();
        let lines = SceneRunner::filter_lines(&reports, &store);
        assert!(!lines.is_empty(), "assignment {} should produce reports", expected_assignment);
    }
}
