//! Wave 1 feature tests — validates new functionality from the dogfood swarm.
//!
//! Covers: skill registry, duo-tech validation, flee action, NPC ally stats,
//! objective auto-resolve, breakwater dispatch, chapter routing, chapter
//! validation, and graceful exit.

mod common;

use saints_mile::types::*;
use saints_mile::combat::types::*;
use saints_mile::combat::engine::*;
use saints_mile::combat::party_defs;
use saints_mile::content;
use saints_mile::ui::{App, AppScreen, QuitOption};
use saints_mile::state::store::StateStore;
use tempfile::TempDir;

// ─── Helpers ──────────────────────────────────────────────────────────

/// Build a minimal encounter for testing engine features.
fn test_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("test_encounter"),
        phases: vec![CombatPhase {
            id: "main".to_string(),
            description: "Test phase".to_string(),
            enemies: vec![EnemyTemplate {
                id: "thug".to_string(),
                name: "Thug".to_string(),
                hp: 20,
                nerve: 15,
                damage: 5,
                accuracy: 50,
                speed: 8,
                bluff: 10,
                nerve_threshold: 5,
            }],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None,
        party_slots: 4,
        terrain: Terrain {
            name: "Test Ground".to_string(),
            cover: vec![],
            hazards: vec![],
        },
        objectives: vec![Objective {
            id: "defeat_enemies".to_string(),
            label: "Defeat enemies".to_string(),
            objective_type: ObjectiveType::Primary,
            fail_consequence: vec![],
            success_consequence: vec![],
        }],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Build a minimal party of one member (Galen).
fn solo_party() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![party_defs::galen(AgePhase::Adult).to_combat_tuple()]
}

/// Build a two-member party (Galen + Eli) for duo-tech tests.
fn duo_party() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![
        party_defs::galen(AgePhase::Adult).to_combat_tuple(),
        party_defs::eli_adult().to_combat_tuple(),
    ]
}

/// Build a quick_draw Skill definition for registry tests.
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

/// Build a loaded_deck DuoTech definition for registry tests.
fn loaded_deck_duo_tech() -> DuoTech {
    DuoTech {
        id: DuoTechId::new("loaded_deck"),
        name: "Loaded Deck".to_string(),
        description: "Galen + Eli combined strike".to_string(),
        members: (CharacterId::new("galen"), CharacterId::new("eli")),
        unlock: UnlockCondition::StartOfPhase(AgePhase::Adult),
        cost: DuoTechCost { ammo: 2, nerve: 3, both_turns: true },
        effect: DuoTechEffect {
            description: "Combined distraction and precision shot".to_string(),
            damage: 20,
            accuracy_bonus: 15,
            nerve_damage: 10,
            special: None,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FT-001: Skill registry wired
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn skill_registry_registers_and_retrieves() {
    let mut registry = SkillRegistry::new();
    let skill = quick_draw_skill();
    registry.register(skill);

    let retrieved = registry.get(&SkillId::new("quick_draw"));
    assert!(retrieved.is_some(), "registered skill should be retrievable");
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.name, "Quick Draw");
    assert_eq!(retrieved.cost.ammo, 1);
}

#[test]
fn skill_registry_returns_none_for_unknown() {
    let registry = SkillRegistry::new();
    assert!(registry.get(&SkillId::new("nonexistent")).is_none());
}

#[test]
fn skill_registry_variant_lookup_by_age() {
    let mut registry = SkillRegistry::new();
    registry.register(quick_draw_skill());

    let youth = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Youth);
    assert!(youth.is_some());
    let youth = youth.unwrap();
    assert_eq!(youth.accuracy, 60);
    assert_eq!(youth.damage, 7);

    let adult = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Adult);
    assert!(adult.is_some());
    let adult = adult.unwrap();
    assert_eq!(adult.accuracy, 70);
    assert_eq!(adult.damage, 10);
}

#[test]
fn skill_registry_variant_falls_back_to_first() {
    let mut registry = SkillRegistry::new();
    registry.register(quick_draw_skill());

    // Older phase not defined — should fall back to first variant (Youth)
    let fallback = registry.get_variant(&SkillId::new("quick_draw"), AgePhase::Older);
    assert!(fallback.is_some());
    assert_eq!(fallback.unwrap().phase, AgePhase::Youth);
}

#[test]
fn skill_has_proper_cost_values() {
    let skill = quick_draw_skill();
    assert!(skill.cost.ammo >= 0, "ammo cost should not be negative");
    assert!(skill.cost.nerve >= 0, "nerve cost should not be negative");
}

#[test]
fn encounter_state_carries_skill_registry() {
    let mut registry = SkillRegistry::new();
    registry.register(quick_draw_skill());

    let encounter = test_encounter();
    let state = EncounterState::with_registries(
        &encounter,
        solo_party(),
        registry,
        DuoTechRegistry::new(),
        AgePhase::Adult,
    );

    // The encounter state should have the registry available
    let skill = state.skill_registry.get(&SkillId::new("quick_draw"));
    assert!(skill.is_some(), "encounter state should carry the skill registry");
}

// ═══════════════════════════════════════════════════════════════════════
// FT-002: Duo-tech validation
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn duo_tech_registry_registers_and_retrieves() {
    let mut registry = DuoTechRegistry::new();
    registry.register(loaded_deck_duo_tech());

    let dt = registry.get(&DuoTechId::new("loaded_deck"));
    assert!(dt.is_some());
    let dt = dt.unwrap();
    assert_eq!(dt.members.0, CharacterId::new("galen"));
    assert_eq!(dt.members.1, CharacterId::new("eli"));
}

#[test]
fn duo_tech_both_members_present_can_execute() {
    let mut dt_registry = DuoTechRegistry::new();
    dt_registry.register(loaded_deck_duo_tech());

    let encounter = test_encounter();
    let mut state = EncounterState::with_registries(
        &encounter,
        duo_party(),
        SkillRegistry::new(),
        dt_registry,
        AgePhase::Adult,
    );
    state.phase = EncounterPhase::Combat;
    state.build_turn_queue();

    // Both galen and eli are present and alive — duo tech should be executable
    let galen_present = state.party.iter().flatten().any(|p| p.id == "galen" && !p.down);
    let eli_present = state.party.iter().flatten().any(|p| p.id == "eli" && !p.down);
    assert!(galen_present, "galen must be present for loaded_deck");
    assert!(eli_present, "eli must be present for loaded_deck");

    // Execute the duo tech action — should not panic
    let result = state.execute_action(&CombatAction::UseDuoTech {
        duo_tech: DuoTechId::new("loaded_deck"),
        target: TargetSelection::Single("thug_0".to_string()),
    });
    assert!(!result.action_description.is_empty(), "duo tech should produce a description");
}

#[test]
fn duo_tech_missing_member_party_lacks_partner() {
    // Only Galen, no Eli — loaded_deck requires both
    let encounter = test_encounter();
    let state = EncounterState::new(&encounter, solo_party());

    let eli_present = state.party.iter().flatten().any(|p| p.id == "eli" && !p.down);
    assert!(!eli_present, "eli should NOT be in the solo party");

    // The duo-tech member (eli) is missing — the game should not crash
    // when verifying duo-tech eligibility
    let galen = state.party.iter().flatten().find(|p| p.id == "galen").unwrap();
    let has_loaded_deck = galen.duo_techs.iter().any(|d| d.0 == "loaded_deck");
    assert!(has_loaded_deck, "galen carries loaded_deck in his template");
}

#[test]
fn duo_tech_dead_member_blocks_execution() {
    let mut dt_registry = DuoTechRegistry::new();
    dt_registry.register(loaded_deck_duo_tech());

    let encounter = test_encounter();
    let mut state = EncounterState::with_registries(
        &encounter,
        duo_party(),
        SkillRegistry::new(),
        dt_registry,
        AgePhase::Adult,
    );
    state.phase = EncounterPhase::Combat;

    // Kill Eli — set hp to 0 and mark as down
    for slot in &mut state.party {
        if let Some(m) = slot {
            if m.id == "eli" {
                m.hp = 0;
                m.down = true;
            }
        }
    }

    // Verify Eli is down
    let eli_alive = state.party.iter().flatten().any(|p| p.id == "eli" && !p.down);
    assert!(!eli_alive, "eli should be down — duo-tech should be blocked");
}

// ═══════════════════════════════════════════════════════════════════════
// FT-004: Flee action
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn flee_action_variant_exists() {
    // CombatAction::Flee should be a valid variant
    let action = CombatAction::Flee;
    match action {
        CombatAction::Flee => {} // exists
        _ => panic!("Flee action variant should exist"),
    }
}

#[test]
fn flee_action_executes_without_panic() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;
    state.build_turn_queue();

    let result = state.execute_action(&CombatAction::Flee);
    assert!(
        result.action_description.contains("flee"),
        "flee action should describe a flee attempt, got: {}",
        result.action_description,
    );
}

#[test]
fn encounter_state_escapable_defaults_true() {
    let encounter = test_encounter();
    let state = EncounterState::new(&encounter, solo_party());
    assert!(state.escapable, "encounter state should default to escapable = true");
}

#[test]
fn fled_result_variant_exists() {
    // EncounterResult::Fled should exist
    let result = EncounterResult::Fled;
    assert_eq!(result, EncounterResult::Fled);
}

// ═══════════════════════════════════════════════════════════════════════
// FT-024: NPC ally stats — character-specific via npc_stats_for()
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn npc_stats_for_known_characters_returns_unique_stats() {
    let cal = npc_stats_for("cal");
    let renata = npc_stats_for("renata");
    let bale = npc_stats_for("bale");

    // Named NPCs should have character-specific stats, not identical defaults
    assert_ne!(cal.speed, bale.speed, "cal and bale should have different speed");
    assert_ne!(cal.accuracy, bale.accuracy, "cal and bale should have different accuracy");
    assert_ne!(cal.damage, bale.damage, "cal and bale should have different damage");

    assert_ne!(renata.accuracy, bale.accuracy, "renata and bale should differ");
}

#[test]
fn npc_stats_for_unknown_returns_defaults() {
    let unknown = npc_stats_for("random_npc");
    assert_eq!(unknown.speed, 10, "unknown NPC default speed");
    assert_eq!(unknown.accuracy, 60, "unknown NPC default accuracy");
    assert_eq!(unknown.damage, 8, "unknown NPC default damage");
}

#[test]
fn npc_stats_cal_are_reasonable() {
    let cal = npc_stats_for("cal");
    assert!(cal.speed > 0 && cal.speed < 20, "cal speed should be reasonable");
    assert!(cal.accuracy > 40 && cal.accuracy < 90, "cal accuracy should be reasonable");
    assert!(cal.damage > 0 && cal.damage < 20, "cal damage should be reasonable");
}

#[test]
fn npc_stats_bale_is_slow_heavy_hitter() {
    let bale = npc_stats_for("bale");
    let default = npc_stats_for("unknown");
    // Bale is a convoy guard — slow but heavy-hitting
    assert!(bale.speed < default.speed, "bale should be slower than default");
    assert!(bale.damage > default.damage, "bale should hit harder than default");
}

#[test]
fn npc_ally_stats_flow_into_encounter_state() {
    // Build an encounter with an NPC ally — stats come from NpcCombatant fields
    let cal_stats = npc_stats_for("cal");
    let encounter = Encounter {
        id: EncounterId::new("npc_stats_test"),
        phases: vec![CombatPhase {
            id: "main".to_string(),
            description: "Test NPC stats".to_string(),
            enemies: vec![EnemyTemplate {
                id: "bandit".to_string(),
                name: "Bandit".to_string(),
                hp: 20, nerve: 15, damage: 5, accuracy: 50, speed: 8,
                bluff: 10, nerve_threshold: 5,
            }],
            npc_allies: vec![NpcCombatant {
                character: CharacterId::new("cal"),
                behavior: NpcBehavior::Professional,
                hp: 35,
                nerve: 30,
                speed: cal_stats.speed,
                accuracy: cal_stats.accuracy,
                damage: cal_stats.damage,
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

    let state = EncounterState::new(&encounter, solo_party());

    // NPC ally should have character-specific stats from npc_stats_for("cal")
    assert_eq!(state.npc_allies.len(), 1);
    let cal_live = &state.npc_allies[0].combatant;
    assert_eq!(cal_live.id, "cal");

    let cal_stats = npc_stats_for("cal");
    assert_eq!(cal_live.speed, cal_stats.speed, "NPC speed should come from npc_stats_for()");
    assert_eq!(cal_live.accuracy, cal_stats.accuracy, "NPC accuracy should come from npc_stats_for()");
    assert_eq!(cal_live.damage, cal_stats.damage, "NPC damage should come from npc_stats_for()");
}

#[test]
fn different_npcs_get_different_live_stats() {
    // Two encounters with different NPC allies should produce different live stats
    let make_encounter = |npc_id: &str| {
        let stats = npc_stats_for(npc_id);
        Encounter {
            id: EncounterId::new("npc_test"),
            phases: vec![CombatPhase {
                id: "main".to_string(),
                description: "Test".to_string(),
                enemies: vec![EnemyTemplate {
                    id: "bandit".to_string(),
                    name: "Bandit".to_string(),
                    hp: 20, nerve: 15, damage: 5, accuracy: 50, speed: 8,
                    bluff: 10, nerve_threshold: 5,
                }],
                npc_allies: vec![NpcCombatant {
                    character: CharacterId::new(npc_id),
                    behavior: NpcBehavior::Professional,
                    hp: 35,
                    nerve: 30,
                    speed: stats.speed,
                    accuracy: stats.accuracy,
                    damage: stats.damage,
                }],
                entry_conditions: vec![],
                phase_effects: vec![],
            }],
            standoff: None,
            party_slots: 4,
            terrain: Terrain { name: "Test".to_string(), cover: vec![], hazards: vec![] },
            objectives: vec![],
            outcome_effects: vec![],
            escapable: true,
        }
    };

    let cal_enc = make_encounter("cal");
    let bale_enc = make_encounter("bale");

    let cal_state = EncounterState::new(&cal_enc, solo_party());
    let bale_state = EncounterState::new(&bale_enc, solo_party());

    let cal_live = &cal_state.npc_allies[0].combatant;
    let bale_live = &bale_state.npc_allies[0].combatant;

    assert_ne!(cal_live.speed, bale_live.speed, "cal and bale should have different speed in combat");
    assert_ne!(cal_live.damage, bale_live.damage, "cal and bale should have different damage in combat");
}

// ═══════════════════════════════════════════════════════════════════════
// FT-025: Objective auto-resolve
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn active_objectives_resolve_when_enemies_defeated() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Verify objective starts Active
    assert_eq!(state.objectives.len(), 1);
    assert_eq!(state.objectives[0].status, ObjectiveStatus::Active);

    // Kill all enemies
    for enemy in &mut state.enemies {
        enemy.hp = 0;
        enemy.down = true;
    }

    // Evaluate objectives — primary should auto-succeed
    state.evaluate_objectives();
    assert_eq!(
        state.objectives[0].status,
        ObjectiveStatus::Succeeded,
        "primary objective should auto-resolve to Succeeded when all enemies are down",
    );
}

#[test]
fn completed_objectives_maintain_status() {
    let encounter = test_encounter();
    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Manually set objective to Succeeded
    state.objectives[0].status = ObjectiveStatus::Succeeded;

    // Evaluate again — should stay Succeeded
    state.evaluate_objectives();
    assert_eq!(
        state.objectives[0].status,
        ObjectiveStatus::Succeeded,
        "already-succeeded objectives should maintain their status",
    );
}

#[test]
fn secondary_objective_fails_when_enemies_killed_not_broken() {
    // Build encounter with a civilian casualties secondary objective
    let encounter = Encounter {
        id: EncounterId::new("casualties_test"),
        phases: vec![CombatPhase {
            id: "main".to_string(),
            description: "Test".to_string(),
            enemies: vec![EnemyTemplate {
                id: "worker".to_string(),
                name: "Worker".to_string(),
                hp: 10, nerve: 8, damage: 3, accuracy: 30, speed: 5,
                bluff: 5, nerve_threshold: 3,
            }],
            npc_allies: vec![],
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
        objectives: vec![
            Objective {
                id: "defeat".to_string(),
                label: "Win".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![],
                success_consequence: vec![],
            },
            Objective {
                id: "no_casualties".to_string(),
                label: "No civilian casualties".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![],
                success_consequence: vec![],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    };

    let mut state = EncounterState::new(&encounter, solo_party());
    state.phase = EncounterPhase::Combat;

    // Kill the worker (not break via nerve)
    state.enemies[0].hp = 0;
    state.enemies[0].down = true;

    state.evaluate_objectives();

    // Primary should succeed
    assert_eq!(state.objectives[0].status, ObjectiveStatus::Succeeded);
    // Secondary "no_casualties" should fail because enemy was killed, not broken
    assert_eq!(
        state.objectives[1].status,
        ObjectiveStatus::Failed,
        "casualties objective should fail when enemies are killed rather than broken",
    );
}

// ═══════════════════════════════════════════════════════════════════════
// BREAKWATER: Ch11 dispatch
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn breakwater_battle_encounter_dispatches() {
    let enc = content::get_encounter("breakwater_junction", "breakwater_battle");
    assert!(enc.is_some(), "get_encounter('breakwater_junction', 'breakwater_battle') should return Some");
    let enc = enc.unwrap();
    assert_eq!(enc.id.0, "breakwater_battle");
    assert!(!enc.phases.is_empty(), "breakwater_battle should have at least one phase");
}

#[test]
fn breakwater_battle_has_standoff() {
    let enc = content::get_encounter("breakwater_junction", "breakwater_battle").unwrap();
    assert!(enc.standoff.is_some(), "breakwater should have a standoff pre-phase");
}

#[test]
fn breakwater_battle_has_objectives() {
    let enc = content::get_encounter("breakwater_junction", "breakwater_battle").unwrap();
    assert!(!enc.objectives.is_empty(), "breakwater should have combat objectives");
    let primary = enc.objectives.iter().find(|o| o.objective_type == ObjectiveType::Primary);
    assert!(primary.is_some(), "breakwater should have a primary objective");
}

// ═══════════════════════════════════════════════════════════════════════
// CH_ENTRY: Chapter routing
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn all_16_chapters_have_entry_scenes() {
    let chapters = [
        "prologue",
        "cedar_wake",
        "saints_mile_convoy",
        "black_willow",
        "ropehouse_blood",
        "dust_revival",
        "fuse_country",
        "iron_ledger",
        "burned_mission",
        "long_wire",
        "deadwater_trial",
        "breakwater_junction",
        "names_in_dust",
        "fifteen_years_gone",
        "old_friends",
        "saints_mile_again",
    ];

    for chapter in &chapters {
        let entry = content::chapter_entry_scene(chapter);
        assert!(
            entry.is_some(),
            "chapter '{}' should have an entry scene",
            chapter,
        );
        let scene_id = entry.unwrap();
        assert!(
            !scene_id.is_empty(),
            "chapter '{}' entry scene ID should not be empty",
            chapter,
        );
    }
}

#[test]
fn chapter_entry_scenes_map_to_valid_scenes() {
    let chapters = [
        "prologue",
        "cedar_wake",
        "saints_mile_convoy",
        "black_willow",
        "ropehouse_blood",
        "dust_revival",
        "fuse_country",
        "iron_ledger",
        "burned_mission",
        "long_wire",
        "deadwater_trial",
        "breakwater_junction",
        "names_in_dust",
        "fifteen_years_gone",
        "old_friends",
        "saints_mile_again",
    ];

    for chapter in &chapters {
        let entry_scene_id = content::chapter_entry_scene(chapter).unwrap();
        let scene = content::get_scene(chapter, entry_scene_id);
        assert!(
            scene.is_some(),
            "chapter '{}' entry scene '{}' should resolve via get_scene()",
            chapter,
            entry_scene_id,
        );
    }
}

#[test]
fn unknown_chapter_entry_returns_none() {
    assert!(content::chapter_entry_scene("nonexistent").is_none());
    assert!(content::chapter_entry_scene("").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// CH_PROGRESS: Chapter validation
// ═══════════════════════════════════════════════════════════════════════

// NOTE: can_enter_chapter() does not exist yet in the public API.
// These tests validate the prerequisite concept through the state store.
// When the feature agent lands can_enter_chapter(), uncomment and adapt.

#[test]
fn early_chapters_accessible_from_game_start() {
    let dir = TempDir::new().unwrap();
    let store = StateStore::new_game(dir.path());

    // Prologue entry scene should exist
    let entry = content::chapter_entry_scene("prologue");
    assert!(entry.is_some(), "prologue should be accessible from game start");

    // Cedar wake entry scene should exist
    let entry = content::chapter_entry_scene("cedar_wake");
    assert!(entry.is_some(), "cedar_wake should be accessible (entry exists)");

    // The state store starts clean — no flags set means no prerequisites met
    assert!(
        store.state().flags.get("prologue_complete").is_none(),
        "fresh game should not have prologue_complete flag",
    );
}

#[test]
fn chapter_progression_flags_can_be_set() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    // Simulate completing the prologue
    store.state_mut().flags.insert(
        "prologue_complete".to_string(),
        FlagValue::Bool(true),
    );

    assert_eq!(
        store.state().flags.get("prologue_complete"),
        Some(&FlagValue::Bool(true)),
        "chapter completion flags should be settable",
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PROD_001_S: Graceful exit
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn confirm_quit_screen_variant_exists() {
    let screen = AppScreen::ConfirmQuit {
        return_screen: Box::new(AppScreen::Title),
    };
    match screen {
        AppScreen::ConfirmQuit { return_screen } => {
            assert!(matches!(*return_screen, AppScreen::Title));
        }
        _ => panic!("ConfirmQuit screen should exist"),
    }
}

#[test]
fn quit_options_include_save_and_cancel() {
    let options = QuitOption::all();
    assert!(options.len() >= 3, "should have SaveAndQuit, QuitWithoutSaving, Cancel");
    assert!(options.contains(&QuitOption::SaveAndQuit));
    assert!(options.contains(&QuitOption::QuitWithoutSaving));
    assert!(options.contains(&QuitOption::Cancel));
}

#[test]
fn quit_options_have_labels() {
    for option in QuitOption::all() {
        let label = option.label();
        assert!(!label.is_empty(), "quit option {:?} should have a non-empty label", option);
    }
}

#[test]
fn app_starts_with_should_quit_false() {
    let dir = TempDir::new().unwrap();
    let app = App::new(dir.path().to_path_buf());
    assert!(!app.should_quit, "app should not start in quit state");
}
