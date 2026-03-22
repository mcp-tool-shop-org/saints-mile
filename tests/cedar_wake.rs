//! Integration tests for Cedar Wake — Chapter 1.
//!
//! Proves the game's central moral mechanic:
//! the same command means something different when the world changes.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, CombatSide, CombatAction, TargetSelection};
use saints_mile::state::store::StateStore;
use saints_mile::content::{cedar_wake, builders::*};
use tempfile::TempDir;

/// Helper: run a Cedar Wake scene.
fn run_cw_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = cedar_wake::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

/// Helper: run combat to victory for Chapter 1.
fn run_youth_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = cedar_wake::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));
    let mut combat = EncounterState::new(&encounter, cedar_wake::youth_galen());

    // No standoff in youth encounters — go straight to combat
    if combat.phase == saints_mile::combat::engine::EncounterPhase::Standoff {
        combat.resolve_standoff(StandoffPosture::SteadyHand, None);
    } else {
        // Force phase to Combat if no standoff
        combat.phase = saints_mile::combat::engine::EncounterPhase::Combat;
    }

    for _ in 0..30 {
        combat.build_turn_queue();
        if combat.turn_queue.is_empty() { break; }

        loop {
            let entry = combat.current_turn_entry().cloned();
            if entry.is_none() { break; }
            let entry = entry.unwrap();

            let target_id = match entry.side {
                CombatSide::Party | CombatSide::NpcAlly => {
                    combat.enemies.iter()
                        .find(|e| !e.down && !e.panicked)
                        .map(|e| e.id.clone())
                        .unwrap_or_default()
                }
                CombatSide::Enemy => "galen".to_string(),
            };

            if target_id.is_empty() { break; }

            let action = match entry.side {
                CombatSide::Party => CombatAction::UseSkill {
                    skill: SkillId::new("quick_draw"),
                    target: TargetSelection::Single(target_id),
                },
                CombatSide::Enemy => CombatAction::UseSkill {
                    skill: SkillId::new("attack"),
                    target: TargetSelection::Single(target_id),
                },
                CombatSide::NpcAlly => CombatAction::UseSkill {
                    skill: SkillId::new("attack"),
                    target: TargetSelection::Single(target_id),
                },
            };

            combat.execute_action(&action);
            combat.evaluate_objectives();

            if let Some(outcome) = combat.check_resolution() {
                store.apply_effects(&outcome.effects);
                return;
            }

            if !combat.advance_turn() { break; }
        }
    }

    panic!("combat did not resolve within 30 rounds");
}

// ─── Core Path Test ────────────────────────────────────────────────

/// Full Chapter 1: arrival → Molly → Voss → courier → evening →
/// shooting post → horse thief → bandit camp → Bitter Cut → return.
#[test]
fn chapter_1_full_path() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // Set age phase to Youth for Chapter 1
    store.state_mut().age_phase = AgePhase::Youth;
    store.state_mut().chapter = ChapterId::new("ch1");

    // 1A1 — Arrival: look around town
    run_cw_scene(&mut store, "cw_arrival", 0);

    // 1A3 — Mercantile: meet Molly
    run_cw_scene(&mut store, "cw_mercantile", 0);
    assert!(store.state().flags.get("met_molly") == Some(&FlagValue::Bool(true)));

    // Visit livery
    run_cw_scene(&mut store, "cw_livery", 0);
    assert!(store.state().flags.get("met_declan") == Some(&FlagValue::Bool(true)));

    // 1A2 — Voss office
    run_cw_scene(&mut store, "cw_voss_office", 0);

    // 1A4 — First courier: Trail Eye unlocks
    run_cw_scene(&mut store, "cw_first_courier", 0);
    assert!(store.state().party.has_skill(
        &CharacterId::new("galen"),
        &SkillId::new("trail_eye"),
    ));

    // 1A5 — Evening: Molly conversation (conditional lines should show)
    let evening = cedar_wake::get_scene("cw_evening").unwrap();
    let prepared = SceneRunner::prepare_scene(&evening, &store);
    let molly_lines: Vec<_> = prepared.lines.iter()
        .filter(|l| l.speaker == "molly")
        .collect();
    assert!(!molly_lines.is_empty(), "Molly should have lines (met_molly is true)");
    run_cw_scene(&mut store, "cw_evening", 0);

    // 1B1 — Shooting post: Steady Aim unlocks
    run_cw_scene(&mut store, "cw_shooting_post", 0);
    assert!(store.state().party.has_skill(
        &CharacterId::new("galen"),
        &SkillId::new("steady_aim"),
    ));
    assert!(store.state().flags.get("voss_taught_steady_aim") == Some(&FlagValue::Bool(true)));

    // 1B2 — Horse thief
    run_cw_scene(&mut store, "cw_horse_thief_briefing", 0);
    run_youth_combat(&mut store, "horse_thief");
    assert!(store.state().flags.get("horse_thief_caught") == Some(&FlagValue::Bool(true)));

    // Return
    run_cw_scene(&mut store, "cw_horse_thief_return", 0); // continue to bandit briefing

    // 1B7 — Bandit camp: flank approach
    run_cw_scene(&mut store, "cw_bandit_briefing", 0);
    run_youth_combat(&mut store, "bandit_camp");
    assert!(store.state().flags.get("bandit_camp_cleared") == Some(&FlagValue::Bool(true)));

    // Return: "Clean work."
    run_cw_scene(&mut store, "cw_bandit_camp_return", 0);
    assert!(store.state().flags.get("clean_work") == Some(&FlagValue::Bool(true)));

    // 1C1 — Bitter Cut dispatch
    run_cw_scene(&mut store, "cw_bitter_cut_dispatch", 0);

    // 1C2 — Arrival at Bitter Cut
    run_cw_scene(&mut store, "cw_bitter_cut_arrival", 0);

    // 1C3 — Dispatch delivery: hold on workers
    run_cw_scene(&mut store, "cw_bitter_cut_dispatch_delivery", 0);

    // 1C4 — The fight
    run_youth_combat(&mut store, "bitter_cut");
    assert!(store.state().flags.get("bitter_cut_order_maintained") == Some(&FlagValue::Bool(true)));

    // 1C5 — Aftermath: Voss's lesson
    run_cw_scene(&mut store, "cw_bitter_cut_aftermath", 0);

    // 1C7 — Return to Cedar Wake
    let return_scene = cedar_wake::get_scene("cw_bitter_cut_return").unwrap();
    let prepared = SceneRunner::prepare_scene(&return_scene, &store);

    // Molly should appear (met_molly is true)
    let molly_return = prepared.lines.iter()
        .find(|l| l.text.contains("You sure you are"));
    assert!(molly_return.is_some(), "Molly should ask 'You sure you are?'");

    SceneRunner::apply_scene_effects(&return_scene, &mut store);
    assert!(store.state().flags.get("chapter1_complete") == Some(&FlagValue::Bool(true)));
    assert!(store.state().flags.get("bitter_cut_done") == Some(&FlagValue::Bool(true)));
}

// ─── Same Skill, Different Meaning ─────────────────────────────────

/// The game's central moral mechanic in code:
/// Steady Aim at the bandit camp vs Steady Aim at Bitter Cut.
/// Same button. Same engine. Different world.
#[test]
fn same_skill_different_meaning() {
    let dir = TempDir::new().unwrap();

    // ─── Bandit Camp: Steady Aim feels earned ───
    let bandit_encounter = cedar_wake::bandit_camp_encounter();
    let mut bandit_combat = EncounterState::new(&bandit_encounter, cedar_wake::youth_galen());
    bandit_combat.build_turn_queue();

    // Enemies are proper threats
    assert!(bandit_combat.enemies.iter().all(|e| e.hp >= 20),
        "bandits should be real threats");
    assert!(bandit_combat.enemies.iter().any(|e| e.nerve_threshold >= 5),
        "bandits should have real nerve thresholds");

    // The encounter has a clean objective
    assert!(bandit_combat.objectives.iter().any(|o| o.label.contains("Clear")),
        "objective should be about clearing the camp");

    // No civilian casualty objective exists
    assert!(!bandit_combat.objectives.iter().any(|o|
        o.label.contains("civilian") || o.label.contains("casualties")),
        "bandit camp should NOT have civilian casualty objective");

    // NPC allies (Cal) are present and professional
    assert!(!bandit_combat.npc_allies.is_empty(),
        "Cal should be fighting alongside");

    // ─── Bitter Cut: same skills, different targets ───
    let bitter_encounter = cedar_wake::bitter_cut_encounter();
    let mut bitter_combat = EncounterState::new(&bitter_encounter, cedar_wake::youth_galen());
    bitter_combat.build_turn_queue();

    // Enemies are NOT proper threats — low stats, desperate
    assert!(bitter_combat.enemies.iter().all(|e| e.hp <= 15),
        "workers should be weak — not real fighters");
    assert!(bitter_combat.enemies.iter().all(|e| e.nerve <= 8),
        "workers should have low nerve — already near breaking");
    assert!(bitter_combat.enemies.iter().all(|e| e.accuracy <= 35),
        "workers should have low accuracy — untrained");

    // THE objective exists: "Minimize civilian casualties"
    let civilian_obj = bitter_combat.objectives.iter()
        .find(|o| o.label.contains("civilian casualties"));
    assert!(civilian_obj.is_some(),
        "Bitter Cut MUST have 'Minimize civilian casualties' objective");
    assert_eq!(civilian_obj.unwrap().objective_type,
        saints_mile::combat::types::ObjectiveType::Secondary,
        "civilian objective should be secondary — new and terrible");

    // Same party, same skills, same engine
    // The only difference is who you're pointing them at.

    // Both encounters use the same Galen with the same skills
    let galen_bandit = bandit_combat.party[0].as_ref().unwrap();
    let galen_bitter = bitter_combat.party[0].as_ref().unwrap();
    assert_eq!(galen_bandit.skills, galen_bitter.skills,
        "same Galen, same skills, different meaning");
    assert_eq!(galen_bandit.speed, galen_bitter.speed,
        "same speed — the button hasn't changed");
    assert_eq!(galen_bandit.accuracy, galen_bitter.accuracy,
        "same accuracy — the target has changed");
}

/// Bitter Cut tracks whether the player pulled punches.
#[test]
fn bitter_cut_tracks_participation() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().age_phase = AgePhase::Youth;

    // Run through Cedar Wake to Bitter Cut
    run_cw_scene(&mut store, "cw_arrival", 1); // office
    run_cw_scene(&mut store, "cw_voss_office", 0);
    run_cw_scene(&mut store, "cw_first_courier", 0);
    run_cw_scene(&mut store, "cw_evening", 0);
    run_cw_scene(&mut store, "cw_shooting_post", 0);
    run_cw_scene(&mut store, "cw_horse_thief_briefing", 0);
    run_youth_combat(&mut store, "horse_thief");
    run_cw_scene(&mut store, "cw_horse_thief_return", 0);
    run_cw_scene(&mut store, "cw_bandit_briefing", 0);
    run_youth_combat(&mut store, "bandit_camp");
    run_cw_scene(&mut store, "cw_bandit_camp_return", 0);
    run_cw_scene(&mut store, "cw_bitter_cut_dispatch", 0);
    run_cw_scene(&mut store, "cw_bitter_cut_arrival", 0);
    run_cw_scene(&mut store, "cw_bitter_cut_dispatch_delivery", 0);

    // Fight Bitter Cut
    run_youth_combat(&mut store, "bitter_cut");

    // The encounter resolved and wrote flags
    assert!(store.state().flags.get("bitter_cut_order_maintained") == Some(&FlagValue::Bool(true)));

    // The casualty objective should have resolved too
    // (either high or low depending on combat flow — both are tracked)
    let has_casualty_flag = store.state().flags.contains_key("bitter_cut_casualties_high")
        || store.state().flags.contains_key("bitter_cut_casualties_low");
    assert!(has_casualty_flag, "bitter cut should track casualty outcome");
}

/// Youth Galen feels different from adult Galen.
#[test]
fn youth_galen_identity() {
    let youth_party = cedar_wake::youth_galen();
    let (_, _, hp, nerve, ammo, speed, accuracy, damage, skills, duo_techs, _) =
        &youth_party[0];

    // Youth: faster, less health, less accuracy, less damage
    assert_eq!(*speed, 14, "youth should be fast");
    assert_eq!(*hp, 30, "youth should have lower HP");
    assert_eq!(*accuracy, 65, "youth should have lower accuracy");
    assert_eq!(*damage, 7, "youth should deal less damage");

    // Youth skills: speed-focused, no command skills
    assert!(skills.iter().any(|s| s.0 == "quick_draw"));
    assert!(skills.iter().any(|s| s.0 == "snap_shot"));
    assert!(skills.iter().any(|s| s.0 == "duck"));
    assert!(skills.iter().any(|s| s.0 == "sprint"));

    // No adult skills
    assert!(!skills.iter().any(|s| s.0 == "called_shot"));
    assert!(!skills.iter().any(|s| s.0 == "take_cover"));
    assert!(!skills.iter().any(|s| s.0 == "rally"));
    assert!(!skills.iter().any(|s| s.0 == "overwatch"));
    assert!(!skills.iter().any(|s| s.0 == "setup_shot"));

    // No duo techs — solo
    assert!(duo_techs.is_empty(), "youth Galen has no party to duo tech with");

    // Compare with adult prologue
    let adult_party = saints_mile::content::prologue::prologue_party();
    let (_, _, adult_hp, _, adult_ammo, adult_speed, adult_accuracy, adult_damage, adult_skills, adult_duos, _) =
        &adult_party[0];

    assert!(*adult_speed < *speed, "adult should be slower than youth");
    assert!(*adult_hp > *hp, "adult should have more HP");
    assert!(*adult_accuracy > *accuracy, "adult should be more accurate");
    assert!(*adult_damage > *damage, "adult should deal more damage");
    assert!(adult_skills.len() > skills.len(), "adult should have more skills");
    assert!(!adult_duos.is_empty(), "adult should have duo techs");
}
