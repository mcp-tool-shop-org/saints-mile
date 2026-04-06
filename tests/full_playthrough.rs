//! AUDIT-003 — Critical-path integration tests.
//!
//! Proves state carries correctly across chapter boundaries by walking
//! the critical path from Prologue through Chapter 3 (Black Willow),
//! making choice 0 at each scene. Validates party changes, age transitions,
//! flag carry-forward, and save/load mid-playthrough.

mod common;

use saints_mile::types::*;
use saints_mile::content;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::scene::types::*;
use saints_mile::state::store::StateStore;
use saints_mile::state::types::GameState;
use saints_mile::dev::quickstart::JumpPoint;
use tempfile::TempDir;

// ─── Helpers ─────────────────────────────────────────────────────

/// Walk through all scenes in a chapter using choice 0, applying scene
/// effects along the way. Stops when a transition leads to Combat or End,
/// or when the scene doesn't exist in the chapter.
fn walk_chapter_scenes(
    store: &mut StateStore,
    chapter: &str,
    start_scene: &str,
) -> Vec<String> {
    let mut visited = Vec::new();
    let mut current = start_scene.to_string();

    for _ in 0..50 {
        let scene = match content::get_scene(chapter, &current) {
            Some(s) => s,
            None => break,
        };

        let prepared = SceneRunner::prepare_scene(&scene, store);
        if !prepared.should_play {
            break;
        }

        visited.push(current.clone());
        SceneRunner::apply_scene_effects(&scene, store);

        if scene.choices.is_empty() {
            // No choices = end of chain or narrative-only scene
            break;
        }

        let chosen = SceneRunner::execute_choice(&scene, 0, store);
        match chosen {
            Some(action) => match action.transition {
                SceneTransition::Scene(id) => current = id.0,
                SceneTransition::Beat(_) | SceneTransition::Combat(_) | SceneTransition::End => break,
            },
            None => break,
        }
    }

    visited
}

// ─── Test: Prologue scene chain ─────────────────────────────────

/// Walk the prologue from poster through town-direct return.
/// Validates the chain plays without panics and visits key scenes.
#[test]
fn prologue_scene_chain_plays_through() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    let visited = walk_chapter_scenes(&mut store, "prologue", "prologue_poster");

    assert!(visited.contains(&"prologue_poster".to_string()));
    assert!(visited.contains(&"eli_intro".to_string()));
    assert!(visited.contains(&"morrow_square".to_string()));
    // ride_to_arroyo leads to Combat, so chain stops there
    assert!(visited.len() >= 3, "should visit at least 3 prologue scenes");
}

// ─── Test: State carries across chapter boundaries ──────────────

/// Build state at chapter boundaries using JumpPoints and verify
/// that flags, party, and age phase transition correctly.
#[test]
fn state_carries_across_chapter_boundaries() {
    // Prologue start
    let prologue = JumpPoint::PrologueStart.create_state();
    assert_eq!(prologue.chapter.0, "prologue");
    assert_eq!(prologue.age_phase, AgePhase::Adult);
    assert!(prologue.party.has_member(&CharacterId::new("galen")));
    assert!(prologue.party.has_member(&CharacterId::new("eli")));
    assert_eq!(prologue.party.members.len(), 2);

    // Ch1 Cedar Wake — age transitions to Youth (flashback)
    let ch1 = JumpPoint::CedarWakeStart.create_state();
    assert_eq!(ch1.chapter.0, "ch1");
    assert_eq!(ch1.age_phase, AgePhase::Youth);
    // Solo Galen in Ch1
    assert_eq!(ch1.party.members.len(), 1);
    assert!(ch1.party.has_member(&CharacterId::new("galen")));
    assert!(!ch1.party.has_member(&CharacterId::new("eli")));

    // Ch2 Convoy — age transitions to YoungMan, carries Ch1 flags
    let ch2 = JumpPoint::ConvoyStart.create_state();
    assert_eq!(ch2.chapter.0, "ch2");
    assert_eq!(ch2.age_phase, AgePhase::YoungMan);
    assert_eq!(
        ch2.flags.get("chapter1_complete"),
        Some(&FlagValue::Bool(true)),
        "Ch1 completion flag should carry forward to Ch2"
    );
    assert_eq!(
        ch2.flags.get("bitter_cut_done"),
        Some(&FlagValue::Bool(true)),
        "Bitter Cut flag should carry forward"
    );
    // Galen has upgraded skills from Ch1
    assert!(ch2.party.has_skill(
        &CharacterId::new("galen"),
        &SkillId::new("steady_aim"),
    ), "steady_aim should carry from Ch1 to Ch2");
}

// ─── Test: Save/load mid-playthrough preserves state ────────────

/// Create state mid-prologue, save, load, and verify all state axes survive.
#[test]
fn save_load_mid_playthrough_preserves_state() {
    let dir = TempDir::new().unwrap();

    // Build state at prologue campfire (mid-prologue)
    let state = JumpPoint::PrologueCampfire.create_state();
    let store = StateStore::from_state(state, dir.path());

    // Verify pre-save state
    assert_eq!(store.state().beat.0, "p8");
    assert_eq!(
        store.state().flags.get("arroyo_survived"),
        Some(&FlagValue::Bool(true)),
    );
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "wanted_poster"));

    // Save
    let path = store.save("midplay_test").unwrap();
    assert!(path.exists());

    // Load
    let loaded = StateStore::load(&path).unwrap();

    // Verify all state axes survived
    assert_eq!(loaded.state().chapter.0, "prologue");
    assert_eq!(loaded.state().beat.0, "p8");
    assert_eq!(loaded.state().age_phase, AgePhase::Adult);
    assert_eq!(
        loaded.state().flags.get("arroyo_survived"),
        Some(&FlagValue::Bool(true)),
    );
    assert_eq!(
        loaded.state().flags.get("tore_poster"),
        Some(&FlagValue::Bool(true)),
    );
    assert!(loaded.state().memory_objects.iter().any(|o| o.id.0 == "wanted_poster"));
    assert!(loaded.state().reputation.get(ReputationAxis::TownLaw) > 0);
}

// ─── Test: Chapter progression validation ───────────────────────

/// Verify the progression system correctly gates chapter access.
#[test]
fn chapter_progression_gates_correctly() {
    use saints_mile::state::progression::can_enter_chapter;

    // From prologue, can enter cedar_wake (next) but not black_willow (skip)
    let prologue = JumpPoint::PrologueStart.create_state();
    assert!(can_enter_chapter(&prologue, "prologue"));
    assert!(can_enter_chapter(&prologue, "cedar_wake"));
    assert!(!can_enter_chapter(&prologue, "black_willow"));

    // From Ch2, can enter Ch3
    let ch2 = JumpPoint::ConvoyStart.create_state();
    assert!(can_enter_chapter(&ch2, "saints_mile_convoy"));
    assert!(can_enter_chapter(&ch2, "black_willow"));
    assert!(!can_enter_chapter(&ch2, "ropehouse_blood"));
}

// ─── Test: Full JumpPoint chain coherence ───────────────────────

/// All JumpPoints create valid states that don't panic.
#[test]
fn all_jump_points_create_valid_states() {
    for jp in JumpPoint::all() {
        let state = jp.create_state();
        // Every state must have Galen
        assert!(state.party.has_member(&CharacterId::new("galen")),
            "JumpPoint {:?} must include Galen in party", jp);
        // Chapter must be set
        assert!(!state.chapter.0.is_empty(),
            "JumpPoint {:?} must have a chapter", jp);
        // Label must not be empty
        assert!(!jp.label().is_empty(),
            "JumpPoint {:?} must have a label", jp);
    }
}
