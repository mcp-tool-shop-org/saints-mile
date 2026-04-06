//! Integration tests for the Morrow Crossing prologue.
//!
//! These tests prove the biography loop: scene state feeds combat,
//! combat result feeds social state, social state feeds memory.

use saints_mile::types::{
    CharacterId, SkillId, FlagValue, ReputationAxis, AgePhase,
};
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, CombatSide, CombatAction, TargetSelection};
use saints_mile::state::store::StateStore;
use saints_mile::content::prologue;
use tempfile::TempDir;

/// Helper: run a scene, apply its effects, execute a choice by index.
fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = prologue::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));

    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);

    // Apply scene-level effects
    SceneRunner::apply_scene_effects(&scene, store);

    // Execute choice
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));

    chosen.transition
}

/// Helper: run a combat encounter to victory.
fn run_combat_to_victory(store: &mut StateStore, encounter_id: &str) {
    let encounter = prologue::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));

    let mut combat = EncounterState::new(&encounter, prologue::prologue_party());

    // Resolve standoff
    combat.resolve_standoff(StandoffPosture::SteadyHand, None);

    // Fight until victory
    for _ in 0..30 {
        combat.build_turn_queue();

        loop {
            let entry = combat.current_turn_entry().cloned();
            if entry.is_none() { break; }
            let entry = entry.unwrap();

            let action = match entry.side {
                CombatSide::Party => {
                    let target = combat.enemies.iter()
                        .find(|e| !e.down && !e.panicked)
                        .map(|e| e.id.clone())
                        .unwrap_or_default();
                    CombatAction::UseSkill {
                        skill: SkillId::new("quick_draw"),
                        target: TargetSelection::Single(target),
                    }
                }
                CombatSide::Enemy => {
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single("galen".to_string()),
                    }
                }
                CombatSide::NpcAlly => {
                    CombatAction::Defend
                }
            };

            combat.execute_action(&action);
            combat.evaluate_objectives();

            if combat.check_resolution().is_some() {
                // Apply victory effects to game state
                if let Some(outcome) = &combat.outcome {
                    store.apply_effects(&outcome.effects);
                }
                return;
            }

            if !combat.advance_turn() { break; }
        }
    }

    assert!(false, "combat must resolve within 30 rounds");
}

// ─── Golden Path ───────────────────────────────────────────────────

/// Golden path: tear poster → ride together → side with law → push hard →
/// fight → camp → ride to town → return with changed eyes.
///
/// Tests: poster choice applied, scene transitions clean, standoff resolves,
/// combat resolves, survival flag written, return scene reflects outcome.
#[test]
fn golden_path_town_direct() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // P2 — Poster scene: tear it down
    let next = run_scene(&mut store, "prologue_poster", 0);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "eli_intro"));
    assert_eq!(store.state().flags.get("tore_poster"), Some(&FlagValue::Bool(true)));
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "wanted_poster"));

    // P3 — Eli intro: ride together
    let next = run_scene(&mut store, "eli_intro", 0);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "morrow_square"));
    assert!(store.state().party.has_member(&CharacterId::new("eli")));
    // Eli relationship is positive (rode together).
    // 5 = max trust gain from the Eli intro scene — riding together
    // is the strongest possible first impression in the prologue.
    assert_eq!(*store.state().party.relationships.get("galen:eli").unwrap(), 5);

    // P5 — Square: side with the deputy (law)
    let next = run_scene(&mut store, "morrow_square", 0);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "ride_to_arroyo"));
    assert_eq!(
        store.state().flags.get("square_stance"),
        Some(&FlagValue::Text("law".to_string()))
    );
    assert!(store.state().reputation.get(ReputationAxis::TownLaw) > 0);

    // P6 — Ride to arroyo: push hard
    let next = run_scene(&mut store, "ride_to_arroyo", 0);
    assert!(matches!(next, SceneTransition::Combat(ref id) if id.0 == "glass_arroyo"));
    // Resources depleted
    assert!(store.state().resources.horse_stamina < 100);
    assert!(store.state().resources.water < 100);

    // P7 — Glass Arroyo fight
    run_combat_to_victory(&mut store, "glass_arroyo");
    assert_eq!(
        store.state().flags.get("arroyo_survived"),
        Some(&FlagValue::Bool(true))
    );

    // P8 — Campfire choice: ride straight to town
    let next = run_scene(&mut store, "campfire_choice", 0);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "return_town_direct"));
    assert_eq!(
        store.state().flags.get("beat5_choice"),
        Some(&FlagValue::Text("town_direct".to_string()))
    );
    assert_eq!(store.state().flags.get("eli_confession"), Some(&FlagValue::Bool(true)));

    // P9 — Return: town direct
    let scene = prologue::get_scene("return_town_direct").unwrap();
    let prepared = SceneRunner::prepare_scene(&scene, &store);
    assert!(prepared.should_play);

    // The return scene should show the law-aligned Alma line
    // (bitter because he sided with law, then went straight back)
    let alma_line = prepared.lines.iter()
        .find(|l| l.speaker == "alma");
    assert!(alma_line.is_some(), "Alma should appear in the return");

    // Apply return effects
    SceneRunner::apply_scene_effects(&scene, &mut store);

    // Reputation reflects the full journey
    assert!(store.state().reputation.get(ReputationAxis::TownLaw) > 0,
        "law rep should be positive after siding with deputy + riding direct");
    assert!(store.state().reputation.get(ReputationAxis::Rancher) < 0,
        "rancher rep should be negative after leaving homestead");

    // Prologue complete
    assert_eq!(
        store.state().flags.get("prologue_complete"),
        Some(&FlagValue::Bool(true))
    );

    // Save survives round-trip
    let path = store.save("golden_path").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("prologue_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(loaded.state().flags.get("square_stance"), Some(&FlagValue::Text("law".to_string())));
    assert_eq!(loaded.state().flags.get("beat5_choice"), Some(&FlagValue::Text("town_direct".to_string())));
}

// ─── Divergent Path ────────────────────────────────────────────────

/// Divergent path: leave poster → keep distance → side with ranchers →
/// pace yourself → fight → camp → divert to homestead → return with different eyes.
///
/// Tests: different poster choice, different stance, different return dialogue,
/// same architecture, different biography.
#[test]
fn divergent_path_homestead_first() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // P2 — Poster: leave it up
    let next = run_scene(&mut store, "prologue_poster", 1);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "eli_intro"));
    assert_eq!(store.state().flags.get("left_poster"), Some(&FlagValue::Bool(true)));
    assert!(store.state().flags.get("tore_poster").is_none());

    // P3 — Eli intro: keep distance
    let next = run_scene(&mut store, "eli_intro", 1);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "morrow_square"));
    assert!(store.state().party.has_member(&CharacterId::new("eli")));
    // Eli relationship is negative (kept distance)
    assert_eq!(*store.state().party.relationships.get("galen:eli").unwrap(), -2);

    // Eli intro should have shown the "left poster" branch line
    let scene = prologue::get_scene("eli_intro").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);
    let left_poster_line = lines.iter()
        .find(|l| l.text.contains("Left the poster up"));
    assert!(left_poster_line.is_some(), "Eli should comment on leaving the poster");

    // P5 — Square: side with ranchers
    let next = run_scene(&mut store, "morrow_square", 1);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "ride_to_arroyo"));
    assert_eq!(
        store.state().flags.get("square_stance"),
        Some(&FlagValue::Text("rancher".to_string()))
    );
    assert!(store.state().reputation.get(ReputationAxis::Rancher) > 0);
    assert!(store.state().reputation.get(ReputationAxis::Railroad) < 0);

    // P6 — Ride: pace yourself
    let next = run_scene(&mut store, "ride_to_arroyo", 1);
    assert!(matches!(next, SceneTransition::Combat(_)));
    // Less horse drain but more water cost
    assert_eq!(store.state().resources.horse_stamina, 80);
    assert_eq!(store.state().resources.water, 70);

    // P7 — Fight
    run_combat_to_victory(&mut store, "glass_arroyo");
    assert_eq!(
        store.state().flags.get("arroyo_survived"),
        Some(&FlagValue::Bool(true))
    );

    // P8 — Campfire: divert to homestead
    let next = run_scene(&mut store, "campfire_choice", 1);
    assert!(matches!(next, SceneTransition::Scene(ref id) if id.0 == "return_homestead"));
    assert_eq!(
        store.state().flags.get("beat5_choice"),
        Some(&FlagValue::Text("homestead_first".to_string()))
    );

    // P9 — Return: homestead path
    let scene = prologue::get_scene("return_homestead").unwrap();
    let prepared = SceneRunner::prepare_scene(&scene, &store);
    assert!(prepared.should_play);

    // Should show Alma's grateful line (sided with ranchers + went to homestead)
    let alma_line = prepared.lines.iter()
        .find(|l| l.text.contains("alive because of you"));
    assert!(alma_line.is_some(),
        "Alma should be grateful (rancher stance + homestead choice)");

    // Apply return effects
    SceneRunner::apply_scene_effects(&scene, &mut store);

    // Reputation reflects the divergent journey
    assert!(store.state().reputation.get(ReputationAxis::Rancher) > 0,
        "rancher rep should be positive after siding with them + homestead");
    assert!(store.state().reputation.get(ReputationAxis::TownLaw) < 0,
        "law rep should be negative after choosing homestead over town");

    // Prologue complete
    assert_eq!(
        store.state().flags.get("prologue_complete"),
        Some(&FlagValue::Bool(true))
    );
}

// ─── Consequence Proof ─────────────────────────────────────────────

/// Prove the two paths produce meaningfully different biographies
/// in the same state architecture.
#[test]
fn two_paths_different_biographies() {
    let dir = TempDir::new().unwrap();

    // Run golden path
    let mut store_a = StateStore::new_game(dir.path().join("a"));
    run_scene(&mut store_a, "prologue_poster", 0); // tear
    run_scene(&mut store_a, "eli_intro", 0);        // ride together
    run_scene(&mut store_a, "morrow_square", 0);    // law
    run_scene(&mut store_a, "ride_to_arroyo", 0);   // push hard
    run_combat_to_victory(&mut store_a, "glass_arroyo");
    run_scene(&mut store_a, "campfire_choice", 0);  // town direct
    let scene_a = prologue::get_scene("return_town_direct").unwrap();
    SceneRunner::apply_scene_effects(&scene_a, &mut store_a);

    // Run divergent path
    let mut store_b = StateStore::new_game(dir.path().join("b"));
    run_scene(&mut store_b, "prologue_poster", 1); // leave
    run_scene(&mut store_b, "eli_intro", 1);        // keep distance
    run_scene(&mut store_b, "morrow_square", 1);    // rancher
    run_scene(&mut store_b, "ride_to_arroyo", 1);   // pace
    run_combat_to_victory(&mut store_b, "glass_arroyo");
    run_scene(&mut store_b, "campfire_choice", 1);  // homestead
    let scene_b = prologue::get_scene("return_homestead").unwrap();
    SceneRunner::apply_scene_effects(&scene_b, &mut store_b);

    // ─── Same architecture, different biography ───

    // Both completed the prologue
    assert_eq!(store_a.state().flags.get("prologue_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store_b.state().flags.get("prologue_complete"), Some(&FlagValue::Bool(true)));

    // Different poster choices
    assert_eq!(store_a.state().flags.get("tore_poster"), Some(&FlagValue::Bool(true)));
    assert!(store_a.state().flags.get("left_poster").is_none());
    assert!(store_b.state().flags.get("tore_poster").is_none());
    assert_eq!(store_b.state().flags.get("left_poster"), Some(&FlagValue::Bool(true)));

    // Different square stances
    assert_eq!(store_a.state().flags.get("square_stance"), Some(&FlagValue::Text("law".to_string())));
    assert_eq!(store_b.state().flags.get("square_stance"), Some(&FlagValue::Text("rancher".to_string())));

    // Different Beat 5 choices
    assert_eq!(store_a.state().flags.get("beat5_choice"), Some(&FlagValue::Text("town_direct".to_string())));
    assert_eq!(store_b.state().flags.get("beat5_choice"), Some(&FlagValue::Text("homestead_first".to_string())));

    // Opposite reputation profiles
    assert!(store_a.state().reputation.get(ReputationAxis::TownLaw) > 0);
    assert!(store_a.state().reputation.get(ReputationAxis::Rancher) < 0);
    assert!(store_b.state().reputation.get(ReputationAxis::TownLaw) < 0);
    assert!(store_b.state().reputation.get(ReputationAxis::Rancher) > 0);

    // Different Eli relationships
    assert_eq!(*store_a.state().party.relationships.get("galen:eli").unwrap(), 5);
    assert_eq!(*store_b.state().party.relationships.get("galen:eli").unwrap(), -2);

    // Both have the wanted poster as a memory object
    assert!(store_a.state().memory_objects.iter().any(|o| o.id.0 == "wanted_poster"));
    assert!(store_b.state().memory_objects.iter().any(|o| o.id.0 == "wanted_poster"));

    // Both survived the arroyo
    assert_eq!(store_a.state().flags.get("arroyo_survived"), Some(&FlagValue::Bool(true)));
    assert_eq!(store_b.state().flags.get("arroyo_survived"), Some(&FlagValue::Bool(true)));

    // Both have Eli's confession
    assert_eq!(store_a.state().flags.get("eli_confession"), Some(&FlagValue::Bool(true)));
    assert_eq!(store_b.state().flags.get("eli_confession"), Some(&FlagValue::Bool(true)));

    // Different return scenes would fire
    let return_a = prologue::get_scene("return_town_direct").unwrap();
    let return_b = prologue::get_scene("return_homestead").unwrap();
    let lines_a = SceneRunner::filter_lines(&return_a, &store_a);
    let lines_b = SceneRunner::filter_lines(&return_b, &store_b);

    // Return A mentions the town being stabilized
    assert!(lines_a.iter().any(|l| l.text.contains("prevented a massacre")));
    // Return B mentions homestead surviving
    assert!(lines_b.iter().any(|l| l.text.contains("ranchers speak your name")));
}
