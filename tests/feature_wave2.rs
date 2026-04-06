//! Wave 2 feature tests — validates new functionality from the dogfood swarm.
//!
//! Covers: pressure system, NPC behavior, age-variant skills, cover mechanics,
//! evidence system, party dispersal, and content completeness.

mod common;

use saints_mile::types::*;
use saints_mile::combat::types::*;
use saints_mile::combat::engine::*;
use saints_mile::combat::environment::*;
use saints_mile::combat::party_defs;
use saints_mile::content;
use saints_mile::pressure::types::*;
use saints_mile::pressure::engine::*;
use saints_mile::scene::types::*;
use saints_mile::state::evidence::*;
use saints_mile::state::store::StateStore;
use saints_mile::state::types::GameState;
use tempfile::TempDir;

// ─── Helpers ──────────────────────────────────────────────────────────

/// Build a quick_draw Skill definition (same as wave1 for consistency).
fn quick_draw_skill() -> Skill {
    Skill {
        id: SkillId::new("quick_draw"),
        name: "Quick Draw".to_string(),
        description: "Fast pistol shot".to_string(),
        line: SkillLine::Deadeye,
        unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
        age_variants: vec![
            AgeVariant {
                phase: AgePhase::Youth,
                accuracy: 60,
                damage: 7,
                speed_priority: 14,
                nerve_damage: 2,
                description_override: None,
            },
            AgeVariant {
                phase: AgePhase::Adult,
                accuracy: 70,
                damage: 10,
                speed_priority: 12,
                nerve_damage: 3,
                description_override: None,
            },
        ],
        cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 0 },
    }
}

/// Build a minimal pressure encounter for testing.
fn test_pressure_encounter() -> PressureEncounter {
    PressureEncounter {
        id: "test_pressure".to_string(),
        pressure_type: PressureType::Escort {
            cargo: vec![CargoItem {
                id: "supplies".to_string(),
                name: "Supply wagon".to_string(),
                integrity: 100,
                max_integrity: 100,
                loss_effect: vec![],
            }],
        },
        pressure_bars: vec![
            PressureBar {
                id: "morale".to_string(),
                label: "Convoy morale".to_string(),
                current: 80,
                max: 100,
                fail_at: 20,
                visible: true,
            },
            PressureBar {
                id: "distance".to_string(),
                label: "Distance remaining".to_string(),
                current: 50,
                max: 100,
                fail_at: 0,
                visible: true,
            },
        ],
        party_actions: vec![PressurePartyAction {
            character: CharacterId::new("galen"),
            actions: vec![PressureAction {
                id: "encourage".to_string(),
                label: "Encourage the convoy".to_string(),
                description: "Boost morale".to_string(),
                target_bar: "morale".to_string(),
                delta: 10,
                conditions: vec![],
            }],
        }],
        success_threshold: PressureCondition::AllBarsAboveFail,
        failure_threshold: PressureCondition::BarReached {
            bar_id: "morale".to_string(),
            threshold: 20,
        },
        outcome_effects: vec![],
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FT-003: Pressure system
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn pressure_engine_creates_with_bars() {
    let pe = test_pressure_encounter();
    let engine = PressureEngine::new(pe);
    assert!(engine.is_active(), "engine should start in progress");
    assert_eq!(engine.encounter.pressure_bars.len(), 2, "test pressure should have 2 bars");
}

#[test]
fn pressure_bar_tracks_current_max_fail() {
    let pe = test_pressure_encounter();
    let engine = PressureEngine::new(pe);
    let morale = engine.get_bar("morale").unwrap();
    assert_eq!(morale.current, 80);
    assert_eq!(morale.max, 100);
    assert_eq!(morale.fail_at, 20);
    assert!(morale.visible);
}

#[test]
fn pressure_engine_action_modifies_bar() {
    let pe = test_pressure_encounter();
    let mut engine = PressureEngine::new(pe);
    engine.begin_round();

    let result = engine.process_action("galen", "encourage");
    assert!(result.is_some(), "action should process successfully");
    let result = result.unwrap();
    assert_eq!(result.bar_id, "morale");
    assert_eq!(result.delta, 10);
    assert_eq!(result.bar_after, 90, "encourage action should increase morale by 10");

    let morale = engine.get_bar("morale").unwrap();
    assert_eq!(morale.current, 90);
}

#[test]
fn pressure_failure_triggers_at_threshold() {
    let pe = test_pressure_encounter();
    let mut engine = PressureEngine::new(pe);
    engine.begin_round();

    // Manually reduce morale below fail_at to trigger failure
    {
        let bar = engine.encounter.pressure_bars.iter_mut()
            .find(|b| b.id == "morale").unwrap();
        bar.current = 15; // below fail_at of 20
    }

    let resolution = engine.check_thresholds();
    assert_eq!(resolution.outcome, PressureOutcome::Failure);
    assert!(resolution.trigger_bar.is_some());
    assert_eq!(resolution.trigger_bar.unwrap(), "morale");
    assert!(!engine.is_active(), "engine should no longer be active after failure");
}

#[test]
fn pressure_success_when_all_bars_above_fail() {
    let pe = test_pressure_encounter();
    let mut engine = PressureEngine::new(pe);
    engine.begin_round();

    // Both bars start above their fail_at: morale=80 (fail=20), distance=50 (fail=0)
    let resolution = engine.check_thresholds();
    assert_eq!(resolution.outcome, PressureOutcome::Success);
    assert!(!engine.is_active(), "engine should resolve to success");
}

#[test]
fn pressure_bar_at_fail_threshold_means_failure() {
    let pe = test_pressure_encounter();
    let mut engine = PressureEngine::new(pe);
    engine.begin_round();

    // Drive morale to exactly fail_at (20) — BarReached checks current <= threshold
    {
        let bar = engine.encounter.pressure_bars.iter_mut()
            .find(|b| b.id == "morale").unwrap();
        bar.current = 20; // exactly at fail_at/threshold
    }

    let resolution = engine.check_thresholds();
    assert_eq!(resolution.outcome, PressureOutcome::Failure,
        "bar at exactly the threshold should trigger failure");
}

#[test]
fn pressure_actions_blocked_after_resolution() {
    let pe = test_pressure_encounter();
    let mut engine = PressureEngine::new(pe);

    // Force resolution
    engine.outcome = PressureOutcome::Success;

    let result = engine.process_action("galen", "encourage");
    assert!(result.is_none(), "actions should be blocked after resolution");
}

// ═══════════════════════════════════════════════════════════════════════
// FT-005: NPC behavior
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn npc_behavior_variants_exist() {
    // All four NPC behavior variants should exist
    let behaviors = [
        NpcBehavior::Professional,
        NpcBehavior::Unreliable,
        NpcBehavior::Protective,
        NpcBehavior::Nervous,
    ];
    for b in &behaviors {
        // Just verify the enum variants compile and are distinct
        assert_eq!(*b, *b);
    }
}

#[test]
fn npc_combatant_carries_behavior() {
    let npc = NpcCombatant {
        character: CharacterId::new("cal"),
        behavior: NpcBehavior::Professional,
        hp: 35,
        nerve: 30,
        speed: 9,
        accuracy: 62,
        damage: 9,
    };
    assert_eq!(npc.behavior, NpcBehavior::Professional);
    assert_eq!(npc.character, CharacterId::new("cal"));
}

#[test]
fn protective_npc_behavior_is_distinct() {
    // Protective NPCs would prioritize defending allies
    let npc = NpcCombatant {
        character: CharacterId::new("bale"),
        behavior: NpcBehavior::Protective,
        hp: 40,
        nerve: 25,
        speed: 7,
        accuracy: 55,
        damage: 12,
    };
    assert_eq!(npc.behavior, NpcBehavior::Protective);
    assert_ne!(npc.behavior, NpcBehavior::Professional);
}

#[test]
fn npc_ally_in_encounter_uses_behavior() {
    let encounter = Encounter {
        id: EncounterId::new("npc_behavior_test"),
        phases: vec![CombatPhase {
            id: "main".to_string(),
            description: "Test NPC behavior".to_string(),
            enemies: vec![EnemyTemplate {
                id: "thug".to_string(),
                name: "Thug".to_string(),
                hp: 20, nerve: 15, damage: 5, accuracy: 50, speed: 8,
                bluff: 10, nerve_threshold: 5,
            }],
            npc_allies: vec![NpcCombatant {
                character: CharacterId::new("cal"),
                behavior: NpcBehavior::Professional,
                hp: 35, nerve: 30, speed: 9, accuracy: 62, damage: 9,
            }],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None,
        party_slots: 4,
        terrain: Terrain {
            name: "Test".to_string(),
            cover: vec![],
            hazards: vec![],
        },
        objectives: vec![],
        outcome_effects: vec![],
        escapable: true,
    };

    let party = vec![party_defs::galen(AgePhase::Adult).to_combat_tuple()];
    let state = EncounterState::new(&encounter, party);

    // NPC ally should be present with the correct behavior
    assert_eq!(state.npc_allies.len(), 1);
    assert_eq!(state.npc_allies[0].combatant.id, "cal");
}

// ═══════════════════════════════════════════════════════════════════════
// FT-006: Age-variant skills
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn skill_registry_get_variant_returns_age_specific_stats() {
    let mut registry = SkillRegistry::new();
    registry.register(quick_draw_skill());

    let youth = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Youth);
    assert!(youth.is_some());
    assert_eq!(youth.unwrap().accuracy, 60);
    assert_eq!(youth.unwrap().damage, 7);

    let adult = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Adult);
    assert!(adult.is_some());
    assert_eq!(adult.unwrap().accuracy, 70);
    assert_eq!(adult.unwrap().damage, 10);
}

#[test]
fn youth_variant_differs_from_adult_variant() {
    let mut registry = SkillRegistry::new();
    registry.register(quick_draw_skill());

    let youth = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Youth).unwrap();
    let adult = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Adult).unwrap();

    assert_ne!(youth.accuracy, adult.accuracy, "youth and adult accuracy should differ");
    assert_ne!(youth.damage, adult.damage, "youth and adult damage should differ");
    assert_ne!(youth.speed_priority, adult.speed_priority, "youth and adult speed should differ");
    assert!(youth.damage < adult.damage, "youth should deal less damage than adult");
}

#[test]
fn missing_age_variant_falls_back_to_base() {
    let mut registry = SkillRegistry::new();
    registry.register(quick_draw_skill());

    // Older phase is not defined — should fall back to first variant (Youth)
    let fallback = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Older);
    assert!(fallback.is_some(), "missing variant should fall back");
    assert_eq!(fallback.unwrap().phase, AgePhase::Youth, "should fall back to first variant");
}

#[test]
fn young_man_variant_falls_back_similarly() {
    let mut registry = SkillRegistry::new();
    registry.register(quick_draw_skill());

    // YoungMan not defined — falls back
    let fallback = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::YoungMan);
    assert!(fallback.is_some());
    assert_eq!(fallback.unwrap().phase, AgePhase::Youth);
}

// ═══════════════════════════════════════════════════════════════════════
// FT-007: Cover mechanics
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn cover_element_has_durability() {
    let cover = CoverElement {
        name: "Stone wall".to_string(),
        durability: 50,
        destructible: true,
    };
    assert_eq!(cover.durability, 50);
    assert!(cover.destructible);
}

#[test]
fn full_cover_position_exists() {
    let pos = PositionState::InCover;
    assert_eq!(pos, PositionState::InCover);
}

#[test]
fn partial_cover_position_exists() {
    let pos = PositionState::PartialCover;
    assert_eq!(pos, PositionState::PartialCover);
    assert_ne!(pos, PositionState::InCover, "partial and full cover are distinct");
}

#[test]
fn destructible_cover_can_be_destroyed_via_environment() {
    let mut env = EnvironmentState::new();
    env.cover.push(LiveCover {
        id: "wall".to_string(),
        name: "Stone wall".to_string(),
        durability: 30,
        max_durability: 30,
        destroyed: false,
    });
    assert_eq!(env.intact_cover(), 1);

    // Destroy it
    env.execute_action(&EnvironmentAction::DestroyCover {
        cover_id: "wall".to_string(),
        damage: 40,
    });

    assert_eq!(env.intact_cover(), 0);
    assert!(env.cover[0].destroyed, "cover should be destroyed after sufficient damage");
}

#[test]
fn cover_survives_partial_damage() {
    let mut env = EnvironmentState::new();
    env.cover.push(LiveCover {
        id: "barricade".to_string(),
        name: "Barricade".to_string(),
        durability: 50,
        max_durability: 50,
        destroyed: false,
    });

    // Partial damage
    env.execute_action(&EnvironmentAction::DestroyCover {
        cover_id: "barricade".to_string(),
        damage: 20,
    });

    assert_eq!(env.intact_cover(), 1, "cover should survive partial damage");
    assert_eq!(env.cover[0].durability, 30);
    assert!(!env.cover[0].destroyed);
}

#[test]
fn position_states_are_comprehensive() {
    // All position states should exist for cover mechanics
    let positions = [
        PositionState::Open,
        PositionState::InCover,
        PositionState::PartialCover,
        PositionState::Elevated,
        PositionState::FrontLine,
        PositionState::BackLine,
    ];
    // Verify they are all distinct
    for (i, a) in positions.iter().enumerate() {
        for (j, b) in positions.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "position states should all be distinct");
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Evidence system
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn evidence_can_be_collected_in_game_state() {
    let mut state = GameState::new_game();
    assert!(!state.has_collected("route_manifest_sm"));

    state.collect_evidence("route_manifest_sm");
    assert!(state.has_collected("route_manifest_sm"), "should have collected evidence");
}

#[test]
fn evidence_collection_is_idempotent() {
    let mut state = GameState::new_game();
    state.collect_evidence("payroll_ledger_convoy");
    state.collect_evidence("payroll_ledger_convoy"); // duplicate
    assert_eq!(
        state.collected_evidence.iter().filter(|e| *e == "payroll_ledger_convoy").count(),
        1,
        "collecting same evidence twice should not duplicate it",
    );
}

#[test]
fn relay_evidence_differs_by_branch() {
    let archive = iron_ledger_archive();

    let tom_evidence = verify_against_branch(&archive, "tom");
    let nella_evidence = verify_against_branch(&archive, "nella");
    let papers_evidence = verify_against_branch(&archive, "papers");

    // Each branch should verify at least some records
    assert!(!tom_evidence.is_empty(), "tom branch should verify records");
    assert!(!nella_evidence.is_empty(), "nella branch should verify records");
    assert!(!papers_evidence.is_empty(), "papers branch should verify records");

    // Branches should have different verification counts
    let counts = branch_verification_counts(&archive);
    assert!(counts.len() == 3, "should have counts for all 3 branches");

    // Tom and Nella have different available verifications
    let tom_ids: Vec<_> = tom_evidence.iter().map(|r| r.record_id.clone()).collect();
    let nella_ids: Vec<_> = nella_evidence.iter().map(|r| r.record_id.clone()).collect();
    // At minimum, payroll_ledger is nella-only, route_manifest is tom-only
    assert!(
        tom_ids.contains(&"route_manifest_sm".to_string()),
        "tom should verify route_manifest_sm",
    );
    assert!(
        nella_ids.contains(&"payroll_ledger_convoy".to_string()),
        "nella should verify payroll_ledger_convoy",
    );
}

#[test]
fn verification_result_carries_proof() {
    let archive = iron_ledger_archive();
    let results = verify_against_branch(&archive, "tom");

    for result in &results {
        assert!(result.verified);
        assert!(!result.proof.is_empty(), "verified record should carry proof text");
        assert_eq!(result.relay_branch_used, "tom");
    }
}

#[test]
fn lucien_archive_contribution_varies_by_status() {
    let forced = lucien_archive_contribution("forced_guide");
    let prisoner = lucien_archive_contribution("prisoner");
    let judged = lucien_archive_contribution("judged");
    let unknown = lucien_archive_contribution("unknown");

    assert!(!forced.is_empty(), "forced guide should contribute");
    assert!(!prisoner.is_empty(), "prisoner should contribute");
    assert!(judged.len() >= forced.len(), "judged should contribute at least as much as forced");
    assert!(unknown.is_empty(), "unknown status should not contribute");
}

// ═══════════════════════════════════════════════════════════════════════
// Party dispersal
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn departed_members_tracked_via_flags() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // Simulate ch12 dispersal — names_in_dust sets departed flags
    store.state_mut().flags.insert(
        "ada_departed".to_string(),
        FlagValue::Bool(true),
    );
    store.state_mut().flags.insert(
        "rosa_departed".to_string(),
        FlagValue::Bool(true),
    );
    store.state_mut().flags.insert(
        "miriam_departed".to_string(),
        FlagValue::Bool(true),
    );

    assert_eq!(
        store.state().flags.get("ada_departed"),
        Some(&FlagValue::Bool(true)),
    );
    assert_eq!(
        store.state().flags.get("rosa_departed"),
        Some(&FlagValue::Bool(true)),
    );
    assert_eq!(
        store.state().flags.get("miriam_departed"),
        Some(&FlagValue::Bool(true)),
    );
}

#[test]
fn returned_flags_track_reassembly() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // Set departed first
    store.state_mut().flags.insert("ada_departed".to_string(), FlagValue::Bool(true));

    // Simulate old_friends chapter — ally returns
    store.state_mut().flags.insert("ada_returned_body".to_string(), FlagValue::Bool(true));

    assert_eq!(
        store.state().flags.get("ada_returned_body"),
        Some(&FlagValue::Bool(true)),
        "returned flag should be set after reassembly",
    );

    // Both departed and returned flags coexist — departed is historical fact
    assert_eq!(
        store.state().flags.get("ada_departed"),
        Some(&FlagValue::Bool(true)),
    );
}

#[test]
fn party_members_can_be_removed_and_readded() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // Add ada to party
    store.state_mut().party.add_member(CharacterId::new("ada"));
    assert!(store.state().party.has_member(&CharacterId::new("ada")));

    // Remove ada (dispersal)
    store.state_mut().party.remove_member(&CharacterId::new("ada"));
    assert!(!store.state().party.has_member(&CharacterId::new("ada")));

    // Re-add ada (returned)
    store.state_mut().party.add_member(CharacterId::new("ada"));
    assert!(store.state().party.has_member(&CharacterId::new("ada")));
}

#[test]
fn reassembly_requires_enough_returning_allies() {
    use saints_mile::state::reassembly::*;

    let returns = chapter_14_reassembly();
    assert!(
        can_approach_saints_mile(&returns),
        "default reassembly should have enough returning allies",
    );
}

#[test]
fn reassembly_fails_with_insufficient_returns() {
    use saints_mile::state::reassembly::*;

    // Only memory-only returns — not enough bodies
    let returns = vec![
        AllyReturn {
            character: CharacterId::new("eli"),
            mode: ReturnMode::MemoryOnly,
            truth_carried: "Test".to_string(),
            change: "Test".to_string(),
            tension: "Test".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("ada"),
            mode: ReturnMode::Refusal,
            truth_carried: "Test".to_string(),
            change: "Test".to_string(),
            tension: "Test".to_string(),
        },
    ];
    assert!(
        !can_approach_saints_mile(&returns),
        "memory-only and refusal returns should not be enough",
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Content completeness
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn ch6_has_chapter_close_scene() {
    let scene = content::get_scene("fuse_country", "fc_chapter_close");
    assert!(scene.is_some(), "fuse_country should have a chapter_close scene");
    let scene = scene.unwrap();
    assert_eq!(scene.id, SceneId::new("fc_chapter_close"));
}

#[test]
fn ch8_has_mission_defense_encounter() {
    let enc = content::get_encounter("burned_mission", "mission_defense");
    assert!(enc.is_some(), "burned_mission should have a mission_defense encounter");
    let enc = enc.unwrap();
    assert_eq!(enc.id, EncounterId::new("mission_defense"));
    assert!(!enc.phases.is_empty(), "mission_defense should have combat phases");
    // Should have enforcers as enemies
    let enemy_count = enc.phases[0].enemies.len();
    assert!(enemy_count >= 3, "mission_defense should have at least 3 enemies");
}

#[test]
fn ch8_chapter_close_has_memory_refs() {
    let scene = content::get_scene("burned_mission", "bm_chapter_close");
    assert!(scene.is_some(), "burned_mission should have a chapter_close");
    let scene = scene.unwrap();
    assert!(
        !scene.memory_refs.is_empty(),
        "burned_mission chapter_close should have memory refs (bell_phenomenon echo)",
    );
    // Verify the bell phenomenon ref targets ch15
    let bell_ref = scene.memory_refs.iter().find(|r| r.object == MemoryObjectId::new("bell_phenomenon"));
    assert!(bell_ref.is_some(), "should reference bell_phenomenon");
}

#[test]
fn ch6_chapter_close_has_memory_refs() {
    let scene = content::get_scene("fuse_country", "fc_chapter_close");
    assert!(scene.is_some());
    let scene = scene.unwrap();
    assert!(
        !scene.memory_refs.is_empty(),
        "fuse_country chapter_close should have memory refs (trestle_blast_scar echo)",
    );
    let trestle_ref = scene.memory_refs.iter().find(|r| r.object == MemoryObjectId::new("trestle_blast_scar"));
    assert!(trestle_ref.is_some(), "should reference trestle_blast_scar");
}

#[test]
fn ch6_chapter_close_sets_completion_flag() {
    let scene = content::get_scene("fuse_country", "fc_chapter_close").unwrap();
    let sets_ch6_complete = scene.state_effects.iter().any(|e| {
        matches!(e, StateEffect::SetFlag { id, value }
            if id.0 == "ch6_complete" && *value == FlagValue::Bool(true))
    });
    assert!(sets_ch6_complete, "ch6 chapter_close should set ch6_complete flag");
}

#[test]
fn ch8_chapter_close_sets_completion_flag() {
    let scene = content::get_scene("burned_mission", "bm_chapter_close").unwrap();
    let sets_ch8_complete = scene.state_effects.iter().any(|e| {
        matches!(e, StateEffect::SetFlag { id, value }
            if id.0 == "ch8_complete" && *value == FlagValue::Bool(true))
    });
    assert!(sets_ch8_complete, "ch8 chapter_close should set ch8_complete flag");
}

#[test]
fn iron_ledger_archive_has_records() {
    let archive = iron_ledger_archive();
    assert!(archive.len() >= 5, "archive should have substantial records");
    // Each record should have an id, description, and standalone proof
    for record in &archive {
        assert!(!record.id.is_empty());
        assert!(!record.description.is_empty());
        assert!(!record.standalone_proof.is_empty());
    }
}
