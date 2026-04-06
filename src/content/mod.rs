//! Content module — authored game content and authoring helpers.

pub mod prologue;
pub mod cedar_wake;
pub mod saints_mile_convoy;
pub mod black_willow;
pub mod ropehouse_blood;
pub mod dust_revival;
pub mod fuse_country;
pub mod iron_ledger;
pub mod burned_mission;
pub mod long_wire;
pub mod deadwater_trial;
pub mod breakwater_junction;
pub mod names_in_dust;
pub mod fifteen_years_gone;
pub mod old_friends;
pub mod saints_mile_again;
pub mod builders;

use crate::scene::types::Scene;
use crate::combat::types::Encounter;

/// Chapter scene dispatcher. To add a new chapter:
/// 1. Create `src/content/new_chapter.rs` with `pub fn get_scene(id: &str) -> Option<Scene>`
/// 2. Add `pub mod new_chapter;` to this file
/// 3. Add a match arm in `get_scene()` below
/// 4. Add a match arm in `get_encounter()` below (use `=> None` if no encounters)
pub fn get_scene(chapter: &str, id: &str) -> Option<Scene> {
    match chapter {
        "prologue" => prologue::get_scene(id),
        "cedar_wake" => cedar_wake::get_scene(id),
        "saints_mile_convoy" => saints_mile_convoy::get_scene(id),
        "black_willow" => black_willow::get_scene(id),
        "ropehouse_blood" => ropehouse_blood::get_scene(id),
        "dust_revival" => dust_revival::get_scene(id),
        "fuse_country" => fuse_country::get_scene(id),
        "iron_ledger" => iron_ledger::get_scene(id),
        "burned_mission" => burned_mission::get_scene(id),
        "long_wire" => long_wire::get_scene(id),
        "deadwater_trial" => deadwater_trial::get_scene(id),
        "breakwater_junction" => breakwater_junction::get_scene(id),
        "names_in_dust" => names_in_dust::get_scene(id),
        "fifteen_years_gone" => fifteen_years_gone::get_scene(id),
        "old_friends" => old_friends::get_scene(id),
        "saints_mile_again" => saints_mile_again::get_scene(id),
        _ => None,
    }
}

/// Chapter encounter dispatcher. To add a new chapter:
/// 1. Create `src/content/new_chapter.rs` with `pub fn get_scene(id: &str) -> Option<Scene>`
/// 2. Add `pub mod new_chapter;` to this file
/// 3. Add a match arm in `get_scene()` above
/// 4. Add a match arm in `get_encounter()` below (use `=> None` if no encounters)
pub fn get_encounter(chapter: &str, id: &str) -> Option<Encounter> {
    match chapter {
        "prologue" => prologue::get_encounter(id),
        "cedar_wake" => cedar_wake::get_encounter(id),
        "saints_mile_convoy" => saints_mile_convoy::get_encounter(id),
        "black_willow" => black_willow::get_encounter(id),
        "ropehouse_blood" => ropehouse_blood::get_encounter(id),
        "dust_revival" => dust_revival::get_encounter(id),
        "fuse_country" => fuse_country::get_encounter(id),
        "iron_ledger" => iron_ledger::get_encounter(id),
        "burned_mission" => burned_mission::get_encounter(id),
        "breakwater_junction" => breakwater_junction::get_encounter(id),
        "long_wire" => None,           // no combat encounters in this chapter
        "deadwater_trial" => None,     // no combat encounters in this chapter
        "names_in_dust" => None,       // no combat encounters in this chapter
        "fifteen_years_gone" => None,  // no combat encounters in this chapter
        "old_friends" => None,         // no combat encounters in this chapter
        "saints_mile_again" => None,   // no combat encounters in this chapter
        _ => None,
    }
}
