//! Integration tests for The Burned Mission — Chapter 8.
//!
//! Proves: the party is the investigative instrument, revelation is older
//! not bigger, Lucien reads himself in the walls, the bell stays unresolved.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::state::investigation::{self, InvestigationDomain};
use saints_mile::content::burned_mission;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = burned_mission::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn run_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = burned_mission::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));
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
        id: FlagId::new("mission_records_defended"), value: FlagValue::Bool(true),
    }]);
}

fn ch8_store() -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch8");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch7_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("conspiracy_documented".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("lucien_captured".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    store.state_mut().party.add_member(CharacterId::new("miriam"));
    (dir, store)
}

// ─── Party Is the Tool ────────────────────────────────────────────

/// No single party member can produce the full revelation alone.
#[test]
fn party_is_the_tool() {
    let mut investigation = investigation::burned_mission_investigation();

    // Only Ada can read medical
    let med = investigation.read_fragment("medical_records", &CharacterId::new("ada"));
    assert!(med.is_some(), "Ada should read medical records");

    // Eli cannot read medical
    let mut inv2 = investigation::burned_mission_investigation();
    let med_eli = inv2.read_fragment("medical_records", &CharacterId::new("eli"));
    assert!(med_eli.is_none(), "Eli cannot read medical records");

    // Only Lucien can read fire patterns
    let fire = investigation.read_fragment("fire_pattern", &CharacterId::new("lucien"));
    assert!(fire.is_some(), "Lucien should read fire patterns");
    assert!(fire.unwrap().contains("same language"),
        "Lucien's fire-pattern reading should include 'same language'");

    // Only Miriam can read the death register
    let death = investigation.read_fragment("death_register", &CharacterId::new("miriam"));
    assert!(death.is_some(), "Miriam should read death register");

    // No single reader produces convergence
    assert!(!investigation.convergence_reached,
        "reading 3 fragments should not produce convergence");

    // Need all domains for convergence
    investigation.read_fragment("financial_transfers", &CharacterId::new("eli"));
    investigation.read_fragment("land_grants", &CharacterId::new("galen"));
    investigation.read_fragment("water_terrain", &CharacterId::new("rosa"));

    let converged = investigation.check_convergence(&[
        InvestigationDomain::Medical,
        InvestigationDomain::Financial,
        InvestigationDomain::LandGrant,
        InvestigationDomain::DeathRegister,
        InvestigationDomain::FirePattern,
        InvestigationDomain::Terrain,
    ]);
    assert!(converged, "all 6 domains should produce convergence");
    assert_eq!(investigation.discovered_count(), 6);
}

// ─── Revelation Is Older ───────────────────────────────────────────

/// The output ties present conspiracy to older violence, not just more documents.
#[test]
fn revelation_is_older_not_bigger() {
    let (_dir, mut store) = ch8_store();

    run_scene(&mut store, "bm_valley_entry", 0);
    run_scene(&mut store, "bm_ruins", 0);
    run_scene(&mut store, "bm_basement", 0);
    run_scene(&mut store, "bm_party_reads", 0);

    // The revelation is about history, not just more names
    assert_eq!(store.state().flags.get("historical_fraud_discovered"),
        Some(&FlagValue::Bool(true)));

    // It proves the fire was deliberate — older than the railroad
    assert_eq!(store.state().flags.get("mission_records_read"),
        Some(&FlagValue::Bool(true)));

    // It does NOT name Voss directly (that's Ch10+)
    assert!(store.state().flags.get("voss_named").is_none());

    // It does NOT resolve the conspiracy (that's Ch10+)
    assert!(store.state().flags.get("conspiracy_resolved").is_none());
}

// ─── Lucien Reads Himself ──────────────────────────────────────────

/// Lucien's chapter state changes through fire-pattern recognition, not confession.
#[test]
fn lucien_reads_himself_in_the_walls() {
    let (_dir, mut store) = ch8_store();

    run_scene(&mut store, "bm_valley_entry", 0);
    run_scene(&mut store, "bm_ruins", 0);
    run_scene(&mut store, "bm_basement", 0);

    // The party reads scene includes Lucien's fire pattern read
    let reads = burned_mission::get_scene("bm_party_reads").unwrap();
    let lines = SceneRunner::filter_lines(&reads, &store);

    // Lucien's line: "This was a job. Better than mine, but the same language."
    assert!(lines.iter().any(|l| l.text.contains("same language")),
        "Lucien should read the fire pattern as professional work");

    // The narration recognizes this as a lineage, not a confession
    assert!(lines.iter().any(|l| l.text.contains("lineage")),
        "The narration should frame Lucien's recognition as lineage discovery");

    run_scene(&mut store, "bm_party_reads", 0);
    assert_eq!(store.state().flags.get("lucien_reads_fire_pattern"),
        Some(&FlagValue::Bool(true)));
}

// ─── Bell Stays Unresolved ─────────────────────────────────────────

/// The chapter records the bell phenomenon without ever resolving it.
#[test]
fn bell_remains_unresolved() {
    let (_dir, mut store) = ch8_store();

    run_scene(&mut store, "bm_valley_entry", 0);
    run_scene(&mut store, "bm_ruins", 0);
    run_scene(&mut store, "bm_basement", 0);
    run_scene(&mut store, "bm_party_reads", 0);

    let bell = burned_mission::get_scene("bm_bell_moment").unwrap();
    let lines = SceneRunner::filter_lines(&bell, &store);

    // Multiple interpretations exist in the same scene
    let ada_explains = lines.iter().any(|l| l.text.contains("Acoustic resonance"));
    let eli_dismisses = lines.iter().any(|l| l.text.contains("Irrelevant"));
    let rosa_warns = lines.iter().any(|l| l.text.contains("old things make noise"));
    let miriam_listens = lines.iter().any(|l| l.text.contains("..."));

    assert!(ada_explains, "Ada should offer a rational explanation");
    assert!(eli_dismisses, "Eli should dismiss it");
    assert!(rosa_warns, "Rosa should be practical-superstitious");
    assert!(miriam_listens, "Miriam should simply listen");

    // The scene does NOT contain a definitive answer
    assert!(!lines.iter().any(|l|
        l.text.contains("the bell is supernatural") ||
        l.text.contains("the bell is just") ||
        l.text.contains("the truth is that the bell")
    ), "the bell must NOT be explained or resolved");

    // The flag records the experience, not the explanation
    run_scene(&mut store, "bm_bell_moment", 0);
    assert_eq!(store.state().flags.get("bell_heard"), Some(&FlagValue::Bool(true)));
}

// ─── Full Chapter Path ─────────────────────────────────────────────

/// Complete Chapter 8: valley → ruins → basement → party reads →
/// bell → fight → chapter close.
#[test]
fn chapter_8_full_path() {
    let (_dir, mut store) = ch8_store();

    run_scene(&mut store, "bm_valley_entry", 0);
    run_scene(&mut store, "bm_ruins", 0);
    run_scene(&mut store, "bm_basement", 0);
    run_scene(&mut store, "bm_party_reads", 0);
    run_scene(&mut store, "bm_bell_moment", 0);
    run_combat(&mut store, "mission_defense");

    let close = burned_mission::get_scene("bm_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    assert_eq!(store.state().flags.get("ch8_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("fire_was_deliberate"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("regrant_was_fraud"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("mission_truth_recovered"), Some(&FlagValue::Bool(true)));

    // Memory objects
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "mission_land_grants"));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "death_register_discrepancy"));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "bell_phenomenon"));

    // Bell echoes to Ch15 (unresolved to the end)
    let close_scene = burned_mission::get_scene("bm_chapter_close").unwrap();
    assert!(close_scene.memory_refs.iter().any(|r|
        r.object.0 == "bell_phenomenon" &&
        r.target_chapter.as_ref().map_or(false, |c| c.0 == "ch15")
    ), "bell phenomenon should echo to the final chapter");

    // Save round-trip
    let path = store.save("ch8_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("regrant_was_fraud"), Some(&FlagValue::Bool(true)));
}
