//! AUDIT-006 — Scene runner unit tests.
//!
//! Validates condition evaluation, state effect application,
//! and choice filtering across the scene system.

mod common;

use saints_mile::types::*;
use saints_mile::scene::types::*;
use saints_mile::scene::runner::SceneRunner;
use saints_mile::state::store::StateStore;
use saints_mile::state::types::GameState;
use tempfile::TempDir;

// ─── Helpers ─────────────────────────────────────────────────────

/// Build a test scene with conditional choices and lines.
fn scene_with_conditions() -> Scene {
    Scene {
        id: SceneId::new("cond_test"),
        location: LocationId::new("test"),
        beat: BeatId::new("t1"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Always visible.".to_string(),
                conditions: vec![],
                emotion: None,
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Only when Tom branch.".to_string(),
                conditions: vec![Condition::RelayBranch(RelayBranch::Tom)],
                emotion: None,
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Only in Youth.".to_string(),
                conditions: vec![Condition::PrologueChoice(PrologueChoice::HomesteadFirst)],
                emotion: None,
            },
        ],
        choices: vec![
            Choice {
                label: "Open choice".to_string(),
                conditions: vec![],
                effects: vec![],
                next: SceneTransition::End,
            },
            Choice {
                label: "Requires flag".to_string(),
                conditions: vec![Condition::Flag {
                    id: FlagId::new("needs_this"),
                    value: FlagValue::Bool(true),
                }],
                effects: vec![],
                next: SceneTransition::End,
            },
            Choice {
                label: "Requires party member".to_string(),
                conditions: vec![Condition::PartyMember {
                    character: CharacterId::new("ada"),
                    present: true,
                }],
                effects: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("ada_helped"),
                        value: FlagValue::Bool(true),
                    },
                ],
                next: SceneTransition::End,
            },
            Choice {
                label: "Requires reputation".to_string(),
                conditions: vec![Condition::Reputation {
                    axis: ReputationAxis::TownLaw,
                    op: CompareOp::Gte,
                    threshold: 10,
                }],
                effects: vec![],
                next: SceneTransition::End,
            },
        ],
        conditions: vec![],
        state_effects: vec![],
        pacing: PacingTag::Exploration,
        memory_refs: vec![],
    }
}

// ─── Condition Evaluation ────────────────────────────────────────

/// Flag condition: true when flag matches, false otherwise.
#[test]
fn condition_flag_evaluation() {
    let mut state = GameState::new_game();
    let cond = Condition::Flag {
        id: FlagId::new("test_flag"),
        value: FlagValue::Bool(true),
    };

    assert!(!state.check_condition(&cond), "flag not set yet");

    state.apply_effect(&StateEffect::SetFlag {
        id: FlagId::new("test_flag"),
        value: FlagValue::Bool(true),
    });
    assert!(state.check_condition(&cond), "flag now set to true");

    // Wrong value
    let wrong = Condition::Flag {
        id: FlagId::new("test_flag"),
        value: FlagValue::Bool(false),
    };
    assert!(!state.check_condition(&wrong));
}

/// Age phase condition via PrologueChoice.
#[test]
fn condition_prologue_choice_evaluation() {
    let mut state = GameState::new_game();

    let cond_home = Condition::PrologueChoice(PrologueChoice::HomesteadFirst);
    let cond_town = Condition::PrologueChoice(PrologueChoice::TownDirect);

    assert!(!state.check_condition(&cond_home));
    assert!(!state.check_condition(&cond_town));

    state.prologue_choice = Some(PrologueChoice::HomesteadFirst);
    assert!(state.check_condition(&cond_home));
    assert!(!state.check_condition(&cond_town));
}

/// Relay branch condition evaluates correctly.
#[test]
fn condition_relay_branch_evaluation() {
    let mut state = GameState::new_game();

    assert!(!state.check_condition(&Condition::RelayBranch(RelayBranch::Tom)));
    assert!(!state.check_condition(&Condition::RelayBranch(RelayBranch::Nella)));
    assert!(!state.check_condition(&Condition::RelayBranch(RelayBranch::Papers)));

    state.relay_branch = Some(RelayBranch::Papers);
    assert!(state.check_condition(&Condition::RelayBranch(RelayBranch::Papers)));
    assert!(!state.check_condition(&Condition::RelayBranch(RelayBranch::Tom)));
}

/// Reputation condition with all compare operators.
#[test]
fn condition_reputation_all_operators() {
    let mut state = GameState::new_game();
    state.apply_effect(&StateEffect::AdjustReputation {
        axis: ReputationAxis::Railroad,
        delta: 15,
    });

    assert!(state.check_condition(&Condition::Reputation {
        axis: ReputationAxis::Railroad, op: CompareOp::Gt, threshold: 10,
    }));
    assert!(state.check_condition(&Condition::Reputation {
        axis: ReputationAxis::Railroad, op: CompareOp::Gte, threshold: 15,
    }));
    assert!(!state.check_condition(&Condition::Reputation {
        axis: ReputationAxis::Railroad, op: CompareOp::Lt, threshold: 15,
    }));
    assert!(state.check_condition(&Condition::Reputation {
        axis: ReputationAxis::Railroad, op: CompareOp::Lte, threshold: 15,
    }));
    assert!(state.check_condition(&Condition::Reputation {
        axis: ReputationAxis::Railroad, op: CompareOp::Eq, threshold: 15,
    }));
    assert!(state.check_condition(&Condition::Reputation {
        axis: ReputationAxis::Railroad, op: CompareOp::Neq, threshold: 10,
    }));
}

// ─── State Effect Application ────────────────────────────────────

/// SetFlag effect works for bool and text values.
#[test]
fn effect_set_flag() {
    let mut state = GameState::new_game();

    state.apply_effect(&StateEffect::SetFlag {
        id: FlagId::new("bool_flag"),
        value: FlagValue::Bool(true),
    });
    assert_eq!(state.flags.get("bool_flag"), Some(&FlagValue::Bool(true)));

    state.apply_effect(&StateEffect::SetFlag {
        id: FlagId::new("text_flag"),
        value: FlagValue::Text("hello".to_string()),
    });
    assert_eq!(
        state.flags.get("text_flag"),
        Some(&FlagValue::Text("hello".to_string())),
    );
}

/// GainSkill (UnlockSkill) adds skill to character and survives check.
#[test]
fn effect_unlock_skill() {
    let mut state = GameState::new_game();

    assert!(!state.party.has_skill(
        &CharacterId::new("galen"),
        &SkillId::new("cold_read"),
    ));

    state.apply_effect(&StateEffect::UnlockSkill {
        character: CharacterId::new("galen"),
        skill: SkillId::new("cold_read"),
    });

    assert!(state.party.has_skill(
        &CharacterId::new("galen"),
        &SkillId::new("cold_read"),
    ));

    // Idempotent — adding again doesn't duplicate
    state.apply_effect(&StateEffect::UnlockSkill {
        character: CharacterId::new("galen"),
        skill: SkillId::new("cold_read"),
    });
    let galen = state.party.members.iter().find(|m| m.id.0 == "galen").unwrap();
    let cold_read_count = galen.unlocked_skills.iter()
        .filter(|s| s.0 == "cold_read").count();
    assert_eq!(cold_read_count, 1, "skill should not be duplicated");
}

/// Relationship changes are bidirectional-keyed.
#[test]
fn effect_set_relationship() {
    let mut state = GameState::new_game();

    state.apply_effect(&StateEffect::SetRelationship {
        a: CharacterId::new("galen"),
        b: CharacterId::new("eli"),
        value: 15,
    });

    assert_eq!(
        state.party.relationships.get("galen:eli"),
        Some(&15),
    );
}

// ─── Choice Filtering ────────────────────────────────────────────

/// Choices with unmet conditions are visible but locked.
#[test]
fn choice_filtering_locked_vs_available() {
    let dir = TempDir::new().unwrap();
    let store = StateStore::new_game(dir.path());
    let scene = scene_with_conditions();

    let choices = SceneRunner::evaluate_choices(&scene, &store);

    // Choice 0 (open) — available
    assert!(choices[0].available);
    assert!(choices[0].lock_reason.is_none());

    // Choice 1 (requires flag) — locked
    assert!(!choices[1].available);
    assert!(choices[1].lock_reason.is_some());

    // Choice 2 (requires ada in party) — locked
    assert!(!choices[2].available);

    // Choice 3 (requires reputation >= 10) — locked
    assert!(!choices[3].available);
}

/// Setting the required flag unlocks the gated choice.
#[test]
fn choice_unlocked_by_flag() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    store.apply_effect(&StateEffect::SetFlag {
        id: FlagId::new("needs_this"),
        value: FlagValue::Bool(true),
    });

    let scene = scene_with_conditions();
    let choices = SceneRunner::evaluate_choices(&scene, &store);

    assert!(choices[1].available, "flag-gated choice should now be available");
}

/// Adding a party member unlocks the party-gated choice.
#[test]
fn choice_unlocked_by_party_member() {
    let dir = TempDir::new().unwrap();
    let mut store = StateStore::new_game(dir.path());

    store.apply_effect(&StateEffect::AddPartyMember(CharacterId::new("ada")));

    let scene = scene_with_conditions();
    let choices = SceneRunner::evaluate_choices(&scene, &store);

    assert!(choices[2].available, "party-gated choice should be available with ada present");
}

/// Scene-level conditions gate the entire scene.
#[test]
fn scene_level_condition_gates_entire_scene() {
    let dir = TempDir::new().unwrap();
    let store = StateStore::new_game(dir.path());

    let scene = Scene {
        id: SceneId::new("gated"),
        location: LocationId::new("test"),
        beat: BeatId::new("t1"),
        lines: vec![SceneLine {
            speaker: SpeakerId::new("narrator"),
            text: "Should not play.".to_string(),
            conditions: vec![],
            emotion: None,
        }],
        choices: vec![],
        conditions: vec![Condition::Flag {
            id: FlagId::new("gate_flag"),
            value: FlagValue::Bool(true),
        }],
        state_effects: vec![],
        pacing: PacingTag::Exploration,
        memory_refs: vec![],
    };

    let prepared = SceneRunner::prepare_scene(&scene, &store);
    assert!(!prepared.should_play, "scene with unmet condition should not play");
    assert!(prepared.lines.is_empty());
}
