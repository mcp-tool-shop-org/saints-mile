//! Integration tests for Dust Revival — Chapter 5.
//!
//! Proves: crowd pressure is a real combat language, Miriam changes
//! the room, and the party's civic disagreements matter.

mod common;

use saints_mile::types::*;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::engine::EncounterState;
use saints_mile::combat::crowd::{CrowdState, CrowdAction, CrowdActionType, CrowdPhase, Ringleader};
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::content::dust_revival;
use tempfile::TempDir;

const CHAPTER: &str = "dust_revival";

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> saints_mile::scene::types::SceneTransition {
    common::run_scene(store, CHAPTER, scene_id, choice_index)
}

fn run_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = dust_revival::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));
    // Use first 4 from the roster for the active slots
    let party: Vec<_> = party_defs::ch5_roster().into_iter().take(4).collect();
    let mut combat = EncounterState::new(&encounter, party);
    let (resolved, _rounds) = common::run_combat(&mut combat, store);
    if !resolved {
        store.apply_effects(&[saints_mile::scene::types::StateEffect::SetFlag {
            id: FlagId::new("aftermath_survived"), value: FlagValue::Bool(true),
        }]);
    }
}

fn ch5_store() -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch5");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch4_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("rosa_joined".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().party.add_member(CharacterId::new("rosa"));
    (dir, store)
}

// ─── Crowd Fight Feels Unique ──────────────────────────────────────

/// The crowd encounter is NOT reducible to standard enemy-HP victory.
#[test]
fn crowd_fight_feels_unique() {
    let mut crowd = CrowdState::new(50, 6, vec![
        Ringleader { id: "loud".to_string(), name: "Loud Man".to_string(), nerve: 12, influence: 8, broken: false },
        Ringleader { id: "angry".to_string(), name: "Angry Woman".to_string(), nerve: 15, influence: 10, broken: false },
    ]);

    // Multiple party members alter the same pressure bar differently
    let miriam_result = crowd.execute_action(&CrowdAction {
        actor: "miriam".to_string(),
        action_type: CrowdActionType::BroadCalm,
        target: None,
    });
    assert!(miriam_result.nerve_change > 0, "Miriam calms the crowd");

    let rosa_result = crowd.execute_action(&CrowdAction {
        actor: "rosa".to_string(),
        action_type: CrowdActionType::PhysicalHold,
        target: None,
    });
    assert!(rosa_result.surge_delayed, "Rosa delays the surge physically");

    let eli_result = crowd.execute_action(&CrowdAction {
        actor: "eli".to_string(),
        action_type: CrowdActionType::TargetedNerve,
        target: Some("loud".to_string()),
    });
    // Eli targets individuals, not the collective
    assert!(eli_result.description.contains("rattles") || eli_result.description.contains("breaks"));

    let ada_result = crowd.execute_action(&CrowdAction {
        actor: "ada".to_string(),
        action_type: CrowdActionType::MedicalAuthority,
        target: None,
    });
    assert!(ada_result.nerve_change > 0, "Ada defuses with evidence");

    // Resolution path preserves the room without "winning" conventionally
    crowd.collective_nerve = crowd.max_nerve;
    crowd.momentum = 5;
    let phase = crowd.advance();
    assert_eq!(phase, CrowdPhase::Dispersed, "crowd can disperse without kills");
}

// ─── Miriam Changes the Room ───────────────────────────────────────

/// Without Miriam, the room escalates faster.
/// With Miriam, containment options exist that no one else can generate.
#[test]
fn miriam_changes_the_room() {
    // Without Miriam: only Galen shock + Rosa hold + Ada evidence
    let mut crowd_no_miriam = CrowdState::new(40, 4, vec![
        Ringleader { id: "loud".to_string(), name: "Loud Man".to_string(), nerve: 12, influence: 8, broken: false },
    ]);

    // Three turns without Miriam
    for _ in 0..3 {
        crowd_no_miriam.execute_action(&CrowdAction {
            actor: "galen".to_string(),
            action_type: CrowdActionType::ShockAction,
            target: None,
        });
        crowd_no_miriam.execute_action(&CrowdAction {
            actor: "rosa".to_string(),
            action_type: CrowdActionType::PhysicalHold,
            target: None,
        });
        crowd_no_miriam.advance();
    }
    let nerve_without = crowd_no_miriam.collective_nerve;

    // With Miriam: add BroadCalm to the mix
    let mut crowd_with_miriam = CrowdState::new(40, 4, vec![
        Ringleader { id: "loud".to_string(), name: "Loud Man".to_string(), nerve: 12, influence: 8, broken: false },
    ]);

    for _ in 0..3 {
        crowd_with_miriam.execute_action(&CrowdAction {
            actor: "miriam".to_string(),
            action_type: CrowdActionType::BroadCalm,
            target: None,
        });
        crowd_with_miriam.execute_action(&CrowdAction {
            actor: "rosa".to_string(),
            action_type: CrowdActionType::PhysicalHold,
            target: None,
        });
        crowd_with_miriam.advance();
    }
    let nerve_with = crowd_with_miriam.collective_nerve;

    assert!(nerve_with > nerve_without,
        "Miriam should keep nerve higher than shock alone ({} vs {})",
        nerve_with, nerve_without);

    // Miriam's Rebuke can break a ringleader that Galen's shock cannot
    let mut crowd_rebuke = CrowdState::new(40, 4, vec![
        Ringleader { id: "loud".to_string(), name: "Loud Man".to_string(), nerve: 12, influence: 8, broken: false },
    ]);

    let result = crowd_rebuke.execute_action(&CrowdAction {
        actor: "miriam".to_string(),
        action_type: CrowdActionType::Rebuke,
        target: Some("loud".to_string()),
    });

    // Rebuke hits for 15, nerve was 12 — should break
    assert!(result.ringleader_broken.is_some(),
        "Miriam's Rebuke should break the ringleader");
}

/// Miriam is not just another party member.
#[test]
fn miriam_is_distinct() {
    let miriam = party_defs::miriam();

    // Highest nerve in the entire game
    assert_eq!(miriam.nerve, 35, "Miriam should have the highest nerve");
    assert!(miriam.nerve > party_defs::ada().nerve);
    assert!(miriam.nerve > party_defs::galen(AgePhase::Adult).nerve);

    // Near-unarmed — shotgun as last resort
    assert_eq!(miriam.ammo, 3, "Miriam barely has ammo");
    assert_eq!(miriam.damage, 3, "Miriam's damage is token");

    // Skills are about the room, not about shooting
    assert!(miriam.skills.iter().any(|s| s.0 == "hymn"));
    assert!(miriam.skills.iter().any(|s| s.0 == "sermon"));
    assert!(miriam.skills.iter().any(|s| s.0 == "witness"));
    assert!(miriam.skills.iter().any(|s| s.0 == "rebuke"));

    // Her duo techs reflect relationships, not just roles
    assert!(miriam.duo_techs.iter().any(|d| d.0 == "false_confession"),
        "Eli + Miriam: same tools, different reasons");
    assert!(miriam.duo_techs.iter().any(|d| d.0 == "sheltered_fire"),
        "Rosa + Miriam: first time Rosa protects someone who isn't family");
}

// ─── Chapter Path ──────────────────────────────────────────────────

/// Full Chapter 5 path: arrival -> read -> Dunnicks -> sermon ->
/// crowd breaks -> crowd fight -> aftermath guns -> Miriam joins.
#[test]
fn chapter_5_full_path_miriam_speaks() {
    let (_dir, mut store) = ch5_store();

    // Arrival
    run_scene(&mut store, "dr_arrival", 0);              // arrive in town
    assert_eq!(store.state().flags.get("ch5_started"), Some(&FlagValue::Bool(true)));

    // Read revival
    run_scene(&mut store, "dr_read_revival", 0);          // choice 0 = visit Dunnick camp

    // Dunnick camp
    run_scene(&mut store, "dr_dunnick_camp", 0);          // explore camp
    assert_eq!(store.state().flags.get("visited_dunnicks"), Some(&FlagValue::Bool(true)));

    // Sermon
    run_scene(&mut store, "dr_sermon", 0);                // attend sermon

    // Crowd breaks — choice 0 = let Miriam speak
    run_scene(&mut store, "dr_crowd_breaks", 0);
    assert_eq!(store.state().flags.get("ch5_stance"), Some(&FlagValue::Text("miriam_speaks".to_string())));
    let miriam_rel = store.state().party.relationships.get("galen:miriam").copied().unwrap_or(0);
    assert!(miriam_rel > 0, "trusting Miriam should build relationship");

    // Crowd fight setup — Miriam joins party here
    run_scene(&mut store, "dr_crowd_fight_setup", 0);     // prepare for fight

    // Miriam should be in the party now
    assert!(store.state().party.has_member(&CharacterId::new("miriam")));

    // Crowd containment (simplified — using standard combat for the encounter shell)
    run_combat(&mut store, "crowd_containment");

    // Aftermath intro
    run_scene(&mut store, "dr_aftermath_intro", 0);       // aftermath begins

    // Aftermath guns
    run_combat(&mut store, "aftermath_guns");

    // Chapter close — Miriam joins permanently
    let close = dust_revival::get_scene("dr_chapter_close").unwrap();
    let lines = SceneRunner::filter_lines(&close, &store);
    assert!(lines.iter().any(|l| l.text.contains("trusted me to speak")),
        "miriam_speaks stance: she should acknowledge being trusted");
    SceneRunner::apply_scene_effects(&close, &mut store);

    assert_eq!(store.state().flags.get("miriam_joined"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("ch5_complete"), Some(&FlagValue::Bool(true)));
}

/// Force-hold stance produces different Miriam reaction.
#[test]
fn chapter_5_force_hold_stance() {
    let (_dir, mut store) = ch5_store();

    run_scene(&mut store, "dr_arrival", 0);              // arrive in town
    run_scene(&mut store, "dr_read_revival", 1);          // choice 1 = skip Dunnicks, go to sermon
    run_scene(&mut store, "dr_sermon", 0);                // attend sermon

    // choice 1 = force hold (override Miriam)
    run_scene(&mut store, "dr_crowd_breaks", 1);
    assert_eq!(store.state().flags.get("ch5_stance"), Some(&FlagValue::Text("force_hold".to_string())));

    // Miriam relationship should be negative
    let miriam_rel = store.state().party.relationships.get("galen:miriam").copied().unwrap_or(0);
    assert!(miriam_rel < 0, "overriding Miriam should lower relationship");

    // Run through fights
    run_scene(&mut store, "dr_crowd_fight_setup", 0);     // prepare for fight
    run_combat(&mut store, "crowd_containment");
    run_scene(&mut store, "dr_aftermath_intro", 0);       // aftermath begins
    run_combat(&mut store, "aftermath_guns");

    // Chapter close — different Miriam line
    let close = dust_revival::get_scene("dr_chapter_close").unwrap();
    let lines = SceneRunner::filter_lines(&close, &store);
    assert!(lines.iter().any(|l| l.text.contains("overrode the voice")),
        "force_hold stance: Miriam should note being overridden");
    SceneRunner::apply_scene_effects(&close, &mut store);

    // She still joins — measured intervention, not petulance
    assert_eq!(store.state().flags.get("miriam_joined"), Some(&FlagValue::Bool(true)));
}

// ─── Crowd Mechanics Validation ───────────────────────────────────

/// Crowd nerve state reflects actions taken.
#[test]
fn crowd_nerve_state_after_actions() {
    let mut crowd = CrowdState::new(50, 6, vec![
        Ringleader { id: "loud".to_string(), name: "Loud Man".to_string(), nerve: 12, influence: 8, broken: false },
        Ringleader { id: "angry".to_string(), name: "Angry Woman".to_string(), nerve: 15, influence: 10, broken: false },
    ]);

    assert_eq!(crowd.phase, CrowdPhase::Tense, "crowd starts tense");
    assert_eq!(crowd.active_ringleaders(), 2, "both ringleaders start active");

    // Degrade nerve first so BroadCalm has room to restore
    crowd.collective_nerve = 30;
    let initial_nerve = crowd.collective_nerve;

    // Broad calm raises nerve
    crowd.execute_action(&CrowdAction {
        actor: "miriam".to_string(),
        action_type: CrowdActionType::BroadCalm,
        target: None,
    });
    assert!(crowd.collective_nerve > initial_nerve, "BroadCalm should raise nerve");
    assert!(crowd.momentum > 0, "BroadCalm should build positive momentum");
}

/// Ringleader presence and breaking affect crowd mood.
#[test]
fn ringleader_presence_affects_crowd_mood() {
    let mut crowd = CrowdState::new(60, 6, vec![
        Ringleader { id: "loud".to_string(), name: "Loud Man".to_string(), nerve: 10, influence: 15, broken: false },
    ]);

    // Degrade nerve so breaking the ringleader has room to calm
    crowd.collective_nerve = 30;
    let nerve_before = crowd.collective_nerve;

    // Break the ringleader with Rebuke (hits for 15, nerve is 10)
    let result = crowd.execute_action(&CrowdAction {
        actor: "miriam".to_string(),
        action_type: CrowdActionType::Rebuke,
        target: Some("loud".to_string()),
    });

    assert!(result.ringleader_broken.is_some(), "ringleader should break");
    assert_eq!(crowd.active_ringleaders(), 0, "no active ringleaders remain");
    // Breaking a ringleader calms the crowd by their influence amount + bonus
    assert!(crowd.collective_nerve > nerve_before,
        "breaking ringleader should calm crowd: {} vs {}", crowd.collective_nerve, nerve_before);
}

/// Five roster members after Chapter 5.
#[test]
fn five_roster_after_ch5() {
    let roster = party_defs::ch5_roster();
    assert_eq!(roster.len(), 5, "Ch5 roster should have 5 members");

    // Verify identities
    assert_eq!(roster[0].0, "galen");
    assert_eq!(roster[1].0, "eli");
    assert_eq!(roster[2].0, "ada");
    assert_eq!(roster[3].0, "rosa");
    assert_eq!(roster[4].0, "miriam");

    // First swap decision: 5 members, 4 active slots
    // This proves the roster exceeds slot capacity
    assert!(roster.len() > 4, "roster must exceed 4 to force swap decisions");
}
