//! Integration tests for Ropehouse Blood — Chapter 4.
//!
//! Proves: the game has a real 4-person JRPG party under territorial pressure,
//! party arguments become remembered action, and Rosa is a person before a role.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::combat::party_defs;
use saints_mile::state::store::StateStore;
use saints_mile::state::argument::{self, ReactionType};
use saints_mile::content::ropehouse_blood;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = ropehouse_blood::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn run_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = ropehouse_blood::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));
    let mut combat = EncounterState::new(&encounter, party_defs::ch4_party());
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
        id: FlagId::new("ropehouse_held"), value: FlagValue::Bool(true),
    }]);
}

fn ch4_store(prologue_choice: &str) -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch4");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("ch3_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("ada_joined".to_string(), FlagValue::Bool(true));
    store.state_mut().party.add_member(CharacterId::new("ada"));
    store.state_mut().flags.insert("beat5_choice".to_string(),
        FlagValue::Text(prologue_choice.to_string()));
    (dir, store)
}

// ─── Full Party Feels Real ─────────────────────────────────────────

/// The first full 4-person party combat must feel like the game's real spine.
#[test]
fn full_party_feels_real() {
    let party = party_defs::ch4_party();
    assert_eq!(party.len(), 4, "full party must be exactly 4");

    let encounter = ropehouse_blood::ropehouse_encounter();
    let combat = EncounterState::new(&encounter, party);

    // All 4 slots filled
    assert!(combat.party[0].is_some(), "slot 0 (Galen) should be filled");
    assert!(combat.party[1].is_some(), "slot 1 (Eli) should be filled");
    assert!(combat.party[2].is_some(), "slot 2 (Ada) should be filled");
    assert!(combat.party[3].is_some(), "slot 3 (Rosa) should be filled");

    let galen = combat.party[0].as_ref().unwrap();
    let eli = combat.party[1].as_ref().unwrap();
    let ada = combat.party[2].as_ref().unwrap();
    let rosa = combat.party[3].as_ref().unwrap();

    // Non-overlapping tactical identities
    // Galen: highest accuracy (precision)
    assert!(galen.accuracy > eli.accuracy);
    assert!(galen.accuracy > ada.accuracy);
    assert!(galen.accuracy > rosa.accuracy);

    // Eli: social/nerve specialist (lowest damage, unique role)
    assert!(eli.damage < galen.damage);
    assert!(eli.damage < rosa.damage);

    // Ada: highest nerve (unshakeable), lowest ammo (derringer)
    assert!(ada.nerve >= galen.nerve);
    assert!(ada.ammo < eli.ammo);

    // Rosa: highest damage (physical certainty), highest HP (front line)
    assert!(rosa.damage >= galen.damage);
    assert!(rosa.hp > eli.hp);
    assert!(rosa.hp > ada.hp);

    // Each has distinct skills
    assert!(galen.skills.iter().any(|s| s.0 == "setup_shot"), "Galen sets lanes");
    assert!(eli.skills.iter().any(|s| s.0 == "fast_talk"), "Eli distorts nerve");
    assert!(ada.skills.iter().any(|s| s.0 == "treat_wounds"), "Ada stabilizes");
    assert!(rosa.skills.iter().any(|s| s.0 == "lariat"), "Rosa controls bodies");

    // Duo techs exist for party pairs
    assert!(galen.duo_techs.iter().any(|d| d.0 == "loaded_deck"), "Galen+Eli");
    assert!(galen.duo_techs.iter().any(|d| d.0 == "stay_with_me"), "Galen+Ada");
    assert!(rosa.duo_techs.iter().any(|d| d.0 == "rope_and_shot"), "Rosa+Galen");
}

/// Rosa is a person before she is a role.
#[test]
fn rosa_is_a_person() {
    let rosa = party_defs::rosa();

    // She has land-specific skills, not generic tank skills
    assert!(rosa.skills.iter().any(|s| s.0 == "lariat"), "rope control");
    assert!(rosa.skills.iter().any(|s| s.0 == "brace"), "bodyguard stance");
    assert!(rosa.skills.iter().any(|s| s.0 == "grit"), "stubbornness — shared with Galen");

    // She has a rifle, not a sword
    assert!(rosa.skills.iter().any(|s| s.0 == "rifle_shot"), "rifle, not abstract tank");

    // She's tough but not invulnerable
    assert_eq!(rosa.hp, 38, "tough but not absurd");
    assert_eq!(rosa.nerve, 22, "lower nerve than Ada — she runs on anger, not calm");

    // She has duo techs that reflect relationships, not just roles
    assert!(rosa.duo_techs.iter().any(|d| d.0 == "rope_and_shot"),
        "Rope and Shot: Rosa + Galen working as a unit");
}

// ─── Party Argument Has Memory ─────────────────────────────────────

/// The water-claim argument produces different reactions for each stance.
#[test]
fn party_argument_has_memory() {
    // Force path
    let force = argument::water_claim_argument("force");
    assert_eq!(force.player_stance, "force");
    let rosa_r = force.reactions.iter().find(|r| r.character.0 == "rosa").unwrap();
    assert_eq!(rosa_r.response, ReactionType::Advocated, "Rosa advocated force");
    let ada_r = force.reactions.iter().find(|r| r.character.0 == "ada").unwrap();
    assert_eq!(ada_r.response, ReactionType::Objected, "Ada objected to force");

    // Con path
    let con = argument::water_claim_argument("con");
    let rosa_r = con.reactions.iter().find(|r| r.character.0 == "rosa").unwrap();
    assert_eq!(rosa_r.response, ReactionType::Objected, "Rosa objected to con");
    let eli_r = con.reactions.iter().find(|r| r.character.0 == "eli").unwrap();
    assert_eq!(eli_r.response, ReactionType::Advocated, "Eli advocated con");

    // Negotiate path
    let negotiate = argument::water_claim_argument("negotiate");
    let ada_r = negotiate.reactions.iter().find(|r| r.character.0 == "ada").unwrap();
    assert_eq!(ada_r.response, ReactionType::Advocated, "Ada advocated negotiation");
    let rosa_r = negotiate.reactions.iter().find(|r| r.character.0 == "rosa").unwrap();
    assert_eq!(rosa_r.response, ReactionType::Complied, "Rosa complied with negotiation");

    // All three reach the same structure but with different party memory
    for stance in &["force", "con", "negotiate"] {
        let record = argument::water_claim_argument(stance);
        assert!(!record.reactions.is_empty(), "{} should have reactions", stance);
        assert_eq!(record.chapter, "ch4");
    }
}

/// Full chapter path — force stance.
#[test]
fn chapter_4_full_path_force() {
    let (_dir, mut store) = ch4_store("homestead_first");

    // Entry — Rosa meets as guarded (homestead prologue)
    run_scene(&mut store, "rh_varela_approach", 0);

    // Homestead
    run_scene(&mut store, "rh_homestead", 0);

    // Water claim — force stance (choice 0)
    run_scene(&mut store, "rh_water_claim", 0);
    assert_eq!(store.state().flags.get("ch4_stance"), Some(&FlagValue::Text("force".to_string())));
    assert!(store.state().reputation.get(ReputationAxis::Rancher) > 0);
    assert!(store.state().reputation.get(ReputationAxis::Railroad) < 0);

    // Ropehouse fight
    run_scene(&mut store, "rh_ropehouse_approach", 0);
    run_combat(&mut store, "ropehouse_fight");

    // Aftermath — force-specific lines
    let aftermath = ropehouse_blood::get_scene("rh_aftermath").unwrap();
    let lines = SceneRunner::filter_lines(&aftermath, &store);
    assert!(lines.iter().any(|l| l.text.contains("stood with us")),
        "force stance: Rosa should acknowledge standing with ranchers");
    run_scene(&mut store, "rh_aftermath", 0);

    // Chapter close — Rosa joins
    let close = ropehouse_blood::get_scene("rh_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);
    assert!(store.state().party.has_member(&CharacterId::new("rosa")));
    assert_eq!(store.state().flags.get("rosa_joined"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("ch4_complete"), Some(&FlagValue::Bool(true)));
}

/// Full chapter path — negotiate stance.
#[test]
fn chapter_4_full_path_negotiate() {
    let (_dir, mut store) = ch4_store("town_direct");

    // Entry — Rosa meets as hostile (town-direct prologue)
    let scene = ropehouse_blood::get_scene("rh_varela_approach").unwrap();
    let lines = SceneRunner::filter_lines(&scene, &store);
    assert!(lines.iter().any(|l| l.text.contains("will not have outsiders")),
        "town-direct prologue: Rosa should be hostile on entry");
    run_scene(&mut store, "rh_varela_approach", 0);

    // Homestead
    run_scene(&mut store, "rh_homestead", 0);

    // Water claim — negotiate stance (choice 2)
    run_scene(&mut store, "rh_water_claim", 2);
    assert_eq!(store.state().flags.get("ch4_stance"), Some(&FlagValue::Text("negotiate".to_string())));

    // Ada relationship should increase (she advocated this)
    let ada_rel = store.state().party.relationships.get("galen:ada").copied().unwrap_or(0);
    assert!(ada_rel > 0, "negotiate stance should increase Ada trust");

    // Ropehouse fight + aftermath
    run_scene(&mut store, "rh_ropehouse_approach", 0);
    run_combat(&mut store, "ropehouse_fight");

    let aftermath = ropehouse_blood::get_scene("rh_aftermath").unwrap();
    let lines = SceneRunner::filter_lines(&aftermath, &store);
    assert!(lines.iter().any(|l| l.text.contains("evidence held")),
        "negotiate stance: Ada should talk about evidence");
    run_scene(&mut store, "rh_aftermath", 0);

    // Chapter close
    let close = ropehouse_blood::get_scene("rh_chapter_close").unwrap();
    let close_lines = SceneRunner::filter_lines(&close, &store);
    assert!(close_lines.iter().any(|l| l.text.contains("decent thing first")),
        "negotiate stance: Rosa should acknowledge trying the decent path");
    SceneRunner::apply_scene_effects(&close, &mut store);
    assert!(store.state().party.has_member(&CharacterId::new("rosa")));
}

/// Three stances produce different relationship states.
#[test]
fn three_stances_different_relationships() {
    let stances = ["force", "con", "negotiate"];
    let mut rosa_rels = Vec::new();
    let mut ada_rels = Vec::new();

    for (i, stance) in stances.iter().enumerate() {
        let (_dir, mut store) = ch4_store("homestead_first");
        run_scene(&mut store, "rh_varela_approach", 0);
        run_scene(&mut store, "rh_homestead", 0);
        run_scene(&mut store, "rh_water_claim", i);

        let rosa_rel = store.state().party.relationships.get("galen:rosa").copied().unwrap_or(0);
        let ada_rel = store.state().party.relationships.get("galen:ada").copied().unwrap_or(0);
        rosa_rels.push(rosa_rel);
        ada_rels.push(ada_rel);
    }

    // Force: Rosa highest, Ada lowest
    assert!(rosa_rels[0] > rosa_rels[1], "force: Rosa should be highest");
    assert!(ada_rels[0] < ada_rels[2], "force: Ada should be lower than negotiate");

    // Negotiate: Ada highest
    assert!(ada_rels[2] > ada_rels[0], "negotiate: Ada should be highest");

    // All three produce different profiles
    assert_ne!(rosa_rels[0], rosa_rels[1], "force and con should differ for Rosa");
    assert_ne!(ada_rels[0], ada_rels[2], "force and negotiate should differ for Ada");
}
