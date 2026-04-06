//! Integration tests for Saint's Mile Again — Chapter 15.
//!
//! Proves: the ending is about authorship not just survival,
//! each ally contributes final truth, four axes produce different
//! legacies, the bell remains unresolved, the ending reads run state.

use saints_mile::types::*;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::state::store::StateStore;
use saints_mile::state::ending::{self, EndingAxis, EndingRunState};
use saints_mile::content::saints_mile_again;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) {
    let scene = saints_mile_again::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    if !scene.choices.is_empty() {
        SceneRunner::execute_choice(&scene, choice_index, store);
    }
}

fn ch15_store() -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch15");
    store.state_mut().age_phase = AgePhase::Older;
    store.state_mut().flags.insert("ch14_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("party_reassembled".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("hand_wounded".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("loyalty_line_unlocked".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("relay_branch".to_string(), FlagValue::Text("tom".to_string()));
    store.state_mut().flags.insert("public_truth_established".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("lucien_captured".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("eli"));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    store.state_mut().party.add_member(CharacterId::new("miriam"));
    (dir, store)
}

// ─── Authorship, Not Just Conspiracy ───────────────────────────────

/// Voss's confrontation is about authorship of history, not just power.
#[test]
fn voss_confrontation_is_about_authorship() {
    let (_dir, store) = ch15_store();

    let scene = saints_mile_again::get_scene("sm_voss_confrontation").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);

    // Verify Voss is actually present as a speaker in the confrontation
    assert!(lines.iter().any(|l| l.speaker.contains("voss")),
        "Voss must have speaker lines in his own confrontation scene");

    // Voss should echo the shooting post lesson
    assert!(lines.iter().any(|l| l.text.contains("decide cleanly")),
        "Voss should echo his own teaching from Cedar Wake");

    // He should argue about what happens AFTER truth
    assert!(lines.iter().any(|l| l.text.contains("next version of me")),
        "Voss should argue that exposure just creates the next authoritarian");

    // He should still sound persuasive, not ranting
    assert!(!lines.iter().any(|l|
        l.text.contains("you fool") || l.text.contains("you'll never") ||
        l.text.contains("my plan")
    ), "Voss must NOT sound like a ranting villain");
}

// ─── Four Axes, Four Legacies ──────────────────────────────────────

/// Each ending axis produces a different legacy.
#[test]
fn four_axes_four_legacies() {
    let run_state = EndingRunState {
        eli_loyalty_active: true,
        hand_wounded: true,
        relay_branch: "tom".to_string(),
        tom_alive: true,
        nella_alive: false,
        party_reassembled: true,
        lucien_present: true,
        public_truth_level: 70,
    };

    let justice = ending::resolve_ending(EndingAxis::Justice, &run_state);
    let exposure = ending::resolve_ending(EndingAxis::Exposure, &run_state);
    let peace = ending::resolve_ending(EndingAxis::Peace, &run_state);
    let burden = ending::resolve_ending(EndingAxis::Burden, &run_state);

    // Voss fates differ
    assert!(justice.voss_fate.contains("Dead"));
    assert!(exposure.voss_fate.contains("Exposed"));
    assert!(peace.voss_fate.contains("Retired"));
    assert!(burden.voss_fate.contains("Untouched"));

    // Legacies differ
    assert_ne!(justice.legacy, exposure.legacy);
    assert_ne!(exposure.legacy, peace.legacy);
    assert_ne!(peace.legacy, burden.legacy);

    // Exposure with Eli loyalty should mention Eli's testimony
    assert!(exposure.public_record.contains("Eli"),
        "exposure ending with loyalty active should reference Eli's testimony");

    // Burden ending: Galen carries alone
    assert!(burden.private_burden.contains("alone") || burden.private_burden.contains("weight"));
}

/// The testament scene shows different text per axis.
#[test]
fn testament_reflects_choice() {
    let axes = ["justice", "exposure", "peace", "burden"];

    for (choice_idx, expected_axis) in axes.iter().enumerate() {
        let (_dir, mut store) = ch15_store();

        run_scene(&mut store, "sm_return", 0);
        run_scene(&mut store, "sm_voss_confrontation", 0);
        run_scene(&mut store, "sm_final_choice", choice_idx);

        assert_eq!(
            store.state().flags.get("ending_axis"),
            Some(&FlagValue::Text(expected_axis.to_string())),
        );

        // Testament should have axis-specific text
        let testament = saints_mile_again::get_scene("sm_testament").unwrap();
        let lines = SceneRunner::filter_lines(&testament, &store);

        let has_axis_text = match *expected_axis {
            "justice" => lines.iter().any(|l| l.text.contains("Voss falls")),
            "exposure" => lines.iter().any(|l| l.text.contains("full truth enters")),
            "peace" => lines.iter().any(|l| l.text.contains("retires")),
            "burden" => lines.iter().any(|l| l.text.contains("plaque stands")),
            _ => false,
        };
        assert!(has_axis_text, "{} ending should have its own testament text", expected_axis);
    }
}

// ─── Each Ally Contributes ─────────────────────────────────────────

/// Every returning ally speaks distinct final truth in the choice scene.
#[test]
fn each_ally_contributes_distinct_final_truth() {
    let (_dir, store) = ch15_store();

    let scene = saints_mile_again::get_scene("sm_final_choice").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);

    // Ada: body and consequence
    assert!(lines.iter().any(|l| l.speaker == "ada" && l.text.contains("anatomy")),
        "Ada should speak about bodies and documented wounds");

    // Rosa: land and blood
    assert!(lines.iter().any(|l| l.speaker == "rosa" && l.text.contains("land")),
        "Rosa should speak about land and mission grants");

    // Miriam: public moral frame
    assert!(lines.iter().any(|l| l.speaker == "miriam" && l.text.contains("rooms")),
        "Miriam should speak about the rooms she held open");

    // Eli: system and kept truth
    assert!(lines.iter().any(|l| l.speaker == "eli" && l.text.contains("ledger")),
        "Eli should speak about the ledger he carried for twenty years");
}

// ─── Bell Remains Unresolved ───────────────────────────────────────

/// The bell is never explained. Not in the final chapter. Not ever.
#[test]
fn bell_remains_unresolved() {
    let (_dir, mut store) = ch15_store();

    run_scene(&mut store, "sm_return", 0);
    run_scene(&mut store, "sm_voss_confrontation", 0);
    run_scene(&mut store, "sm_final_choice", 0); // any axis

    let testament = saints_mile_again::get_scene("sm_testament").unwrap();
    let lines = SceneRunner::filter_lines(&testament, &store);

    // The bell is mentioned
    assert!(lines.iter().any(|l| l.text.contains("bell")),
        "the bell should be present in the testament");

    // But NEVER explained
    assert!(!lines.iter().any(|l|
        l.text.contains("the bell was") ||
        l.text.contains("bell is just") ||
        l.text.contains("bell is supernatural") ||
        l.text.contains("truth is that the bell")
    ), "the bell must NEVER be explained — not even at the end");

    // "May or may not ring" — ambiguity preserved
    assert!(lines.iter().any(|l| l.text.contains("may or may not")),
        "the bell's ambiguity must be preserved to the last scene");
}

// ─── Ending Reads Run State ────────────────────────────────────────

/// The ending should reflect the specific run, not generic completion.
#[test]
fn ending_reads_run_state() {
    // Run with Eli loyalty
    let run_a = EndingRunState {
        eli_loyalty_active: true,
        hand_wounded: true,
        relay_branch: "tom".to_string(),
        tom_alive: true,
        nella_alive: false,
        party_reassembled: true,
        lucien_present: true,
        public_truth_level: 70,
    };

    // Run without Eli loyalty (hypothetical)
    let run_b = EndingRunState {
        eli_loyalty_active: false,
        hand_wounded: true,
        relay_branch: "nella".to_string(),
        tom_alive: false,
        nella_alive: true,
        party_reassembled: true,
        lucien_present: false,
        public_truth_level: 40,
    };

    let ending_a = ending::resolve_ending(EndingAxis::Exposure, &run_a);
    let ending_b = ending::resolve_ending(EndingAxis::Exposure, &run_b);

    // Same axis, different runs, different public records
    assert_ne!(ending_a.public_record, ending_b.public_record,
        "same ending axis should produce different text based on run state");
}

// ─── Full Chapter Path ─────────────────────────────────────────────

/// Complete Chapter 15 — the game ends.
#[test]
fn chapter_15_the_game_ends() {
    let (_dir, mut store) = ch15_store();

    run_scene(&mut store, "sm_return", 0);
    assert_eq!(store.state().flags.get("ch15_started"), Some(&FlagValue::Bool(true)));

    run_scene(&mut store, "sm_voss_confrontation", 0);
    run_scene(&mut store, "sm_final_choice", 1); // exposure

    let testament = saints_mile_again::get_scene("sm_testament").unwrap();
    SceneRunner::apply_scene_effects(&testament, &mut store);

    assert_eq!(store.state().flags.get("ch15_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("game_complete"), Some(&FlagValue::Bool(true)));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "saints_mile_testament"));

    // Save the completed game
    let path = store.save("game_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("game_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(loaded.state().flags.get("ending_axis"), Some(&FlagValue::Text("exposure".to_string())));
}
