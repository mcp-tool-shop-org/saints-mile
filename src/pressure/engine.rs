//! Pressure engine — runtime for nonstandard encounters.
//!
//! FT-003: The pressure system was schema-only. This module provides
//! a PressureEngine that can run a pressure encounter to completion:
//! track bars, process actions, check thresholds, resolve outcomes.

use tracing::{debug, info};

use super::types::*;
use crate::scene::types::StateEffect;

/// Result of processing a single pressure action.
#[derive(Debug, Clone)]
pub struct PressureActionResult {
    pub actor: String,
    pub action_label: String,
    pub bar_id: String,
    pub delta: i32,
    pub bar_after: i32,
    pub description: String,
}

/// How the pressure encounter ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureOutcome {
    Success,
    Failure,
    /// Still in progress.
    InProgress,
}

/// Result of checking thresholds after an action.
#[derive(Debug, Clone)]
pub struct PressureResolution {
    pub outcome: PressureOutcome,
    pub effects: Vec<StateEffect>,
    /// Which bar triggered the resolution, if any.
    pub trigger_bar: Option<String>,
}

/// Live state of a pressure encounter in progress.
#[derive(Debug)]
pub struct PressureEngine {
    /// The encounter definition.
    pub encounter: PressureEncounter,
    /// Current round number.
    pub round: u32,
    /// Whether the encounter is resolved.
    pub outcome: PressureOutcome,
    /// Accumulated state effects to apply after resolution.
    pub pending_effects: Vec<StateEffect>,
    /// Cargo integrity tracking (for Escort type).
    pub cargo_state: Vec<CargoLiveState>,
}

/// Live state of a cargo item during an Escort pressure encounter.
#[derive(Debug, Clone)]
pub struct CargoLiveState {
    pub id: String,
    pub name: String,
    pub integrity: i32,
    pub max_integrity: i32,
    pub lost: bool,
    pub loss_effects: Vec<StateEffect>,
}

impl PressureEngine {
    /// Create a new pressure engine from an encounter definition.
    pub fn new(encounter: PressureEncounter) -> Self {
        let cargo_state = match &encounter.pressure_type {
            PressureType::Escort { cargo } => {
                cargo.iter().map(|c| CargoLiveState {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    integrity: c.integrity,
                    max_integrity: c.max_integrity,
                    lost: false,
                    loss_effects: c.loss_effect.clone(),
                }).collect()
            }
            _ => Vec::new(),
        };

        PressureEngine {
            encounter,
            round: 0,
            outcome: PressureOutcome::InProgress,
            pending_effects: Vec::new(),
            cargo_state,
        }
    }

    /// Start a new round. Returns the round number.
    pub fn begin_round(&mut self) -> u32 {
        self.round += 1;
        debug!(round = self.round, "pressure round started");
        self.round
    }

    /// Get the current value of a pressure bar.
    pub fn get_bar(&self, bar_id: &str) -> Option<&PressureBar> {
        self.encounter.pressure_bars.iter().find(|b| b.id == bar_id)
    }

    /// Get a mutable reference to a pressure bar.
    fn get_bar_mut(&mut self, bar_id: &str) -> Option<&mut PressureBar> {
        self.encounter.pressure_bars.iter_mut().find(|b| b.id == bar_id)
    }

    /// Get all available actions for a character this round.
    pub fn available_actions(&self, character_id: &str) -> Vec<&PressureAction> {
        self.encounter.party_actions.iter()
            .filter(|pa| pa.character.0 == character_id)
            .flat_map(|pa| pa.actions.iter())
            .collect()
    }

    /// Process a character's chosen action. Returns the result.
    pub fn process_action(
        &mut self,
        character_id: &str,
        action_id: &str,
    ) -> Option<PressureActionResult> {
        if self.outcome != PressureOutcome::InProgress {
            return None;
        }

        // Find the action
        let action = self.encounter.party_actions.iter()
            .filter(|pa| pa.character.0 == character_id)
            .flat_map(|pa| pa.actions.iter())
            .find(|a| a.id == action_id)?
            .clone();

        // Apply delta to the target bar
        let bar = self.get_bar_mut(&action.target_bar)?;
        bar.current = (bar.current + action.delta).clamp(0, bar.max);
        let bar_after = bar.current;
        let bar_id = bar.id.clone();

        info!(
            character = character_id,
            action = action_id,
            bar = %bar_id,
            delta = action.delta,
            bar_after = bar_after,
            "pressure action processed"
        );

        Some(PressureActionResult {
            actor: character_id.to_string(),
            action_label: action.label.clone(),
            bar_id,
            delta: action.delta,
            bar_after,
            description: action.description.clone(),
        })
    }

    /// Damage a cargo item (for Escort encounters). Returns true if the item was lost.
    pub fn damage_cargo(&mut self, cargo_id: &str, damage: i32) -> bool {
        if let Some(cargo) = self.cargo_state.iter_mut().find(|c| c.id == cargo_id && !c.lost) {
            cargo.integrity = (cargo.integrity - damage).max(0);
            if cargo.integrity == 0 {
                cargo.lost = true;
                self.pending_effects.extend(cargo.loss_effects.clone());
                debug!(cargo = cargo_id, "cargo lost");
                return true;
            }
        }
        false
    }

    /// Check success/failure thresholds. Call after each action or event.
    pub fn check_thresholds(&mut self) -> PressureResolution {
        if self.outcome != PressureOutcome::InProgress {
            return PressureResolution {
                outcome: self.outcome,
                effects: Vec::new(),
                trigger_bar: None,
            };
        }

        // Check failure threshold first (failure takes priority)
        let failed = self.check_condition(&self.encounter.failure_threshold.clone());
        if let Some(bar_id) = failed {
            self.outcome = PressureOutcome::Failure;
            self.pending_effects.extend(self.encounter.outcome_effects.clone());
            info!(trigger = %bar_id, "pressure encounter FAILED");
            return PressureResolution {
                outcome: PressureOutcome::Failure,
                effects: self.pending_effects.clone(),
                trigger_bar: Some(bar_id),
            };
        }

        // Check success threshold
        let succeeded = self.check_condition(&self.encounter.success_threshold.clone());
        if let Some(bar_id) = succeeded {
            self.outcome = PressureOutcome::Success;
            self.pending_effects.extend(self.encounter.outcome_effects.clone());
            info!(trigger = %bar_id, "pressure encounter SUCCEEDED");
            return PressureResolution {
                outcome: PressureOutcome::Success,
                effects: self.pending_effects.clone(),
                trigger_bar: Some(bar_id),
            };
        }

        PressureResolution {
            outcome: PressureOutcome::InProgress,
            effects: Vec::new(),
            trigger_bar: None,
        }
    }

    /// Check if a condition is met. Returns the triggering bar ID if so.
    fn check_condition(&self, condition: &PressureCondition) -> Option<String> {
        match condition {
            PressureCondition::BarReached { bar_id, threshold } => {
                self.encounter.pressure_bars.iter()
                    .find(|b| b.id == *bar_id && b.current <= *threshold)
                    .map(|b| b.id.clone())
            }
            PressureCondition::AllBarsAboveFail => {
                // Success: all bars are above their fail_at threshold
                let all_above = self.encounter.pressure_bars.iter()
                    .all(|b| b.current > b.fail_at);
                if all_above {
                    Some("all_bars".to_string())
                } else {
                    None
                }
            }
            PressureCondition::TimeExpired => {
                // Not yet implemented — needs a time/round limit system
                None
            }
            PressureCondition::FlagSet(_flag_id) => {
                // Requires game state integration — stub for now
                None
            }
        }
    }

    /// Whether the encounter is still in progress.
    pub fn is_active(&self) -> bool {
        self.outcome == PressureOutcome::InProgress
    }

    /// Get a summary of all bar states for display.
    pub fn bar_summary(&self) -> Vec<BarStatus> {
        self.encounter.pressure_bars.iter()
            .filter(|b| b.visible)
            .map(|b| BarStatus {
                id: b.id.clone(),
                label: b.label.clone(),
                current: b.current,
                max: b.max,
                fail_at: b.fail_at,
                critical: b.current <= b.fail_at + (b.max / 5), // within 20% of fail
            })
            .collect()
    }
}

/// A bar's current status for UI display.
#[derive(Debug, Clone)]
pub struct BarStatus {
    pub id: String,
    pub label: String,
    pub current: i32,
    pub max: i32,
    pub fail_at: i32,
    pub critical: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn escort_encounter() -> PressureEncounter {
        PressureEncounter {
            id: "ch2_convoy_escort".to_string(),
            pressure_type: PressureType::Escort {
                cargo: vec![
                    CargoItem {
                        id: "medicine_crate".to_string(),
                        name: "Medicine Crate".to_string(),
                        integrity: 20,
                        max_integrity: 20,
                        loss_effect: vec![
                            StateEffect::SetFlag {
                                id: FlagId::new("medicine_lost"),
                                value: FlagValue::Bool(true),
                            },
                        ],
                    },
                    CargoItem {
                        id: "supplies".to_string(),
                        name: "Supply Wagon".to_string(),
                        integrity: 30,
                        max_integrity: 30,
                        loss_effect: vec![],
                    },
                ],
            },
            pressure_bars: vec![
                PressureBar {
                    id: "convoy_safety".to_string(),
                    label: "Convoy Safety".to_string(),
                    current: 80,
                    max: 100,
                    fail_at: 20,
                    visible: true,
                },
                PressureBar {
                    id: "bandit_pressure".to_string(),
                    label: "Bandit Threat".to_string(),
                    current: 40,
                    max: 100,
                    fail_at: 0,
                    visible: true,
                },
            ],
            party_actions: vec![
                PressurePartyAction {
                    character: CharacterId::new("galen"),
                    actions: vec![
                        PressureAction {
                            id: "scout_ahead".to_string(),
                            label: "Scout Ahead".to_string(),
                            description: "Ride ahead to check the trail.".to_string(),
                            target_bar: "convoy_safety".to_string(),
                            delta: 10,
                            conditions: vec![],
                        },
                        PressureAction {
                            id: "suppressive_fire".to_string(),
                            label: "Suppressive Fire".to_string(),
                            description: "Lay down covering fire on the ridge.".to_string(),
                            target_bar: "bandit_pressure".to_string(),
                            delta: -15,
                            conditions: vec![],
                        },
                    ],
                },
                PressurePartyAction {
                    character: CharacterId::new("eli"),
                    actions: vec![
                        PressureAction {
                            id: "talk_down".to_string(),
                            label: "Talk Them Down".to_string(),
                            description: "Try to convince the bandits to back off.".to_string(),
                            target_bar: "bandit_pressure".to_string(),
                            delta: -10,
                            conditions: vec![],
                        },
                        PressureAction {
                            id: "check_cargo".to_string(),
                            label: "Check Cargo".to_string(),
                            description: "Secure the wagon and check for damage.".to_string(),
                            target_bar: "convoy_safety".to_string(),
                            delta: 5,
                            conditions: vec![],
                        },
                    ],
                },
            ],
            success_threshold: PressureCondition::AllBarsAboveFail,
            failure_threshold: PressureCondition::BarReached {
                bar_id: "convoy_safety".to_string(),
                threshold: 20,
            },
            outcome_effects: vec![
                StateEffect::SetFlag {
                    id: FlagId::new("convoy_complete"),
                    value: FlagValue::Bool(true),
                },
            ],
        }
    }

    #[test]
    fn pressure_engine_creates_from_encounter() {
        let enc = escort_encounter();
        let engine = PressureEngine::new(enc);

        assert_eq!(engine.round, 0);
        assert_eq!(engine.outcome, PressureOutcome::InProgress);
        assert_eq!(engine.cargo_state.len(), 2);
        assert!(engine.is_active());
    }

    #[test]
    fn pressure_action_modifies_bar() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);
        engine.begin_round();

        let result = engine.process_action("galen", "scout_ahead");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.bar_id, "convoy_safety");
        assert_eq!(result.delta, 10);
        assert_eq!(result.bar_after, 90); // 80 + 10

        let bar = engine.get_bar("convoy_safety").unwrap();
        assert_eq!(bar.current, 90);
    }

    #[test]
    fn pressure_bar_clamps_to_max() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);
        engine.begin_round();

        // Scout ahead 3 times to try to exceed max
        engine.process_action("galen", "scout_ahead");
        engine.process_action("galen", "scout_ahead");
        engine.process_action("galen", "scout_ahead");

        let bar = engine.get_bar("convoy_safety").unwrap();
        assert_eq!(bar.current, 100); // clamped to max
    }

    #[test]
    fn pressure_failure_triggers_on_bar_threshold() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);
        engine.begin_round();

        // Directly reduce convoy_safety below fail_at (20)
        // Start at 80, need to reduce by 65 to reach 15
        {
            let bar = engine.get_bar_mut("convoy_safety").unwrap();
            bar.current = 15; // below fail_at of 20
        }

        let resolution = engine.check_thresholds();
        assert_eq!(resolution.outcome, PressureOutcome::Failure);
        assert!(resolution.trigger_bar.is_some());
        assert_eq!(resolution.trigger_bar.unwrap(), "convoy_safety");
        assert!(!engine.is_active());
    }

    #[test]
    fn pressure_success_when_all_bars_above_fail() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);
        engine.begin_round();

        // Both bars start above fail_at: convoy_safety=80 (fail=20), bandit_pressure=40 (fail=0)
        let resolution = engine.check_thresholds();
        assert_eq!(resolution.outcome, PressureOutcome::Success);
        assert!(!engine.is_active());
    }

    #[test]
    fn cargo_damage_and_loss() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);

        // Damage medicine crate
        let lost = engine.damage_cargo("medicine_crate", 15);
        assert!(!lost); // 20 - 15 = 5, still alive
        assert_eq!(engine.cargo_state[0].integrity, 5);

        // Finish it off
        let lost = engine.damage_cargo("medicine_crate", 10);
        assert!(lost);
        assert_eq!(engine.cargo_state[0].integrity, 0);
        assert!(engine.cargo_state[0].lost);
        assert!(!engine.pending_effects.is_empty()); // loss effects applied
    }

    #[test]
    fn available_actions_per_character() {
        let enc = escort_encounter();
        let engine = PressureEngine::new(enc);

        let galen_actions = engine.available_actions("galen");
        assert_eq!(galen_actions.len(), 2);

        let eli_actions = engine.available_actions("eli");
        assert_eq!(eli_actions.len(), 2);

        let nobody_actions = engine.available_actions("nobody");
        assert_eq!(nobody_actions.len(), 0);
    }

    #[test]
    fn bar_summary_only_shows_visible() {
        let enc = escort_encounter();
        let engine = PressureEngine::new(enc);

        let summary = engine.bar_summary();
        assert_eq!(summary.len(), 2); // both visible

        // Check critical state — 80 is not critical for fail_at=20 (20% threshold = 20+20=40)
        assert!(!summary[0].critical); // convoy_safety at 80, critical threshold ~40
    }

    #[test]
    fn actions_blocked_after_resolution() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);

        // Force resolution
        engine.outcome = PressureOutcome::Success;

        let result = engine.process_action("galen", "scout_ahead");
        assert!(result.is_none(), "actions should be blocked after resolution");
    }

    #[test]
    fn negative_delta_reduces_bar() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);
        engine.begin_round();

        let result = engine.process_action("galen", "suppressive_fire");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.bar_id, "bandit_pressure");
        assert_eq!(result.delta, -15);
        assert_eq!(result.bar_after, 25); // 40 - 15
    }

    #[test]
    fn full_pressure_round_flow() {
        let enc = escort_encounter();
        let mut engine = PressureEngine::new(enc);

        // Round 1
        let round = engine.begin_round();
        assert_eq!(round, 1);

        // Galen scouts, Eli checks cargo
        engine.process_action("galen", "scout_ahead");
        engine.process_action("eli", "check_cargo");

        // Check — should succeed (all bars above fail)
        let res = engine.check_thresholds();
        assert_eq!(res.outcome, PressureOutcome::Success);
    }
}
