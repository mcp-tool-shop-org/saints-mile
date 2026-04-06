//! Tests for content/mod.rs — central scene/encounter dispatcher with invalid IDs.

use saints_mile::content;

#[test]
fn get_scene_nonexistent_chapter_returns_none() {
    assert!(content::get_scene("nonexistent_chapter", "any_scene").is_none());
    assert!(content::get_scene("chapter_99", "intro").is_none());
    assert!(content::get_scene("", "").is_none());
}

#[test]
fn get_scene_nonexistent_scene_id_returns_none() {
    // Valid chapter, invalid scene
    assert!(content::get_scene("prologue", "nonexistent_scene_xyz").is_none());
    assert!(content::get_scene("cedar_wake", "does_not_exist").is_none());
    assert!(content::get_scene("dust_revival", "fake_scene").is_none());
}

#[test]
fn get_encounter_nonexistent_chapter_returns_none() {
    assert!(content::get_encounter("nonexistent_chapter", "any_encounter").is_none());
    assert!(content::get_encounter("chapter_99", "battle").is_none());
    assert!(content::get_encounter("", "").is_none());
}

#[test]
fn get_encounter_nonexistent_encounter_id_returns_none() {
    // Valid chapter, invalid encounter
    assert!(content::get_encounter("prologue", "nonexistent_fight").is_none());
    assert!(content::get_encounter("fuse_country", "fake_battle").is_none());
    assert!(content::get_encounter("breakwater_junction", "no_such_fight").is_none());
}

#[test]
fn get_scene_valid_lookup_returns_some() {
    // Sanity check: valid lookups work
    let scene = content::get_scene("breakwater_junction", "bj_entry");
    assert!(scene.is_some(), "valid chapter + scene should return Some");

    let scene = content::get_scene("names_in_dust", "nd_aftermath");
    assert!(scene.is_some());
}

#[test]
fn get_encounter_valid_lookup_returns_some() {
    let enc = content::get_encounter("breakwater_junction", "breakwater_battle");
    assert!(enc.is_some(), "valid chapter + encounter should return Some");
}
