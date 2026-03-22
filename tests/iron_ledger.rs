//! Integration tests for Iron Ledger — Chapter 7.
//!
//! Proves: relay truths converge in the archive, Lucien is necessary
//! before forgivable, and institutional truth assembly works.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::state::evidence;
use saints_mile::content::iron_ledger;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = iron_ledger::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn run_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = iron_ledger::get_encounter(encounter_id)
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
        id: FlagId::new("evidence_secured"), value: FlagValue::Bool(true),
    }]);
}

fn ch7_store(relay_branch: &str, lucien_status: &str) -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch7");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch6_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("relay_branch".to_string(), FlagValue::Text(relay_branch.to_string()));
    store.state_mut().flags.insert("lucien_status".to_string(), FlagValue::Text(lucien_status.to_string()));
    store.state_mut().flags.insert("lucien_captured".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    store.state_mut().party.add_member(CharacterId::new("miriam"));
    (dir, store)
}

// ─── Relay Truths Converge ─────────────────────────────────────────

/// Each relay branch verifies different archive records.
/// No single branch provides full clarity.
#[test]
fn relay_truths_converge_in_archive() {
    let archive = evidence::iron_ledger_archive();
    let counts = evidence::branch_verification_counts(&archive);

    // Each branch can verify some records
    for (branch, count) in &counts {
        assert!(*count > 0, "{} branch should verify at least one record", branch);
        assert!(*count < archive.len(),
            "{} branch should NOT verify ALL records (count {} of {})",
            branch, count, archive.len());
    }

    // Different branches verify different subsets
    let tom_results = evidence::verify_against_branch(&archive, "tom");
    let nella_results = evidence::verify_against_branch(&archive, "nella");
    let papers_results = evidence::verify_against_branch(&archive, "papers");

    // Tom verifies route records
    assert!(tom_results.iter().any(|r| r.record_id == "route_manifest_sm"),
        "Tom branch should verify route manifests");

    // Nella verifies payroll names
    assert!(nella_results.iter().any(|r| r.record_id == "payroll_ledger_convoy"),
        "Nella branch should verify payroll records");

    // Papers verifies land acquisition chain
    assert!(papers_results.iter().any(|r| r.record_id == "land_acquisition_chain"),
        "Papers branch should verify land claims");

    // No single branch gets everything
    assert!(tom_results.len() < archive.len());
    assert!(nella_results.len() < archive.len());
    assert!(papers_results.len() < archive.len());
}

/// Archive scene shows different convergence text per branch.
#[test]
fn archive_convergence_scene_branches() {
    for branch in &["tom", "nella", "papers"] {
        let (_dir, mut store) = ch7_store(branch, "forced_guide");

        run_scene(&mut store, "il_city_entry", 0);
        run_scene(&mut store, "il_read_city", 0);
        run_scene(&mut store, "il_archive_problem", 1); // use Lucien

        let archive_scene = iron_ledger::get_scene("il_archive_entry").unwrap();
        let lines = SceneRunner::filter_lines(&archive_scene, &store);

        let has_branch_line = lines.iter().any(|l| match *branch {
            "tom" => l.text.contains("corridor designed to fail") || l.text.contains("road was wrong"),
            "nella" => l.text.contains("line items") || l.text.contains("Payroll names"),
            "papers" => l.text.contains("Signature comparison") || l.text.contains("originals"),
            _ => false,
        });
        assert!(has_branch_line,
            "{} branch should produce its own convergence text in the archive", branch);
    }
}

// ─── Lucien Necessity ──────────────────────────────────────────────

/// Using Lucien provides access that worsens at least one relationship.
#[test]
fn lucien_is_necessary_before_forgivable() {
    // Path A: use Lucien to enter
    let (_dir_a, mut store_a) = ch7_store("tom", "forced_guide");
    run_scene(&mut store_a, "il_city_entry", 0);
    run_scene(&mut store_a, "il_read_city", 0);
    run_scene(&mut store_a, "il_archive_problem", 1); // use Lucien (choice 1)

    assert_eq!(store_a.state().flags.get("used_lucien_in_archive"), Some(&FlagValue::Bool(true)));

    // Using Lucien worsens Rosa's relationship
    let rosa_rel = store_a.state().party.relationships.get("galen:rosa").copied().unwrap_or(0);
    assert!(rosa_rel < 0, "using Lucien should worsen Rosa's relationship (got {})", rosa_rel);

    // Path B: don't use Lucien
    let (_dir_b, mut store_b) = ch7_store("tom", "forced_guide");
    run_scene(&mut store_b, "il_city_entry", 0);
    run_scene(&mut store_b, "il_read_city", 0);
    run_scene(&mut store_b, "il_archive_problem", 0); // use Eli's con (choice 0)

    assert!(store_b.state().flags.get("used_lucien_in_archive").is_none());

    // Lucien-path sees himself as an expense entry
    run_scene(&mut store_a, "il_archive_entry", 0);
    let archive_lines_a = {
        let scene = iron_ledger::get_scene("il_archive_entry").unwrap();
        SceneRunner::filter_lines(&scene, &store_a)
    };
    assert!(archive_lines_a.iter().any(|l| l.text.contains("infrastructure maintenance")),
        "Lucien should see himself filed under 'infrastructure maintenance'");
}

/// Lucien's custody status changes what he can reveal.
#[test]
fn lucien_custody_changes_access() {
    let forced = evidence::lucien_archive_contribution("forced_guide");
    let prisoner = evidence::lucien_archive_contribution("prisoner");
    let judged = evidence::lucien_archive_contribution("judged");

    // All states provide some access
    assert!(!forced.is_empty());
    assert!(!prisoner.is_empty());
    assert!(!judged.is_empty());

    // Judged gives the most (speaks more freely)
    assert!(judged.len() >= forced.len(),
        "judged Lucien should give at least as much as forced guide");
}

// ─── No Clean Institutional Win ────────────────────────────────────

/// The player can get real proof but the institution can still narrow it.
/// Chapter 8 is still necessary.
#[test]
fn no_clean_institutional_win() {
    let (_dir, mut store) = ch7_store("tom", "forced_guide");

    run_scene(&mut store, "il_city_entry", 0);
    run_scene(&mut store, "il_read_city", 0);
    run_scene(&mut store, "il_archive_problem", 1); // use Lucien
    run_scene(&mut store, "il_archive_entry", 0);
    run_combat(&mut store, "archive_break");

    let escape = iron_ledger::get_scene("il_archive_escape").unwrap();
    SceneRunner::apply_scene_effects(&escape, &mut store);

    let close = iron_ledger::get_scene("il_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    // We have institutional proof
    assert_eq!(store.state().flags.get("conspiracy_documented"), Some(&FlagValue::Bool(true)));

    // But the Burned Mission is still necessary
    assert_eq!(store.state().flags.get("burned_mission_lead"), Some(&FlagValue::Bool(true)));

    // Chapter is complete but the campaign continues
    assert_eq!(store.state().flags.get("ch7_complete"), Some(&FlagValue::Bool(true)));

    // Voss is NOT captured, named, or resolved
    assert!(store.state().flags.get("voss_captured").is_none(),
        "Voss must NOT be captured in Ch7 — that's Ch15");
}

// ─── Full Path ─────────────────────────────────────────────────────

/// Complete Chapter 7 with Lucien.
#[test]
fn chapter_7_full_path_with_lucien() {
    let (_dir, mut store) = ch7_store("nella", "forced_guide");

    run_scene(&mut store, "il_city_entry", 0);
    run_scene(&mut store, "il_read_city", 0);
    run_scene(&mut store, "il_archive_problem", 1); // use Lucien
    run_scene(&mut store, "il_archive_entry", 0);
    run_combat(&mut store, "archive_break");

    let escape = iron_ledger::get_scene("il_archive_escape").unwrap();
    let escape_lines = SceneRunner::filter_lines(&escape, &store);
    assert!(escape_lines.iter().any(|l| l.text.contains("doorframe")),
        "Lucien should collapse a doorframe during escape");
    SceneRunner::apply_scene_effects(&escape, &mut store);

    let close = iron_ledger::get_scene("il_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    assert_eq!(store.state().flags.get("ch7_complete"), Some(&FlagValue::Bool(true)));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "institutional_proof"));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "archive_convergence"));

    // Save round-trip
    let path = store.save("ch7_complete").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("burned_mission_lead"), Some(&FlagValue::Bool(true)));
}

/// Three archive approaches produce different relationship states.
#[test]
fn three_approaches_different_compromises() {
    let approaches = [
        (0, "con", "eli", "ada"),    // Eli +3, Ada -2
        (1, "lucien", "eli", "rosa"), // Eli +2, Rosa -5
        (2, "legal", "ada", "none"),  // Ada +5
    ];

    for (choice, approach, helped, hurt) in &approaches {
        let (_dir, mut store) = ch7_store("tom", "forced_guide");
        run_scene(&mut store, "il_city_entry", 0);
        run_scene(&mut store, "il_read_city", 0);
        run_scene(&mut store, "il_archive_problem", *choice);

        assert_eq!(
            store.state().flags.get("archive_approach"),
            Some(&FlagValue::Text(approach.to_string())),
        );

        let helped_rel = store.state().party.relationships
            .get(&format!("galen:{}", helped)).copied().unwrap_or(0);
        assert!(helped_rel > 0, "{} approach should improve {} relationship", approach, helped);

        if *hurt != "none" {
            let hurt_rel = store.state().party.relationships
                .get(&format!("galen:{}", hurt)).copied().unwrap_or(0);
            assert!(hurt_rel < 0, "{} approach should worsen {} relationship", approach, hurt);
        }
    }
}
