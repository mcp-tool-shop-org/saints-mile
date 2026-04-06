//! Integration tests for Black Willow Fever — Chapter 3.
//!
//! Developer truth harness: compensates for no human testing
//! by being stricter about biography, menu identity, wound persistence,
//! relay-branch texture, and the care-versus-pursuit tension.

use saints_mile::types::*;
use saints_mile::scene::types::SceneTransition;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::combat::types::StandoffPosture;
use saints_mile::combat::engine::{EncounterState, EncounterPhase, CombatSide, CombatAction, TargetSelection};
use saints_mile::combat::party_defs;
use saints_mile::combat::wounds;
use saints_mile::state::store::StateStore;
use saints_mile::content::black_willow;
use tempfile::TempDir;

fn run_scene(store: &mut StateStore, scene_id: &str, choice_index: usize) -> SceneTransition {
    let scene = black_willow::get_scene(scene_id)
        .unwrap_or_else(|| panic!("scene not found: {}", scene_id));
    let prepared = SceneRunner::prepare_scene(&scene, store);
    assert!(prepared.should_play, "scene {} should play", scene_id);
    SceneRunner::apply_scene_effects(&scene, store);
    let chosen = SceneRunner::execute_choice(&scene, choice_index, store)
        .unwrap_or_else(|| panic!("choice {} not available in {}", choice_index, scene_id));
    chosen.transition
}

fn run_combat(store: &mut StateStore, encounter_id: &str) {
    let encounter = black_willow::get_encounter(encounter_id)
        .unwrap_or_else(|| panic!("encounter not found: {}", encounter_id));
    let party = party_defs::ch3_party();
    // Validate party construction: 3 members (Galen, Eli, Ada) with required skills
    assert_eq!(party.len(), 3, "ch3_party must have exactly 3 members (Galen, Eli, Ada)");
    assert_eq!(party[0].0, "galen", "slot 0 must be Galen");
    assert_eq!(party[1].0, "eli", "slot 1 must be Eli");
    assert_eq!(party[2].0, "ada", "slot 2 must be Ada");
    // Verify party members have required combat skills
    assert!(party[0].8.iter().any(|s| s.0 == "quick_draw" || s.0 == "called_shot_precise"),
        "Galen must have a primary attack skill");
    assert!(party[2].8.iter().any(|s| s.0 == "treat_wounds"),
        "Ada must have treat_wounds");
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
                CombatSide::Party | CombatSide::NpcAlly => {
                    combat.enemies.iter()
                        .find(|e| !e.down && !e.panicked)
                        .map(|e| e.id.clone())
                        .unwrap_or_default()
                }
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
    assert!(false, "combat must resolve within 30 rounds");
}

/// Set up a store at Ch3 start with a specific relay branch.
fn ch3_store(relay_branch: &str) -> (TempDir, StateStore) {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());
    store.state_mut().chapter = ChapterId::new("ch3");
    store.state_mut().age_phase = AgePhase::Adult;
    store.state_mut().flags.insert("relay_branch".to_string(), FlagValue::Text(relay_branch.to_string()));
    store.state_mut().flags.insert("poster_born".to_string(), FlagValue::Bool(true));
    store.state_mut().flags.insert("chapter2_complete".to_string(), FlagValue::Bool(true));
    store.state_mut().party.unlock_skill(&CharacterId::new("galen"), &SkillId::new("dead_drop"));
    (dir, store)
}

// ─── Command Menu Diff: Youth vs Adult ─────────────────────────────

/// Adult Galen feels mechanically different from youth Galen.
/// This is the biography-in-the-menu proof.
#[test]
fn adult_galen_feels_different_from_youth() {
    let youth = party_defs::galen(AgePhase::Youth);
    let adult = party_defs::galen(AgePhase::Adult);

    // Speed: youth is faster (reflex), adult is slower (judgment)
    assert!(youth.speed > adult.speed,
        "youth speed {} should exceed adult speed {}", youth.speed, adult.speed);

    // Accuracy: adult is more precise
    assert!(adult.accuracy > youth.accuracy,
        "adult accuracy {} should exceed youth {}", adult.accuracy, youth.accuracy);

    // HP: adult has more (experience = survival)
    assert!(adult.hp > youth.hp);

    // Skill count: adult has more (command, setup, overwatch)
    assert!(adult.skills.len() > youth.skills.len(),
        "adult skills {} should exceed youth {}", adult.skills.len(), youth.skills.len());

    // Youth has speed skills the adult replaced
    assert!(youth.skills.iter().any(|s| s.0 == "snap_shot"), "youth should have snap_shot");
    assert!(youth.skills.iter().any(|s| s.0 == "duck"), "youth should have duck");
    assert!(youth.skills.iter().any(|s| s.0 == "sprint"), "youth should have sprint");

    // Adult has command skills the youth doesn't
    assert!(adult.skills.iter().any(|s| s.0 == "rally"), "adult should have rally");
    assert!(adult.skills.iter().any(|s| s.0 == "setup_shot"), "adult should have setup_shot");
    assert!(adult.skills.iter().any(|s| s.0 == "overwatch"), "adult should have overwatch");
    assert!(adult.skills.iter().any(|s| s.0 == "take_cover"), "adult should have take_cover");

    // Adult has duo techs, youth doesn't
    assert!(!adult.duo_techs.is_empty(), "adult should have duo techs");
    assert!(youth.duo_techs.is_empty(), "youth should have no duo techs");

    // Adult has Loaded Deck AND Stay With Me
    assert!(adult.duo_techs.iter().any(|d| d.0 == "loaded_deck"));
    assert!(adult.duo_techs.iter().any(|d| d.0 == "stay_with_me"));
}

/// Older Galen is not just "adult with different numbers."
/// The hand injury should be felt in the menu.
#[test]
fn older_galen_feels_transformed() {
    let adult = party_defs::galen(AgePhase::Adult);
    let older = party_defs::galen(AgePhase::Older);

    // Older is much slower (damaged hand)
    assert!(older.speed < adult.speed,
        "older speed {} should be less than adult {}", older.speed, adult.speed);

    // But most accurate ever
    assert!(older.accuracy > adult.accuracy,
        "older accuracy {} should exceed adult {}", older.accuracy, adult.accuracy);

    // Hardest-hitting
    assert!(older.damage > adult.damage);

    // Higher nerve (harder to break — he's seen everything)
    assert!(older.nerve > adult.nerve);

    // Has judgment skills adult doesn't
    assert!(older.skills.iter().any(|s| s.0 == "initiative_read"));
    assert!(older.skills.iter().any(|s| s.0 == "party_command"));
    assert!(older.skills.iter().any(|s| s.0 == "judgment_shot"));

    // Lost Quick Draw (speed-first identity gone with the hand)
    assert!(!older.skills.iter().any(|s| s.0 == "quick_draw"),
        "older Galen should NOT have quick_draw — the hand changed him");

    // But still has Steady Aim (the gift outlives the giver)
    assert!(older.skills.iter().any(|s| s.0 == "steady_aim"),
        "older Galen MUST still have steady_aim");
}

// ─── Ada Tests ─────────────────────────────────────────────────────

/// Ada is not just "a healer." She has a distinct identity.
#[test]
fn ada_is_not_just_a_healer() {
    let ada = party_defs::ada();

    // She has triage skills
    assert!(ada.skills.iter().any(|s| s.0 == "treat_wounds"));
    assert!(ada.skills.iter().any(|s| s.0 == "tourniquet"));
    assert!(ada.skills.iter().any(|s| s.0 == "steady_nerves"));
    assert!(ada.skills.iter().any(|s| s.0 == "smelling_salts"));

    // She has a weapon (derringer — emergency only)
    assert!(ada.skills.iter().any(|s| s.0 == "derringer"));

    // She has duo techs that are NOT just healing
    assert!(ada.duo_techs.iter().any(|d| d.0 == "stay_with_me"),
        "Ada should have Stay With Me (Galen covers, Ada stabilizes)");
    assert!(ada.duo_techs.iter().any(|d| d.0 == "second_opinion"),
        "Ada should have Second Opinion (Eli + Ada intelligence)");

    // Her stats are different from a generic healer:
    // Low ammo (4 — derringer only), high nerve (30 — unshakeable)
    assert_eq!(ada.ammo, 4, "Ada has a derringer, not a rifle");
    assert_eq!(ada.nerve, 30, "Ada has high nerve — impossible to impress");
}

// ─── Wound Persistence ─────────────────────────────────────────────

/// Wounds persist between encounters and are treatable by Ada.
#[test]
fn wound_persistence_and_triage() {
    let wound = wounds::gunshot_wound();
    assert!(wound.treatable);
    assert!(wound.penalties.iter().any(|p| p.stat == "accuracy" && p.amount < 0));

    // Triage heals treatable wounds
    let result = wounds::triage(&[wound.clone()], false);
    assert_eq!(result.healed.len(), 1);
    assert!(result.hp_restored > 0 || result.nerve_restored > 0);
    assert!(result.time_cost > 0, "triage should cost time");

    // Thorough triage costs more time but heals more
    let quick = wounds::triage(&[wound.clone()], false);
    let thorough = wounds::triage(&[wound], true);
    assert!(thorough.time_cost > quick.time_cost, "thorough triage costs more time");
}

// ─── Relay Branch Entry Variants ───────────────────────────────────

/// Chapter 3 opens differently based on relay branch.
/// Same structure, different texture of truth.
#[test]
fn relay_branch_entry_variants() {
    for branch in &["tom", "nella", "papers"] {
        let (_dir, store) = ch3_store(branch);

        let scene = black_willow::get_scene("bw_morrow_aftermath").unwrap();
        let prepared = SceneRunner::prepare_scene(&scene, &store);

        // The scene should play for all branches
        assert!(prepared.should_play, "aftermath should play for {} branch", branch);

        // Each branch should have its own texture line
        let branch_line = prepared.lines.iter()
            .find(|l| match *branch {
                "tom" => l.text.contains("freight") || l.text.contains("wagon"),
                "nella" => l.text.contains("names") || l.text.contains("kitchens"),
                "papers" => l.text.contains("fragment") || l.text.contains("paper"),
                _ => false,
            });
        assert!(branch_line.is_some(),
            "{} branch should have its own texture line in the aftermath", branch);
    }
}

/// Sheriff trail scene integrates relay branch evidence differently.
#[test]
fn sheriff_trail_integrates_relay_branch() {
    for branch in &["tom", "nella", "papers"] {
        let (_dir, mut store) = ch3_store(branch);

        // Run to the sheriff trail
        run_scene(&mut store, "bw_morrow_aftermath", 0);
        run_scene(&mut store, "bw_road", 0);
        run_scene(&mut store, "bw_district", 0); // treat first
        run_scene(&mut store, "bw_triage", 0);

        let scene = black_willow::get_scene("bw_sheriff_trail").unwrap();
        let lines = SceneRunner::filter_lines(&scene, &store);

        // Each branch should produce a different investigatory texture
        let has_branch_line = lines.iter().any(|l| match *branch {
            "tom" => l.text.contains("freight") || l.text.contains("Wagon"),
            "nella" => l.text.contains("names") || l.text.contains("roster"),
            "papers" => l.text.contains("fragment") || l.text.contains("consignment"),
            _ => false,
        });
        assert!(has_branch_line,
            "{} branch should have investigatory texture in sheriff trail", branch);
    }
}

// ─── Care vs Pursuit — The Chapter Soul Test ───────────────────────

/// The game's care-versus-pursuit tension:
/// treating patients delays clue progress, pursuing clues first
/// changes Ada's tone. Neither is wrong. Both cost something.
#[test]
fn care_versus_pursuit_tension() {
    // Path A: care first
    let (_dir_a, mut store_a) = ch3_store("tom");
    run_scene(&mut store_a, "bw_morrow_aftermath", 0);
    run_scene(&mut store_a, "bw_road", 0);
    run_scene(&mut store_a, "bw_district", 0); // choice 0 = treat first
    assert_eq!(store_a.state().flags.get("treated_first"), Some(&FlagValue::Bool(true)));
    assert_eq!(store_a.state().flags.get("care_before_pursuit"), Some(&FlagValue::Bool(true)));

    // Path B: pursuit first
    let (_dir_b, mut store_b) = ch3_store("tom");
    run_scene(&mut store_b, "bw_morrow_aftermath", 0);
    run_scene(&mut store_b, "bw_road", 0);
    run_scene(&mut store_b, "bw_district", 1); // choice 1 = pursue first
    assert_eq!(store_b.state().flags.get("pursued_first"), Some(&FlagValue::Bool(true)));
    assert!(store_b.state().flags.get("care_before_pursuit").is_none());

    // Care-first path: Ada's chapter-close line acknowledges it
    run_scene(&mut store_a, "bw_triage", 0);
    run_scene(&mut store_a, "bw_sheriff_trail", 0);
    run_combat(&mut store_a, "pump_house_hold");
    let close_a = black_willow::get_scene("bw_chapter_close").unwrap();
    let lines_a = SceneRunner::filter_lines(&close_a, &store_a);
    assert!(lines_a.iter().any(|l| l.text.contains("stopped for the patients")),
        "care-first: Ada should acknowledge treating patients first");

    // Pursuit-first path: Ada's tone is different
    run_scene(&mut store_b, "bw_sheriff_trail", 0);
    run_combat(&mut store_b, "pump_house_hold");
    let close_b = black_willow::get_scene("bw_chapter_close").unwrap();
    let lines_b = SceneRunner::filter_lines(&close_b, &store_b);
    assert!(lines_b.iter().any(|l| l.text.contains("records first")),
        "pursuit-first: Ada should acknowledge records-first approach");

    // Both paths result in Ada joining
    SceneRunner::apply_scene_effects(&close_a, &mut store_a);
    SceneRunner::apply_scene_effects(&close_b, &mut store_b);
    assert!(store_a.state().party.has_member(&CharacterId::new("ada")));
    assert!(store_b.state().party.has_member(&CharacterId::new("ada")));
    assert_eq!(store_a.state().flags.get("ada_joined"), Some(&FlagValue::Bool(true)));
    assert_eq!(store_b.state().flags.get("ada_joined"), Some(&FlagValue::Bool(true)));

    // But the Galen-Ada relationship differs
    // (care-first built trust in the triage scene)
    let galen_ada_a = store_a.state().party.relationships.get("galen:ada");
    let galen_ada_b = store_b.state().party.relationships.get("galen:ada");
    // Care path should have higher relationship
    let rel_a = galen_ada_a.copied().unwrap_or(0);
    let rel_b = galen_ada_b.copied().unwrap_or(0);
    assert!(rel_a > rel_b,
        "care-first relationship ({}) should exceed pursuit-first ({})", rel_a, rel_b);
}

// ─── Full Chapter Path ─────────────────────────────────────────────

/// Complete Chapter 3 golden path: aftermath → road → district →
/// triage → sheriff trail → pump house fight → Ada joins.
#[test]
fn chapter_3_full_path() {
    let (_dir, mut store) = ch3_store("tom");

    // Entry
    run_scene(&mut store, "bw_morrow_aftermath", 0);
    assert_eq!(store.state().flags.get("ch3_started"), Some(&FlagValue::Bool(true)));

    // Road
    run_scene(&mut store, "bw_road", 0);

    // District — treat first
    run_scene(&mut store, "bw_district", 0);

    // Triage — help patients
    run_scene(&mut store, "bw_triage", 0);
    assert!(store.state().party.relationships.get("galen:ada")
        .map_or(false, |v| *v > 0),
        "helping patients should build Ada trust");

    // Sheriff trail
    run_scene(&mut store, "bw_sheriff_trail", 0);

    // Pump house fight
    run_combat(&mut store, "pump_house_hold");

    // Chapter close
    let close = black_willow::get_scene("bw_chapter_close").unwrap();
    SceneRunner::apply_scene_effects(&close, &mut store);

    // Ada joined
    assert!(store.state().party.has_member(&CharacterId::new("ada")));
    assert_eq!(store.state().flags.get("ada_joined"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("ch3_complete"), Some(&FlagValue::Bool(true)));
    assert_eq!(store.state().flags.get("sheriff_trail_found"), Some(&FlagValue::Bool(true)));

    // Memory: sheriff security file echoes to Ch7
    assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "sheriff_security_file"));

    // Save round-trip
    let path = store.save("ch3_golden").unwrap();
    let loaded = StateStore::load(&path).unwrap();
    assert_eq!(loaded.state().flags.get("ch3_complete"), Some(&FlagValue::Bool(true)));
    assert!(loaded.state().party.has_member(&CharacterId::new("ada")));
}

// ─── 3-Character Party Proof ───────────────────────────────────────

/// Chapter 3's pump house fight uses a 3-character party.
/// The combat engine must handle Galen + Eli + Ada in 4 slots.
#[test]
fn three_character_party_combat() {
    let encounter = black_willow::pump_house_encounter();
    let combat = EncounterState::new(&encounter, party_defs::ch3_party());

    // 3 of 4 slots filled
    assert!(combat.party[0].is_some()); // Galen
    assert!(combat.party[1].is_some()); // Eli
    assert!(combat.party[2].is_some()); // Ada
    assert!(combat.party[3].is_none()); // Empty — slot 4 for Rosa in Ch4

    // Galen is the fastest party member
    let galen = combat.party[0].as_ref().unwrap();
    let eli = combat.party[1].as_ref().unwrap();
    let ada = combat.party[2].as_ref().unwrap();
    assert!(galen.speed > eli.speed);
    assert!(galen.speed > ada.speed);

    // Ada has the highest nerve (hardest to break)
    assert!(ada.nerve >= galen.nerve);

    // Ada has the least ammo (derringer)
    assert!(ada.ammo < eli.ammo);
    assert!(ada.ammo < galen.ammo);
}
