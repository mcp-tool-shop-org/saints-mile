//! Integration tests for Dust Revival — Chapter 5.
//!
//! Proves: crowd pressure is a real combat language, Miriam changes
//! the room, and the party's civic disagreements matter.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::combat::crowd::{CrowdState, CrowdAction, CrowdActionType, CrowdPhase, Ringleader};
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::content::dust_revival;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = dust_revival::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn run_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = dust_revival::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));
    // Use first 4 from the roster for the active slots
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
        id: FlagId::new("aftermath_survived"), value: FlagValue::Bool(true),
    }]);
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

/// Full Chapter 5 path: arrival → read → Dunnicks → sermon →
/// crowd breaks → crowd fight → aftermath guns → Miriam joins.
#[test]
fn chapter_5_full_path_miriam_speaks() {
    let (_dir, mut store) = ch5_store();

    // Arrival
    run_scene(&mut store, "dr_arrival", 0);
    assert_eq!(store.state().flags.get("ch5_started"), Some(&FlagValue::Bool(true)));

    // Read revival
    run_scene(&mut store, "dr_read_revival", 0); // visit Dunnick camp

    // Dunnick camp
    run_scene(&mut store, "dr_dunnick_camp", 0);
    assert_eq!(store.state().flags.get("visited_dunnicks"), Some(&FlagValue::Bool(true)));

    // Sermon
    run_scene(&mut store, "dr_sermon", 0);

    // Crowd breaks — let Miriam speak (choice 0)
    run_scene(&mut store, "dr_crowd_breaks", 0);
    assert_eq!(store.state().flags.get("ch5_stance"), Some(&FlagValue::Text("miriam_speaks".to_string())));
    let miriam_rel = store.state().party.relationships.get("galen:miriam").copied().unwrap_or(0);
    assert!(miriam_rel > 0, "trusting Miriam should build relationship");

    // Crowd fight setup — Miriam joins party here
    run_scene(&mut store, "dr_crowd_fight_setup", 0);

    // Miriam should be in the party now
    assert!(store.state().party.has_member(&CharacterId::new("miriam")));

    // Crowd containment (simplified — using standard combat for the encounter shell)
    run_combat(&mut store, "crowd_containment");

    // Aftermath intro
    run_scene(&mut store, "dr_aftermath_intro", 0);

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

    run_scene(&mut store, "dr_arrival", 0);
    run_scene(&mut store, "dr_read_revival", 1); // skip Dunnicks, go to sermon
    run_scene(&mut store, "dr_sermon", 0);

    // Force hold (choice 1)
    run_scene(&mut store, "dr_crowd_breaks", 1);
    assert_eq!(store.state().flags.get("ch5_stance"), Some(&FlagValue::Text("force_hold".to_string())));

    // Miriam relationship should be negative
    let miriam_rel = store.state().party.relationships.get("galen:miriam").copied().unwrap_or(0);
    assert!(miriam_rel < 0, "overriding Miriam should lower relationship");

    // Run through fights
    run_scene(&mut store, "dr_crowd_fight_setup", 0);
    run_combat(&mut store, "crowd_containment");
    run_scene(&mut store, "dr_aftermath_intro", 0);
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
