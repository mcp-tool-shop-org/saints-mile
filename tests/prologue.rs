//! AUDIT-001 — Prologue test coverage.
//!
//! Validates the prologue's scene graph: poster, branches, age phase,
//! relay triage choice, and Beat 5 campfire consequence return.

mod common;

use saints_mile::types::*;
use saints_mile::content;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::scene::types::*;
use saints_mile::state::store::StateStore;
use tempfile::TempDir;

// ─── Prologue Poster ─────────────────────────────────────────────

/// The prologue entry scene exists and has exactly 2 choices.
#[test]
fn prologue_poster_exists_and_has_choices() {
    let scene = content::get_scene("prologue", "prologue_poster")
        .expect("prologue_poster scene must exist");
    assert_eq!(scene.id.0, "prologue_poster");
    assert_eq!(scene.choices.len(), 2, "poster should have 2 choices (tear/leave)");
    assert_eq!(scene.choices[0].label, "Tear it down");
    assert_eq!(scene.choices[1].label, "Leave it. Let them look.");
}

/// The entry scene is correctly routed by chapter_entry_scene.
#[test]
fn prologue_entry_scene_routing() {
    let entry = content::chapter_entry_scene("prologue");
    assert_eq!(entry, Some("prologue_poster"));
}

// ─── Prologue Branches ──────────────────────────────────────────

/// Town-direct branch: campfire choice 0 leads to return_town_direct.
#[test]
fn prologue_branch_town_direct() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // Play through poster → eli_intro → morrow_square → ride_to_arroyo → campfire_choice
    let scenes = ["prologue_poster", "eli_intro", "morrow_square", "ride_to_arroyo"];
    for scene_id in &scenes {
        common::run_scene(&mut store, "prologue", scene_id, 0);
    }

    // Campfire choice 0 = "Ride straight to town"
    let transition = common::run_scene(&mut store, "prologue", "campfire_choice", 0);
    match transition {
        SceneTransition::Scene(id) => assert_eq!(id.0, "return_town_direct"),
        other => panic!("expected Scene transition, got {:?}", other),
    }

    // Verify flag set
    assert_eq!(
        store.state().flags.get("beat5_choice"),
        Some(&FlagValue::Text("town_direct".to_string())),
    );
}

/// Homestead-first branch: campfire choice 1 leads to return_homestead.
#[test]
fn prologue_branch_homestead_first() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    let scenes = ["prologue_poster", "eli_intro", "morrow_square", "ride_to_arroyo"];
    for scene_id in &scenes {
        common::run_scene(&mut store, "prologue", scene_id, 0);
    }

    // Campfire choice 1 = "Divert to the homestead"
    let transition = common::run_scene(&mut store, "prologue", "campfire_choice", 1);
    match transition {
        SceneTransition::Scene(id) => assert_eq!(id.0, "return_homestead"),
        other => panic!("expected Scene transition, got {:?}", other),
    }

    assert_eq!(
        store.state().flags.get("beat5_choice"),
        Some(&FlagValue::Text("homestead_first".to_string())),
    );
}

// ─── Age Phase ──────────────────────────────────────────────────

/// At prologue start, age phase is Adult (Galen is 34 in the prologue).
#[test]
fn age_phase_is_adult_at_prologue() {
    let state = saints_mile::state::types::GameState::new_game();
    assert_eq!(state.age_phase, AgePhase::Adult,
        "Prologue opens in Galen's adult years");
    assert_eq!(state.chapter.0, "prologue");
}

// ─── Relay Triage Branch Flags ──────────────────────────────────

/// Setting relay branch to Tom makes the tom evidence categories available.
#[test]
fn relay_triage_tom_sets_branch() {
    let mut state = saints_mile::state::types::GameState::new_game();
    state.relay_branch = Some(RelayBranch::Tom);
    assert!(state.check_condition(&Condition::RelayBranch(RelayBranch::Tom)));
    assert!(!state.check_condition(&Condition::RelayBranch(RelayBranch::Nella)));
    assert!(!state.check_condition(&Condition::RelayBranch(RelayBranch::Papers)));

    let evidence = saints_mile::state::types::relay_evidence_available(&state);
    assert!(evidence.contains(&"structural"));
    assert!(evidence.contains(&"route"));
}

/// Setting relay branch to Nella makes human_witness categories available.
#[test]
fn relay_triage_nella_sets_branch() {
    let mut state = saints_mile::state::types::GameState::new_game();
    state.relay_branch = Some(RelayBranch::Nella);
    assert!(state.check_condition(&Condition::RelayBranch(RelayBranch::Nella)));

    let evidence = saints_mile::state::types::relay_evidence_available(&state);
    assert!(evidence.contains(&"human_witness"));
    assert!(evidence.contains(&"community"));
}

/// Setting relay branch to Papers makes documentary categories available.
#[test]
fn relay_triage_papers_sets_branch() {
    let mut state = saints_mile::state::types::GameState::new_game();
    state.relay_branch = Some(RelayBranch::Papers);
    assert!(state.check_condition(&Condition::RelayBranch(RelayBranch::Papers)));

    let evidence = saints_mile::state::types::relay_evidence_available(&state);
    assert!(evidence.contains(&"documentary"));
    assert!(evidence.contains(&"filing"));
}

// ─── Beat 5 Campfire Consequence Return ─────────────────────────

/// Town-direct return sets prologue_complete and adjusts reputation.
#[test]
fn beat5_town_direct_consequence_in_return() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // Set up state as if we just made the town_direct choice
    store.state_mut().flags.insert(
        "beat5_choice".to_string(),
        FlagValue::Text("town_direct".to_string()),
    );
    store.state_mut().flags.insert(
        "square_stance".to_string(),
        FlagValue::Text("law".to_string()),
    );

    let scene = content::get_scene("prologue", "return_town_direct")
        .expect("return_town_direct must exist");

    // Apply scene effects (they fire on play)
    SceneRunner::apply_scene_effects(&scene, &mut store);

    // Check that prologue_complete is now set
    assert_eq!(
        store.state().flags.get("prologue_complete"),
        Some(&FlagValue::Bool(true)),
    );
    // Town-direct should boost TownLaw and penalize Rancher
    assert!(store.state().reputation.get(ReputationAxis::TownLaw) > 0,
        "town direct should boost TownLaw");
    assert!(store.state().reputation.get(ReputationAxis::Rancher) < 0,
        "town direct should penalize Rancher");
}

/// Homestead return sets prologue_complete with opposite reputation shift.
#[test]
fn beat5_homestead_consequence_in_return() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    store.state_mut().flags.insert(
        "beat5_choice".to_string(),
        FlagValue::Text("homestead_first".to_string()),
    );
    store.state_mut().flags.insert(
        "square_stance".to_string(),
        FlagValue::Text("rancher".to_string()),
    );

    let scene = content::get_scene("prologue", "return_homestead")
        .expect("return_homestead must exist");

    SceneRunner::apply_scene_effects(&scene, &mut store);

    assert_eq!(
        store.state().flags.get("prologue_complete"),
        Some(&FlagValue::Bool(true)),
    );
    // Homestead should boost Rancher and penalize TownLaw
    assert!(store.state().reputation.get(ReputationAxis::Rancher) > 0,
        "homestead should boost Rancher");
    assert!(store.state().reputation.get(ReputationAxis::TownLaw) < 0,
        "homestead should penalize TownLaw");
}
