//! Integration tests for Fifteen Years Gone — Chapter 13.
//!
//! Proves: the return feels necessary, older Galen is not weaker,
//! the wrong history is becoming permanent, the first contact is selective.

use saints_mile::types::*;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::state::history::HistoricalState;
use saints_mile::content::fifteen_years_gone;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) {
    let scene = fifteen_years_gone::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    if !scene.choices.is_empty() {
        SceneRunner::execute_choice(&scene, choice_index, store);
    }
}

fn ch13_store(relay_branch: &str) -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch13");
    store.state_mut().age_phase = AgePhase::Older;
    store.state_mut().flags.insert("adult_arc_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("hand_wounded".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("party_dispersed".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("voss_still_free".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("relay_branch".to_string(),
        FlagValue::Text(relay_branch.to_string()));
    // Older Galen is solo at the start of the return
    store.state_mut().party.members.retain(|m| m.id.0 == "galen");
    (dir, store)
}

// ─── Older Galen Feels Necessary ───────────────────────────────────

/// Older Galen is not a nerfed version. He is a different kind of threat.
#[test]
fn older_galen_feels_necessary_not_weaker() {
    let older = party_defs::galen(AgePhase::Older);
    let adult = party_defs::galen(AgePhase::Adult);

    // Most accurate he's ever been
    assert!(older.accuracy > adult.accuracy,
        "older Galen should be MORE accurate ({} > {})", older.accuracy, adult.accuracy);

    // Hardest-hitting
    assert!(older.damage > adult.damage,
        "older Galen should hit HARDER ({} > {})", older.damage, adult.damage);

    // Higher nerve — harder to break
    assert!(older.nerve > adult.nerve,
        "older Galen should have MORE nerve ({} > {})", older.nerve, adult.nerve);

    // But slower
    assert!(older.speed < adult.speed,
        "older Galen should be SLOWER ({} < {})", older.speed, adult.speed);

    // Has judgment skills: initiative_read, party_command, judgment_shot
    assert!(older.skills.iter().any(|s| s.0 == "judgment_shot"),
        "older should have judgment_shot — one shot, certain");
    assert!(older.skills.iter().any(|s| s.0 == "initiative_read"),
        "older should predict enemy action");
    assert!(older.skills.iter().any(|s| s.0 == "party_command"),
        "older should direct others' turns");

    // No Quick Draw — the hand changed him
    assert!(!older.skills.iter().any(|s| s.0 == "quick_draw"));
    // Still has Steady Aim — the gift outlives the giver
    assert!(older.skills.iter().any(|s| s.0 == "steady_aim"));
    // Overwatch is now his signature
    assert!(older.skills.iter().any(|s| s.0 == "overwatch") ||
            older.skills.iter().any(|s| s.0 == "called_shot_precise"));
}

// ─── Wrong History ─────────────────────────────────────────────────

/// The wrong version of events is becoming permanent.
#[test]
fn wrong_history_is_becoming_permanent() {
    let history = HistoricalState::fifteen_years_later();

    // The institutional version is winning
    assert!(history.wrong_version_winning(),
        "the wrong version should be winning after 15 years");

    // The truth gap is real
    assert!(history.truth_gap() > 30,
        "there should be a significant gap between institutional and private truth");

    // Public and institutional are actively maintained
    assert!(history.public_version.actively_maintained);
    assert!(history.institutional_version.actively_maintained);

    // Road and private are fading
    assert!(!history.road_version.actively_maintained);
    assert!(!history.private_version.actively_maintained);

    // Place memory doesn't fade
    assert_eq!(history.place_version.entrenchment, 100,
        "place memory should not fade — the bell is permanent");
}

// ─── Not Nostalgia ─────────────────────────────────────────────────

/// The return chapter is not a victory lap. It's historical emergency.
#[test]
fn return_is_not_nostalgia() {
    let (_dir, mut store) = ch13_store("tom");

    run_scene(&mut store, "fg_return", 0);

    // Chapter started
    assert_eq!(store.state().flags.get("ch13_started"), Some(&FlagValue::Bool(true)));

    // The official lie scene shows false memorialization
    let lie_scene = fifteen_years_gone::get_scene("fg_official_lie").unwrap();
    let lines = SceneRunner::filter_lines(&lie_scene, &store);

    // Should contain the false plaque
    assert!(lines.iter().any(|l| l.text.contains("diligence of territorial authorities")),
        "the official lie should whitewash what happened");

    // Should mention Deadwater being reduced
    assert!(lines.iter().any(|l| l.text.contains("cited, not felt")),
        "Deadwater should be reduced to a citation");

    run_scene(&mut store, "fg_official_lie", 0);
    assert_eq!(store.state().flags.get("official_lie_seen"), Some(&FlagValue::Bool(true)));
}

// ─── Relay Branch Echoes ───────────────────────────────────────────

/// The old witness scene reflects which relay branch the player chose.
#[test]
fn relay_branch_echoes_in_return() {
    for branch in &["tom", "nella", "papers"] {
        let (_dir, store) = ch13_store(branch);

        let witness = fifteen_years_gone::get_scene("fg_old_witness").unwrap();
        let lines = SceneRunner::filter_lines(&witness, &store);

        let has_branch_echo = lines.iter().any(|l| match *branch {
            "tom" => l.text.contains("Tom Reed") || l.text.contains("road make sense"),
            "nella" => l.text.contains("bad coffee") || l.text.contains("kept people alive"),
            "papers" => l.text.contains("papers") || l.text.contains("only thing true"),
            _ => false,
        });
        assert!(has_branch_echo,
            "{} branch should echo in the old witness's memory", branch);
    }
}

// ─── Selective Contact ─────────────────────────────────────────────

/// First reassembly contact is selective, not automatic.
#[test]
fn first_contact_is_selective() {
    let (_dir, mut store) = ch13_store("tom");

    run_scene(&mut store, "fg_return", 0);
    run_scene(&mut store, "fg_official_lie", 0);
    run_scene(&mut store, "fg_old_witness", 0);

    // First contact: reach for Eli
    run_scene(&mut store, "fg_first_contact", 0);
    assert_eq!(store.state().flags.get("first_contact"),
        Some(&FlagValue::Text("eli".to_string())));
    assert_eq!(store.state().flags.get("reaching_for_eli"),
        Some(&FlagValue::Bool(true)));

    // The full party is NOT reassembled yet
    assert!(!store.state().party.has_member(&CharacterId::new("ada")));
    assert!(!store.state().party.has_member(&CharacterId::new("rosa")));
    assert!(!store.state().party.has_member(&CharacterId::new("miriam")));
}

// ─── Full Chapter Path ─────────────────────────────────────────────

/// Complete Chapter 13 with save round-trip.
#[test]
fn chapter_13_full_path() {
    let (_dir, mut store) = ch13_store("nella");

    run_scene(&mut store, "fg_return", 0);
    run_scene(&mut store, "fg_official_lie", 0);
    run_scene(&mut store, "fg_old_witness", 0);
    run_scene(&mut store, "fg_first_contact", 0);

    let close = fifteen_years_gone::get_scene("fg_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    assert_eq!(store.state().flags.get("ch13_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("return_committed"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("voss_on_plaque_committee"), Some(&FlagValue::Bool(true)));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "return_necessity"));

    // Save round-trip
    let path = store.save("ch13_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("return_committed"), Some(&FlagValue::Bool(true)));
}

/// Older Galen with modified context flags still constructs a valid party.
/// Guards against panics when flags are missing or age_phase is unexpected.
#[test]
fn older_galen_with_modified_context_does_not_panic() {
    // Create an older Galen state but strip some expected flags
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch13");
    store.state_mut().age_phase = AgePhase::Older;
    // Deliberately omit adult_arc_complete, hand_wounded, party_dispersed
    // to simulate a corrupted or partial save state
    store.state_mut().flags.insert("voss_still_free".to_string(), FlagValue::Bool(true));
    store.state_mut().party.members.retain(|m| m.id.0 == "galen");

    // party_defs::galen should not panic with Older age phase
    let older = party_defs::galen(AgePhase::Older);
    assert!(!older.skills.is_empty(), "older Galen should have skills regardless of flags");
    assert!(older.accuracy > 0, "older Galen should have positive accuracy");

    // The scene system should handle missing flags gracefully
    let scene = fifteen_years_gone::get_scene("fg_return").unwrap();
    let prepared = SceneRunner::prepare_scene(&scene, &store);
    // Scene should still be playable even without full adult arc flags
    assert!(prepared.should_play, "fg_return should play even with partial flags");
}
