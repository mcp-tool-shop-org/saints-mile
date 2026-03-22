//! Integration tests for Old Friends, Bad Ground — Chapter 14.
//!
//! Proves: reassembly is selective, each return carries distinct truth,
//! old chemistry is changed not restored, the chapter is necessary not sentimental.

use saints_mile::types::*;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::state::store::StateStore;
use saints_mile::state::reassembly::{self, ReturnMode};
use saints_mile::content::old_friends;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) {
    let scene = old_friends::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    if !scene.choices.is_empty() {
        SceneRunner::execute_choice(&scene, choice_index, store);
    }
}

fn ch14_store() -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch14");
    store.state_mut().age_phase = AgePhase::Older;
    store.state_mut().flags.insert("ch13_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("return_committed".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("hand_wounded".to_string(), FlagValue::Bool(true));
    // Solo Galen at chapter start
    store.state_mut().party.members.retain(|m| m.id.0 == "galen");
    (dir, store)
}

// ─── Reassembly Is Selective ───────────────────────────────────────

/// Not everyone returns the same way. Each has a different mode.
#[test]
fn reassembly_is_selective_not_total() {
    let returns = reassembly::chapter_14_reassembly();

    // Different return modes
    let modes: Vec<_> = returns.iter().map(|r| r.mode).collect();
    assert!(modes.contains(&ReturnMode::Body), "some should return in body");
    assert!(modes.contains(&ReturnMode::Conditional), "some should return conditionally");

    // Not everyone is Body — that would be too easy
    assert!(modes.iter().filter(|m| **m == ReturnMode::Conditional).count() >= 1,
        "at least one return should be conditional");
}

/// Each return carries a distinct kind of truth.
#[test]
fn each_return_carries_distinct_truth() {
    let returns = reassembly::chapter_14_reassembly();

    // No two allies carry the same truth
    let truths: Vec<&str> = returns.iter().map(|r| r.truth_carried.as_str()).collect();
    for (i, truth_a) in truths.iter().enumerate() {
        for (j, truth_b) in truths.iter().enumerate() {
            if i != j {
                assert_ne!(*truth_a, *truth_b,
                    "{} and {} should carry different truths",
                    returns[i].character, returns[j].character);
            }
        }
    }

    // Specific truths are character-appropriate
    let eli = returns.iter().find(|r| r.character.0 == "eli").unwrap();
    assert!(eli.truth_carried.contains("System") || eli.truth_carried.contains("ledger"),
        "Eli should carry system intelligence");

    let ada = returns.iter().find(|r| r.character.0 == "ada").unwrap();
    assert!(ada.truth_carried.contains("Medical") || ada.truth_carried.contains("Body"),
        "Ada should carry medical witness");

    let rosa = returns.iter().find(|r| r.character.0 == "rosa").unwrap();
    assert!(rosa.truth_carried.contains("Land") || rosa.truth_carried.contains("land"),
        "Rosa should carry land truth");
}

// ─── Chemistry Changed ────────────────────────────────────────────

/// The old party's chemistry is altered, not restored.
#[test]
fn old_chemistry_is_changed_not_restored() {
    let (_dir, store) = ch14_store();

    // Ada sees the hand first — that is Ada
    let ada_scene = old_friends::get_scene("of_ada_return").unwrap();
    let lines = SceneRunner::filter_lines(&ada_scene, &store);
    assert!(lines.iter().any(|l| l.text.contains("hand") && l.text.contains("before")),
        "Ada should see the hand before commenting on anything else");

    // Rosa still sets terms — that is Rosa
    let rosa_scene = old_friends::get_scene("of_rosa_return").unwrap();
    let lines = SceneRunner::filter_lines(&rosa_scene, &store);
    assert!(lines.iter().any(|l| l.text.contains("You move when I say")),
        "Rosa should still set terms — some rhythms persist");

    // But the weight is different
    let assembly = old_friends::get_scene("of_assembly_scene").unwrap();
    let lines = SceneRunner::filter_lines(&assembly, &store);
    assert!(lines.iter().any(|l| l.text.contains("silences last longer")),
        "the assembly should acknowledge things have changed");
    assert!(lines.iter().any(|l| l.text.contains("Nobody pretends")),
        "nobody should pretend this is what it was");
}

/// Eli still has the ledger after fifteen years.
#[test]
fn eli_still_has_the_ledger() {
    let (_dir, store) = ch14_store();

    let eli_scene = old_friends::get_scene("of_eli_return").unwrap();
    let lines = SceneRunner::filter_lines(&eli_scene, &store);
    assert!(lines.iter().any(|l| l.text.contains("still has the ledger")),
        "Eli should still have the Saint's Mile ledger — after everything");
}

// ─── Necessary Not Sentimental ─────────────────────────────────────

/// The chapter is powered by necessity, not nostalgia.
#[test]
fn chapter_14_feels_necessary() {
    let returns = reassembly::chapter_14_reassembly();

    // Every return has unresolved tension — not just warm feelings
    for ally in &returns {
        assert!(!ally.tension.is_empty(),
            "{} should carry unresolved tension into the finale", ally.character);
    }

    // Can approach Saint's Mile only with enough people
    assert!(reassembly::can_approach_saints_mile(&returns),
        "with all returns, should be able to approach");

    // Without enough people, can't approach
    let too_few = vec![returns[0].clone()]; // just Eli
    assert!(!reassembly::can_approach_saints_mile(&too_few),
        "one person is not enough for the final approach");
}

// ─── Full Chapter Path ─────────────────────────────────────────────

/// Complete Chapter 14: each ally returns, party reassembles.
#[test]
fn chapter_14_full_path() {
    let (_dir, mut store) = ch14_store();

    // Eli first (nearest)
    run_scene(&mut store, "of_eli_return", 0);
    assert!(store.state().party.has_member(&CharacterId::new("eli")));
    assert_eq!(store.state().flags.get("eli_returned_body"), Some(&FlagValue::Bool(true)));

    // Ada
    run_scene(&mut store, "of_ada_return", 0);
    assert!(store.state().party.has_member(&CharacterId::new("ada")));

    // Rosa (conditional)
    run_scene(&mut store, "of_rosa_return", 0);
    assert!(store.state().party.has_member(&CharacterId::new("rosa")));
    assert_eq!(store.state().flags.get("rosa_returned_conditional"), Some(&FlagValue::Bool(true)));

    // Miriam
    run_scene(&mut store, "of_miriam_return", 0);
    assert!(store.state().party.has_member(&CharacterId::new("miriam")));

    // Assembly
    run_scene(&mut store, "of_assembly_scene", 0);

    // Chapter close
    let close = old_friends::get_scene("of_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    assert_eq!(store.state().flags.get("ch14_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("party_reassembled"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("saints_mile_approach"), Some(&FlagValue::Bool(true)));

    // Party is back — but as older versions of themselves
    assert_eq!(store.state().party.members.len(), 5); // Galen + 4 allies

    // Save round-trip
    let path = store.save("ch14_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("saints_mile_approach"), Some(&FlagValue::Bool(true)));
}
