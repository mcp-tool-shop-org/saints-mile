//! Tests for content/names_in_dust.rs — Chapter 12.

use saints_mile::types::*;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::content::names_in_dust;
use saints_mile::state::store::StateStore;
use tempfile::TempDir;

#[test]
fn all_scenes_are_available() {
    let scene_ids = ["nd_aftermath", "nd_separations", "nd_campfire", "nd_last_road"];
    for id in &scene_ids {
        assert!(names_in_dust::get_scene(id).is_some(),
            "scene {} should exist", id);
    }
}

#[test]
fn nonexistent_scene_returns_none() {
    assert!(names_in_dust::get_scene("nd_fake").is_none());
    assert!(names_in_dust::get_scene("").is_none());
}

#[test]
fn aftermath_sets_ch12_started() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    let scene = names_in_dust::get_scene("nd_aftermath").unwrap();
    SceneRunner::apply_scene_effects(&scene, &mut store);
    assert_eq!(store.state().flags.get("ch12_started"), Some(&FlagValue::Bool(true)));
}

#[test]
fn separations_set_departure_flags() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    let scene = names_in_dust::get_scene("nd_separations").unwrap();
    SceneRunner::apply_scene_effects(&scene, &mut store);

    assert_eq!(store.state().flags.get("ada_departed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("rosa_departed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("miriam_departed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("eli_nearest"), Some(&FlagValue::Bool(true)));
}

#[test]
fn last_road_completes_adult_arc() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    let scene = names_in_dust::get_scene("nd_last_road").unwrap();
    SceneRunner::apply_scene_effects(&scene, &mut store);

    assert_eq!(store.state().flags.get("ch12_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("adult_arc_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("party_dispersed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("voss_still_free"), Some(&FlagValue::Bool(true)));
}

#[test]
fn last_road_has_memory_ref_to_ch14() {
    let scene = names_in_dust::get_scene("nd_last_road").unwrap();
    assert!(!scene.memory_refs.is_empty());
    let campfire_ref = scene.memory_refs.iter().find(|r| r.object.0 == "last_campfire");
    assert!(campfire_ref.is_some(), "last_road should echo the campfire to ch14");
    assert_eq!(campfire_ref.unwrap().target_chapter.as_ref().unwrap().0, "ch14");
}

#[test]
fn campfire_adds_memory_object() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    let scene = names_in_dust::get_scene("nd_campfire").unwrap();
    SceneRunner::apply_scene_effects(&scene, &mut store);

    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "last_campfire"),
        "campfire scene should add last_campfire memory object");
}
