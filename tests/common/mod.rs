//! Shared test helpers for Saints Mile integration tests.
//!
//! Eliminates duplication of run_scene / run_combat across chapter test files.
//! Usage: add `mod common;` to your test file, then call `common::run_scene(...)`.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::content;
use saints_mile::state::store::StateStore;

// ─── Relay Branch Constants ──────────────────────────────────────────
// Use these instead of scattering string literals across test files.

pub const RELAY_TOM: &str = "tom";
pub const RELAY_NELLA: &str = "nella";
pub const RELAY_PAPERS: &str = "papers";

// ─── Scene Runner ────────────────────────────────────────────────────

/// Run a scene through the central content dispatcher, apply effects,
/// execute the given choice, and return the transition.
///
/// Panics if the scene is not found, should not play, or the choice is unavailable.
pub fn run_scene(
    store: &mut StateStore,
    chapter: &str,
    scene_id: &str,
    choice_index: usize,
) -> SceneTransition {
    let scene = content::get_scene(chapter, scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}::{}", chapter, scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

// ─── Combat Runner ───────────────────────────────────────────────────

/// Maximum rounds before asserting combat failed to resolve.
pub const MAX_COMBAT_ROUNDS: usize = 30;

/// Run an already-constructed EncounterState to resolution.
///
/// Handles standoff resolution, the turn loop, and asserts that combat
/// resolves within MAX_COMBAT_ROUNDS. Returns (resolved, rounds_taken).
/// If resolved, applies outcome effects to the store.
pub fn run_combat(
    combat: &mut EncounterState,
    store: &mut StateStore,
) -> (bool, usize) {
    if combat.phase == EncounterPhase::Standoff {
        combat.resolve_standoff(StandoffPosture::SteadyHand, None);
    } else {
        combat.phase = EncounterPhase::Combat;
    }

    for round in 0..MAX_COMBAT_ROUNDS {
        combat.build_turn_queue();
        if combat.turn_queue.is_empty() { break; }
        loop {
            let entry = combat.current_turn_entry().cloned();
            if entry.is_none() { break; }
            let entry = entry.unwrap();
            let target_id = match entry.side {
                CombatSide::Party | CombatSide::NpcAlly => {
                    combat.enemies.iter()
                        .find(|e| !e.down && !e.panicked)
                        .map(|e| e.id.clone())
                        .unwrap_or_default()
                }
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
                return (true, round + 1);
            }
            if !combat.advance_turn() { break; }
        }
    }

    (false, MAX_COMBAT_ROUNDS)
}
