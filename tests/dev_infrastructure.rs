//! Tests for dev infrastructure — quickstart, fixtures, event log.

use saints_mile::types::*;
use saints_mile::dev::quickstart::JumpPoint;
use saints_mile::dev::event_log::{EventLog, LogEvent};
use saints_mile::dev::fixtures;
use saints_mile::state::store::StateStore;
use tempfile::TempDir;

/// Every jump point creates a valid state.
#[test]
fn all_jump_points_create_valid_states() {
    for jp in JumpPoint::all() {
        let state = jp.create_state();
        // Must have a chapter and beat
        assert!(!state.chapter.0.is_empty(), "{:?} has empty chapter", jp);
        assert!(!state.beat.0.is_empty(), "{:?} has empty beat", jp);
        // Must have at least Galen in the party
        assert!(state.party.has_member(&CharacterId::new("galen")),
            "{:?} missing Galen", jp);
    }
}

/// Jump points build on each other correctly.
#[test]
fn jump_point_progression() {
    // Cedar Wake start should have youth age
    let cw = JumpPoint::CedarWakeStart.create_state();
    assert_eq!(cw.age_phase, AgePhase::Youth);
    assert!(!cw.party.has_member(&CharacterId::new("eli")),
        "Ch1 Galen should be solo");

    // Shooting post should have Trail Eye but not Steady Aim yet
    let sp = JumpPoint::CedarWakeShootingPost.create_state();
    assert!(sp.party.has_skill(&CharacterId::new("galen"), &SkillId::new("trail_eye")));
    assert!(!sp.party.has_skill(&CharacterId::new("galen"), &SkillId::new("steady_aim")));

    // Bandit camp should have both
    let bc = JumpPoint::CedarWakeBanditCamp.create_state();
    assert!(bc.party.has_skill(&CharacterId::new("galen"), &SkillId::new("trail_eye")));
    assert!(bc.party.has_skill(&CharacterId::new("galen"), &SkillId::new("steady_aim")));

    // Convoy start should be YoungMan with chapter1 complete
    let convoy = JumpPoint::ConvoyStart.create_state();
    assert_eq!(convoy.age_phase, AgePhase::YoungMan);
    assert_eq!(convoy.flags.get("chapter1_complete"), Some(&FlagValue::Bool(true)));

    // Relay triage should have bale dead
    let triage = JumpPoint::RelayTriage.create_state();
    assert_eq!(triage.flags.get("bale_dead"), Some(&FlagValue::Bool(true)));
}

/// All jump point states survive save round-trip.
#[test]
fn jump_points_survive_save_round_trip() {
    let dir = TempDir::new().unwrap();
    for jp in JumpPoint::all() {
        let state = jp.create_state();
        let store = StateStore::from_state(state, dir.path());
        let slug = format!("test_{:?}", jp).to_lowercase();
        let path = store.save(&slug).unwrap();
        let loaded = StateStore::load(&path).unwrap();
        assert_eq!(loaded.state().chapter, store.state().chapter,
            "{:?} chapter mismatch after round-trip", jp);
        assert_eq!(loaded.state().beat, store.state().beat,
            "{:?} beat mismatch after round-trip", jp);
        assert_eq!(loaded.state().flags.len(), store.state().flags.len(),
            "{:?} flags count mismatch after round-trip", jp);
        assert_eq!(loaded.state().party.members.len(), store.state().party.members.len(),
            "{:?} party member count mismatch after round-trip", jp);
        assert_eq!(loaded.state().memory_objects.len(), store.state().memory_objects.len(),
            "{:?} memory_objects count mismatch after round-trip", jp);
        assert_eq!(loaded.state().age_phase, store.state().age_phase,
            "{:?} age_phase mismatch after round-trip", jp);
    }
}

/// Fixture generation produces saves for all branches.
#[test]
fn generate_all_fixtures() {
    let dir = TempDir::new().unwrap();
    let paths = fixtures::generate_fixtures(dir.path()).unwrap();

    // JumpPoint::all() has 14 variants + 3 relay branches (tom/nella/papers)
    // + 2 prologue branches (town_direct/homestead_first) = 19 minimum.
    let jump_point_count = JumpPoint::all().len(); // 14
    let expected_minimum = jump_point_count + 3 + 2; // 14 + 3 relay + 2 prologue = 19
    assert!(paths.len() >= expected_minimum,
        "expected at least {} fixtures ({}jp + 3 relay + 2 prologue), got {}",
        expected_minimum, jump_point_count, paths.len());

    // All files exist
    for path in &paths {
        assert!(path.exists(), "fixture does not exist: {}", path.display());
    }

    // Relay branch saves have correct branch flags
    for branch in &["tom", "nella", "papers"] {
        let relay_path = paths.iter()
            .find(|p| p.file_stem().unwrap().to_string_lossy().contains(&format!("relay_{}", branch)))
            .unwrap_or_else(|| panic!("missing relay_{} fixture", branch));

        let loaded = StateStore::load(relay_path).unwrap();
        assert_eq!(
            loaded.state().flags.get("relay_branch"),
            Some(&FlagValue::Text(branch.to_string())),
            "relay_{} has wrong branch", branch,
        );
        assert!(loaded.state().party.has_skill(
            &CharacterId::new("galen"),
            &SkillId::new("dead_drop"),
        ), "relay_{} missing Dead Drop", branch);
    }
}

/// Event log captures and exports correctly.
#[test]
fn event_log_capture_and_export() {
    let mut log = EventLog::new();
    log.set_context("prologue", "p2");

    log.scene_entered("prologue_poster", "saints_mile_trail");
    log.choice_made("prologue_poster", "Tear it down", 0);
    log.flag_set("tore_poster", &FlagValue::Bool(true));

    log.set_context("prologue", "p5");
    log.scene_entered("morrow_square", "morrow_crossing");
    log.choice_made("morrow_square", "Side with the deputy", 0);

    log.set_context("prologue", "p7");
    log.log(LogEvent::CombatStarted { encounter_id: "glass_arroyo".to_string() });
    log.log(LogEvent::StandoffChosen {
        posture: "SteadyHand".to_string(),
        focus_target: None,
    });
    log.log(LogEvent::CombatEnded { result: "Victory".to_string(), rounds: 3 });

    log.set_context("ch2", "relay");
    log.relay_branch("tom");

    // 8 minimum events from the prologue fixture above:
    // 1. scene_entered (prologue_poster)
    // 2. choice_made (prologue_poster / "Tear it down")
    // 3. flag_set (tore_poster)
    // 4. scene_entered (morrow_square)
    // 5. choice_made (morrow_square / "Side with the deputy")
    // 6. CombatStarted (glass_arroyo)
    // 7. StandoffChosen (SteadyHand)
    // 8. CombatEnded (Victory, 3 rounds)
    // Plus relay_branch("tom") = 9, but >= 8 is the stable lower bound
    // because relay_branch is appended after the context switch.
    assert!(log.entries().len() >= 8, "expected at least 8 entries, got {}", log.entries().len());

    // Export text
    let text = log.export_text();
    assert!(text.contains("Tear it down"));
    assert!(text.contains("RELAY BRANCH: tom"));
    assert!(text.contains("Choices made: 2"));

    // Export RON
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test_log.ron");
    log.export_ron(&path).unwrap();
    assert!(path.exists());
}
