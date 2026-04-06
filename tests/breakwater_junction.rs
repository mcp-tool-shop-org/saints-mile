//! Tests for content/breakwater_junction.rs — Chapter 11.

use saints_mile::types::*;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::content::breakwater_junction;
use saints_mile::state::store::StateStore;
use tempfile::TempDir;

#[test]
fn all_scenes_are_available() {
    let scene_ids = ["bj_entry", "bj_preparation", "bj_hand_injury", "bj_ada_treatment", "bj_victory"];
    for id in &scene_ids {
        assert!(breakwater_junction::get_scene(id).is_some(),
            "scene {} should exist", id);
    }
}

#[test]
fn nonexistent_scene_returns_none() {
    assert!(breakwater_junction::get_scene("bj_fake").is_none());
    assert!(breakwater_junction::get_scene("").is_none());
}

#[test]
fn entry_scene_sets_ch11_started() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch11");

    let scene = breakwater_junction::get_scene("bj_entry").unwrap();
    SceneRunner::apply_scene_effects(&scene, &mut store);
    assert_eq!(store.state().flags.get("ch11_started"), Some(&FlagValue::Bool(true)));
}

#[test]
fn breakwater_battle_encounter_exists() {
    let enc = breakwater_junction::get_encounter("breakwater_battle");
    assert!(enc.is_some());
    let enc = enc.unwrap();
    assert_eq!(enc.id.0, "breakwater_battle");
    assert!(!enc.phases.is_empty());
    assert_eq!(enc.party_slots, 4);
    // Terrain has cover
    assert!(!enc.terrain.cover.is_empty(), "battle should have cover elements");
}

#[test]
fn hand_injury_scene_has_injury_effect() {
    let scene = breakwater_junction::get_scene("bj_hand_injury").unwrap();
    // The choice should contain an ApplyInjury effect
    assert!(!scene.choices.is_empty());
    let choice = &scene.choices[0];
    let has_injury = choice.effects.iter().any(|e| {
        matches!(e, saints_mile::scene::types::StateEffect::ApplyInjury { .. })
    });
    assert!(has_injury, "hand injury scene must apply an injury");
}

#[test]
fn victory_scene_sets_completion_flags() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    let scene = breakwater_junction::get_scene("bj_victory").unwrap();
    SceneRunner::apply_scene_effects(&scene, &mut store);

    assert_eq!(store.state().flags.get("ch11_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("breakwater_held"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("retaliation_answered"), Some(&FlagValue::Bool(true)));
}

#[test]
fn victory_scene_has_memory_ref_to_ch13() {
    let scene = breakwater_junction::get_scene("bj_victory").unwrap();
    assert!(!scene.memory_refs.is_empty());
    let hand_ref = scene.memory_refs.iter().find(|r| r.object.0 == "hand_injury");
    assert!(hand_ref.is_some(), "victory should reference hand_injury for ch13");
    assert_eq!(hand_ref.unwrap().target_chapter.as_ref().unwrap().0, "ch13");
}
