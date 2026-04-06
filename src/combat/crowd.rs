//! Crowd pressure system — collective nerve bar, escalation, party action hooks.
//!
//! The crowd is not enemies with a shared HP bar. It is a room-state that
//! the party manages through different vectors: rhetoric, physical presence,
//! medical authority, misdirection, and command.
//!
//! Victory is not killing. Victory is containment.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::types::*;

// ─── Crowd Action Nerve Restoration Constants ─────────────────────
// Each value represents how much collective nerve a specific action
// type restores. These are narratively motivated:

/// Miriam's Hymn/Sermon — the most effective broad calming action.
/// Faith-based authority carries weight in a frontier crowd.
const BROAD_CALM_RESTORE: i32 = 8;

/// Ada's medical evidence — defuses supernatural panic with facts.
/// Less effective than faith but builds lasting trust.
const MEDICAL_AUTHORITY_RESTORE: i32 = 6;

/// Eli's misdirection — redirects crowd anger to a different target.
/// Effective but morally ugly; doesn't improve momentum.
const REDIRECT_RESTORE: i32 = 7;

/// Live state of a crowd pressure encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrowdState {
    /// Collective nerve bar — when it hits 0, the crowd becomes a mob.
    pub collective_nerve: i32,
    pub max_nerve: i32,

    /// Crowd momentum — positive = calming, negative = escalating.
    /// Affected by party actions each turn.
    pub momentum: i32,

    /// Current crowd phase.
    pub phase: CrowdPhase,

    /// Ringleaders — specific instigators whose nerve can be targeted individually.
    pub ringleaders: Vec<Ringleader>,

    /// Whether the protected target (Dunnick family etc.) is safe.
    pub target_safe: bool,

    /// Turn counter.
    pub turn: u32,

    /// How many turns until the crowd reaches the target if not contained.
    pub surge_countdown: u32,
}

/// The phases a crowd moves through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrowdPhase {
    /// Crowd is tense but containable.
    Tense,
    /// Crowd is surging — momentum toward violence.
    Surging,
    /// Crowd has been calmed — de-escalation holding.
    Calming,
    /// Crowd broke — mob violence, target at risk.
    Broken,
    /// Crowd dispersed — containment successful.
    Dispersed,
}

/// A ringleader in the crowd — individual nerve target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ringleader {
    pub id: String,
    pub name: String,
    pub nerve: i32,
    pub influence: i32,  // how much they affect collective nerve when broken
    pub broken: bool,
}

/// An action a party member can take in crowd pressure.
#[derive(Debug, Clone)]
pub struct CrowdAction {
    pub actor: String,
    pub action_type: CrowdActionType,
    pub target: Option<String>,  // ringleader ID if targeted
}

/// The types of actions available in crowd pressure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrowdActionType {
    /// Hymn/Sermon — broad crowd calming (Miriam).
    BroadCalm,
    /// Fast Talk — target a specific ringleader's nerve (Eli).
    TargetedNerve,
    /// Brace/Physical line — hold the barrier between crowd and target (Rosa).
    PhysicalHold,
    /// Warning shots / Suppressing fire — shock the crowd briefly (Galen).
    ShockAction,
    /// Medical authority — defuse fear-based panic with evidence (Ada).
    MedicalAuthority,
    /// Redirect — give the crowd a different target for anger (Eli).
    Redirect,
    /// Rebuke — direct nerve attack through conviction (Miriam).
    Rebuke,
}

/// Result of a crowd action.
#[derive(Debug, Clone)]
pub struct CrowdActionResult {
    pub actor: String,
    pub description: String,
    pub nerve_change: i32,
    pub momentum_change: i32,
    pub ringleader_broken: Option<String>,
    pub surge_delayed: bool,
}

impl CrowdState {
    /// Create a new crowd pressure encounter.
    pub fn new(collective_nerve: i32, surge_countdown: u32, ringleaders: Vec<Ringleader>) -> Self {
        Self {
            collective_nerve,
            max_nerve: collective_nerve.max(1),
            momentum: 0,
            phase: CrowdPhase::Tense,
            ringleaders,
            target_safe: true,
            turn: 0,
            surge_countdown,
        }
    }

    /// Execute a party member's crowd action.
    pub fn execute_action(&mut self, action: &CrowdAction) -> CrowdActionResult {
        self.turn += 1;
        let mut result = CrowdActionResult {
            actor: action.actor.clone(),
            description: String::new(),
            nerve_change: 0,
            momentum_change: 0,
            ringleader_broken: None,
            surge_delayed: false,
        };

        match action.action_type {
            CrowdActionType::BroadCalm => {
                // Miriam's Hymn/Sermon — affects the whole crowd
                let calm = BROAD_CALM_RESTORE;
                self.collective_nerve = (self.collective_nerve + calm).min(self.max_nerve);
                self.momentum += 2;
                result.nerve_change = calm;
                result.momentum_change = 2;
                result.description = format!("{} calms the crowd — collective nerve restored", action.actor);
            }

            CrowdActionType::TargetedNerve => {
                // Eli's Fast Talk — breaks a specific ringleader
                if let Some(target_id) = &action.target {
                    if let Some(rl) = self.ringleaders.iter_mut().find(|r| r.id == *target_id) {
                        rl.nerve -= 10;
                        if rl.nerve <= 0 && !rl.broken {
                            rl.broken = true;
                            // Breaking a ringleader calms the crowd proportional to their influence
                            let calm = rl.influence;
                            self.collective_nerve = (self.collective_nerve + calm).min(self.max_nerve);
                            self.momentum += 1;
                            result.ringleader_broken = Some(rl.name.clone());
                            result.nerve_change = calm;
                            result.description = format!(
                                "{} breaks {} — crowd calms as leader falters", action.actor, rl.name
                            );
                        } else {
                            result.description = format!(
                                "{} rattles {} — they hesitate", action.actor, rl.name
                            );
                        }
                    }
                }
            }

            CrowdActionType::PhysicalHold => {
                // Rosa's Brace — delays the surge, doesn't calm the crowd
                self.surge_countdown += 1;
                result.surge_delayed = true;
                result.description = format!(
                    "{} holds the line — the crowd cannot reach the target yet", action.actor
                );
            }

            CrowdActionType::ShockAction => {
                // Galen's warning shots — brief nerve boost but increases tension long-term
                let shock = 5;
                self.collective_nerve = (self.collective_nerve + shock).min(self.max_nerve);
                self.momentum -= 1; // shock doesn't build lasting calm
                result.nerve_change = shock;
                result.momentum_change = -1;
                result.description = format!(
                    "{} fires warning shots — the crowd flinches, but the fear deepens", action.actor
                );
            }

            CrowdActionType::MedicalAuthority => {
                // Ada's evidence — defuses the supernatural panic
                let calm = MEDICAL_AUTHORITY_RESTORE;
                self.collective_nerve = (self.collective_nerve + calm).min(self.max_nerve);
                self.momentum += 1;
                result.nerve_change = calm;
                result.momentum_change = 1;
                result.description = format!(
                    "{} presents medical evidence — some of the fear loosens", action.actor
                );
            }

            CrowdActionType::Redirect => {
                // Eli's misdirection — give the crowd a different target
                let redirect = REDIRECT_RESTORE;
                self.collective_nerve = (self.collective_nerve + redirect).min(self.max_nerve);
                // But momentum doesn't improve — this is manipulation, not calm
                result.nerve_change = redirect;
                result.description = format!(
                    "{} redirects the crowd's anger — effective and ugly", action.actor
                );
            }

            CrowdActionType::Rebuke => {
                // Miriam's conviction — direct nerve attack on specific instigator
                if let Some(target_id) = &action.target {
                    if let Some(rl) = self.ringleaders.iter_mut().find(|r| r.id == *target_id) {
                        rl.nerve -= 15; // stronger than Fast Talk
                        if rl.nerve <= 0 && !rl.broken {
                            rl.broken = true;
                            let calm = rl.influence + 3;
                            self.collective_nerve = (self.collective_nerve + calm).min(self.max_nerve);
                            self.momentum += 2;
                            result.ringleader_broken = Some(rl.name.clone());
                            result.nerve_change = calm;
                            result.momentum_change = 2;
                            result.description = format!(
                                "{} speaks a truth that {} cannot answer — they break", action.actor, rl.name
                            );
                        }
                    }
                }
            }
        }

        debug!(
            turn = self.turn,
            nerve = self.collective_nerve,
            momentum = self.momentum,
            phase = ?self.phase,
            "crowd action executed"
        );

        result
    }

    /// Advance the crowd state after all party actions this turn.
    /// Returns the new phase.
    pub fn advance(&mut self) -> CrowdPhase {
        // Natural crowd degradation each turn
        self.collective_nerve -= 3;

        // Apply momentum
        self.collective_nerve = (self.collective_nerve + self.momentum).max(0).min(self.max_nerve);

        // Surge countdown
        if self.surge_countdown > 0 {
            self.surge_countdown -= 1;
        }

        // Belt-and-suspenders: guard against division by zero even though the
        // constructor clamps max_nerve to at least 1. If something upstream
        // mutates max_nerve to 0, this prevents a crash.
        if self.max_nerve == 0 {
            self.phase = CrowdPhase::Broken;
            self.target_safe = false;
            eprintln!(
                "[crowd] max_nerve is 0 — this should never happen. \
                 Treating crowd as broken to avoid division by zero."
            );
            return self.phase;
        }

        // Update phase
        let nerve_pct = (self.collective_nerve as f32 / self.max_nerve as f32 * 100.0).round() as i32;

        self.phase = if self.collective_nerve <= 0 || self.surge_countdown == 0 {
            CrowdPhase::Broken
        } else if nerve_pct >= 80 {
            CrowdPhase::Dispersed
        } else if nerve_pct >= 50 || self.momentum > 2 {
            CrowdPhase::Calming
        } else if nerve_pct < 30 {
            CrowdPhase::Surging
        } else {
            CrowdPhase::Tense
        };

        if self.phase == CrowdPhase::Broken {
            self.target_safe = false;
            info!("crowd broke — target at risk");
        } else if self.phase == CrowdPhase::Dispersed {
            info!("crowd dispersed — containment successful");
        }

        self.phase
    }

    /// Check if the encounter is resolved.
    pub fn is_resolved(&self) -> bool {
        self.phase == CrowdPhase::Dispersed || self.phase == CrowdPhase::Broken
    }

    /// Was containment successful?
    pub fn contained(&self) -> bool {
        self.phase == CrowdPhase::Dispersed
    }

    /// Count active (non-broken) ringleaders.
    pub fn active_ringleaders(&self) -> usize {
        self.ringleaders.iter().filter(|r| !r.broken).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_crowd() -> CrowdState {
        CrowdState::new(50, 5, vec![
            Ringleader { id: "loud_man".to_string(), name: "Loud Man".to_string(), nerve: 12, influence: 8, broken: false },
            Ringleader { id: "angry_woman".to_string(), name: "Angry Woman".to_string(), nerve: 15, influence: 10, broken: false },
        ])
    }

    #[test]
    fn broad_calm_restores_nerve() {
        let mut crowd = test_crowd();
        crowd.collective_nerve = 30; // degraded

        let result = crowd.execute_action(&CrowdAction {
            actor: "miriam".to_string(),
            action_type: CrowdActionType::BroadCalm,
            target: None,
        });

        assert!(result.nerve_change > 0);
        assert!(crowd.collective_nerve > 30);
    }

    #[test]
    fn targeted_nerve_breaks_ringleader() {
        let mut crowd = test_crowd();

        // Weaken the loud man
        crowd.execute_action(&CrowdAction {
            actor: "eli".to_string(),
            action_type: CrowdActionType::TargetedNerve,
            target: Some("loud_man".to_string()),
        });

        // Should be broken (nerve was 12, hit for 10)
        let rl = crowd.ringleaders.iter().find(|r| r.id == "loud_man").expect("test ringleader must exist in crowd setup");
        assert!(rl.nerve <= 2);

        // Hit again to break
        let result = crowd.execute_action(&CrowdAction {
            actor: "eli".to_string(),
            action_type: CrowdActionType::TargetedNerve,
            target: Some("loud_man".to_string()),
        });

        assert!(result.ringleader_broken.is_some());
        assert_eq!(crowd.active_ringleaders(), 1);
    }

    #[test]
    fn physical_hold_delays_surge() {
        let mut crowd = test_crowd();
        let initial_countdown = crowd.surge_countdown;

        let result = crowd.execute_action(&CrowdAction {
            actor: "rosa".to_string(),
            action_type: CrowdActionType::PhysicalHold,
            target: None,
        });

        assert!(result.surge_delayed);
        assert_eq!(crowd.surge_countdown, initial_countdown + 1);
    }

    #[test]
    fn shock_calms_briefly_but_worsens_momentum() {
        let mut crowd = test_crowd();
        crowd.collective_nerve = 25;

        let result = crowd.execute_action(&CrowdAction {
            actor: "galen".to_string(),
            action_type: CrowdActionType::ShockAction,
            target: None,
        });

        assert!(result.nerve_change > 0, "shock should restore some nerve");
        assert!(result.momentum_change < 0, "shock should worsen momentum");
    }

    #[test]
    fn crowd_disperses_when_nerve_high() {
        let mut crowd = test_crowd();
        crowd.collective_nerve = crowd.max_nerve; // full nerve
        crowd.momentum = 5; // strong positive momentum

        let phase = crowd.advance();
        assert_eq!(phase, CrowdPhase::Dispersed);
        assert!(crowd.contained());
    }

    #[test]
    fn crowd_breaks_when_nerve_zero() {
        let mut crowd = test_crowd();
        crowd.collective_nerve = 0;

        let phase = crowd.advance();
        assert_eq!(phase, CrowdPhase::Broken);
        assert!(!crowd.target_safe);
        assert!(!crowd.contained());
    }

    #[test]
    fn crowd_breaks_when_surge_reaches_zero() {
        let mut crowd = test_crowd();
        crowd.surge_countdown = 1;
        crowd.collective_nerve = 20; // still has nerve, but no time

        let phase = crowd.advance();
        assert_eq!(phase, CrowdPhase::Broken);
    }

    #[test]
    fn different_actions_affect_crowd_differently() {
        // This is the core proof: multiple party members alter the same
        // pressure bar through different vectors.
        let mut crowd_a = test_crowd();
        let mut crowd_b = test_crowd();
        let mut crowd_c = test_crowd();

        crowd_a.collective_nerve = 25;
        crowd_b.collective_nerve = 25;
        crowd_c.collective_nerve = 25;

        // Miriam: broad calm
        crowd_a.execute_action(&CrowdAction {
            actor: "miriam".to_string(),
            action_type: CrowdActionType::BroadCalm,
            target: None,
        });

        // Galen: shock
        crowd_b.execute_action(&CrowdAction {
            actor: "galen".to_string(),
            action_type: CrowdActionType::ShockAction,
            target: None,
        });

        // Ada: medical authority
        crowd_c.execute_action(&CrowdAction {
            actor: "ada".to_string(),
            action_type: CrowdActionType::MedicalAuthority,
            target: None,
        });

        // All three affect nerve but through different mechanisms
        // Miriam: high nerve restore + positive momentum
        // Galen: moderate nerve restore + negative momentum
        // Ada: moderate nerve restore + slight positive momentum
        assert!(crowd_a.momentum > crowd_b.momentum,
            "Miriam should build better momentum than Galen's shock");
        assert!(crowd_b.momentum < 0,
            "Galen's shock should produce negative momentum");
    }
}
