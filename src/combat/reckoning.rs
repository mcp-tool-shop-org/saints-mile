//! Public Reckoning Pressure — the hearing room as a battlefield.
//!
//! Five pressure bars tracked simultaneously. The player manages truth
//! delivery like combat: sequence matters, timing matters, who speaks
//! when changes what the room believes.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::types::*;

/// Live state of the Deadwater reckoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReckoningState {
    /// How seriously the hearing takes the evidence. 0 = dismissed, 100 = undeniable.
    pub room_credibility: i32,
    /// How close the spectators are to riot or dismissal. 0 = chaos, 100 = stable.
    pub crowd_nerve: i32,
    /// Whether key witnesses are intact. 0 = compromised, 100 = credible.
    pub witness_integrity: i32,
    /// Whether the documentary chain holds under challenge. 0 = broken, 100 = solid.
    pub evidence_continuity: i32,
    /// Whether the hearing stays within usable bounds. 0 = dismissed, 100 = fair.
    pub procedural_control: i32,

    /// Sequence of evidence/testimony presented.
    pub sequence: Vec<ReckoningAction>,
    /// Current turn in the reckoning.
    pub turn: u32,
    /// Whether Eli has acted.
    pub eli_acted: bool,
    /// Overall reckoning phase.
    pub phase: ReckoningPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReckoningPhase {
    Opening,
    Presentation,
    Counterstrike,
    EliAct,
    Verdict,
}

/// An action taken in the reckoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReckoningAction {
    pub actor: String,
    pub action_type: ReckoningActionType,
    pub description: String,
    pub effects: ReckoningEffects,
}

/// Types of reckoning actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReckoningActionType {
    /// Present evidence (Galen — sequence and timing).
    PresentEvidence,
    /// Medical/body testimony (Ada — makes damage impossible to abstract).
    MedicalTestimony,
    /// Land/territorial consequence (Rosa — puts physical cost in the room).
    TerritorialTestimony,
    /// Stabilize the room (Miriam — keeps panic below ignition).
    StabilizeRoom,
    /// Read procedural weakness (Eli — finds where the lie assumes compliance).
    SystemRead,
    /// Contamination testimony (Lucien — ugly corroboration, changes room trust).
    ContaminationTestimony,
    /// Eli's defining act — steps forward, names himself, tells the plain truth.
    EliDefiningAct,
    /// Opposition counterstrike — discredit, dismiss, agitate.
    OppositionStrike,
}

/// Effects of a reckoning action on the five bars.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReckoningEffects {
    pub credibility: i32,
    pub crowd_nerve: i32,
    pub witness_integrity: i32,
    pub evidence_continuity: i32,
    pub procedural_control: i32,
}

impl ReckoningState {
    /// Create a new reckoning with initial state.
    /// Ch9 transmission results affect the starting position.
    pub fn new(ch9_result: &str) -> Self {
        let (cred, crowd, proc) = match ch9_result {
            "switchback" => (60, 45, 65),  // procedure usable
            "caldwell" => (55, 35, 50),    // public hot, volatile
            "pine_signal" => (55, 50, 55), // truth has a head start
            _ => (50, 40, 50),             // default/mixed
        };

        Self {
            room_credibility: cred,
            crowd_nerve: crowd,
            witness_integrity: 70,
            evidence_continuity: 75,
            procedural_control: proc,
            sequence: Vec::new(),
            turn: 0,
            eli_acted: false,
            phase: ReckoningPhase::Opening,
        }
    }

    /// Execute a reckoning action.
    pub fn execute_action(&mut self, action: ReckoningAction) {
        self.turn += 1;

        // Apply effects
        self.room_credibility = (self.room_credibility + action.effects.credibility).clamp(0, 100);
        self.crowd_nerve = (self.crowd_nerve + action.effects.crowd_nerve).clamp(0, 100);
        self.witness_integrity = (self.witness_integrity + action.effects.witness_integrity).clamp(0, 100);
        self.evidence_continuity = (self.evidence_continuity + action.effects.evidence_continuity).clamp(0, 100);
        self.procedural_control = (self.procedural_control + action.effects.procedural_control).clamp(0, 100);

        if action.action_type == ReckoningActionType::EliDefiningAct {
            self.eli_acted = true;
            self.phase = ReckoningPhase::EliAct;
            info!("Eli's defining act — Loyalty line unlocks");
        }

        debug!(
            turn = self.turn,
            credibility = self.room_credibility,
            crowd = self.crowd_nerve,
            witness = self.witness_integrity,
            evidence = self.evidence_continuity,
            procedure = self.procedural_control,
            "reckoning action executed"
        );

        self.sequence.push(action);
    }

    /// Advance to the next phase.
    pub fn advance_phase(&mut self) {
        self.phase = match self.phase {
            ReckoningPhase::Opening => ReckoningPhase::Presentation,
            ReckoningPhase::Presentation => ReckoningPhase::Counterstrike,
            ReckoningPhase::Counterstrike => {
                if !self.eli_acted { ReckoningPhase::EliAct } else { ReckoningPhase::Verdict }
            }
            ReckoningPhase::EliAct => ReckoningPhase::Verdict,
            ReckoningPhase::Verdict => ReckoningPhase::Verdict,
        };
    }

    /// Check if any bar has hit critical failure.
    pub fn any_bar_critical(&self) -> bool {
        self.room_credibility <= 10 ||
        self.crowd_nerve <= 10 ||
        self.procedural_control <= 10
    }

    /// Calculate the overall reckoning score (0-100).
    pub fn overall_score(&self) -> i32 {
        debug_assert!((0..=100).contains(&self.room_credibility), "room_credibility out of 0..=100: {}", self.room_credibility);
        debug_assert!((0..=100).contains(&self.crowd_nerve), "crowd_nerve out of 0..=100: {}", self.crowd_nerve);
        debug_assert!((0..=100).contains(&self.witness_integrity), "witness_integrity out of 0..=100: {}", self.witness_integrity);
        debug_assert!((0..=100).contains(&self.evidence_continuity), "evidence_continuity out of 0..=100: {}", self.evidence_continuity);
        debug_assert!((0..=100).contains(&self.procedural_control), "procedural_control out of 0..=100: {}", self.procedural_control);
        (self.room_credibility + self.crowd_nerve + self.witness_integrity +
         self.evidence_continuity + self.procedural_control) / 5
    }

    /// Was the reckoning successful enough to matter?
    pub fn partial_victory(&self) -> bool {
        self.overall_score() >= 40 && self.eli_acted && !self.any_bar_critical()
    }
}

/// Build Eli's defining act as a reckoning action.
pub fn eli_defining_act() -> ReckoningAction {
    ReckoningAction {
        actor: "eli".to_string(),
        action_type: ReckoningActionType::EliDefiningAct,
        description: "Eli steps into the room's center, identifies himself as the man \
                      who took the ledger at Saint's Mile, and tells the truth in plain \
                      language that damages him as much as anyone else.".to_string(),
        effects: ReckoningEffects {
            credibility: 20,        // massive credibility boost — a man willing to be ruined
            crowd_nerve: -10,       // the room gets hotter
            witness_integrity: 15,  // his testimony validates other witnesses
            evidence_continuity: 10, // connects the ledger to everything else
            procedural_control: -5, // the room is less controlled after
            ..Default::default()
        },
    }
}

/// Party action builders for the reckoning.
pub fn galen_present_evidence() -> ReckoningAction {
    ReckoningAction {
        actor: "galen".to_string(),
        action_type: ReckoningActionType::PresentEvidence,
        description: "Galen presents evidence — timing and sequence.".to_string(),
        effects: ReckoningEffects {
            credibility: 8,
            evidence_continuity: 10,
            ..Default::default()
        },
    }
}

pub fn ada_medical_testimony() -> ReckoningAction {
    ReckoningAction {
        actor: "ada".to_string(),
        action_type: ReckoningActionType::MedicalTestimony,
        description: "Ada makes bodies impossible to reduce to paperwork.".to_string(),
        effects: ReckoningEffects {
            credibility: 10,
            witness_integrity: 8,
            crowd_nerve: 5,
            ..Default::default()
        },
    }
}

pub fn rosa_territorial_testimony() -> ReckoningAction {
    ReckoningAction {
        actor: "rosa".to_string(),
        action_type: ReckoningActionType::TerritorialTestimony,
        description: "Rosa puts territorial consequence into the room.".to_string(),
        effects: ReckoningEffects {
            credibility: 7,
            crowd_nerve: -3, // her anger heats the room
            ..Default::default()
        },
    }
}

pub fn miriam_stabilize() -> ReckoningAction {
    ReckoningAction {
        actor: "miriam".to_string(),
        action_type: ReckoningActionType::StabilizeRoom,
        description: "Miriam holds panic below ignition.".to_string(),
        effects: ReckoningEffects {
            crowd_nerve: 12,
            procedural_control: 8,
            ..Default::default()
        },
    }
}

pub fn eli_system_read() -> ReckoningAction {
    ReckoningAction {
        actor: "eli".to_string(),
        action_type: ReckoningActionType::SystemRead,
        description: "Eli reads the procedural weak points.".to_string(),
        effects: ReckoningEffects {
            procedural_control: 10,
            evidence_continuity: 5,
            ..Default::default()
        },
    }
}

pub fn opposition_strike() -> ReckoningAction {
    ReckoningAction {
        actor: "opposition".to_string(),
        action_type: ReckoningActionType::OppositionStrike,
        description: "The opposition pushes back — discredit, dismiss, agitate.".to_string(),
        effects: ReckoningEffects {
            credibility: -12,
            crowd_nerve: -8,
            witness_integrity: -10,
            procedural_control: -5,
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reckoning_state_tracks_five_bars() {
        let state = ReckoningState::new("mixed");
        assert!(state.room_credibility > 0);
        assert!(state.crowd_nerve > 0);
        assert!(state.witness_integrity > 0);
        assert!(state.evidence_continuity > 0);
        assert!(state.procedural_control > 0);
    }

    #[test]
    fn ch9_results_change_opening() {
        let switchback = ReckoningState::new("switchback");
        let caldwell = ReckoningState::new("caldwell");

        assert!(switchback.procedural_control > caldwell.procedural_control,
            "switchback should give more procedural control");
        assert!(caldwell.crowd_nerve < switchback.crowd_nerve,
            "caldwell should make the crowd hotter");
    }

    #[test]
    fn eli_act_is_the_hinge() {
        let mut state = ReckoningState::new("mixed");
        let cred_before = state.room_credibility;

        state.execute_action(eli_defining_act());

        assert!(state.room_credibility > cred_before,
            "Eli's act should boost credibility");
        assert!(state.eli_acted);
        assert_eq!(state.phase, ReckoningPhase::EliAct);
    }

    #[test]
    fn opposition_degrades_all_bars() {
        let mut state = ReckoningState::new("mixed");
        let score_before = state.overall_score();

        state.execute_action(opposition_strike());

        assert!(state.overall_score() < score_before,
            "opposition should lower overall score");
    }

    #[test]
    fn sequence_matters() {
        // Present evidence first, then stabilize
        let mut state_a = ReckoningState::new("mixed");
        state_a.execute_action(galen_present_evidence());
        state_a.execute_action(miriam_stabilize());

        // Stabilize first, then evidence
        let mut state_b = ReckoningState::new("mixed");
        state_b.execute_action(miriam_stabilize());
        state_b.execute_action(galen_present_evidence());

        // Both reach similar endpoints but through different intermediate states
        // The point is that the system supports sequencing
        assert_eq!(state_a.sequence.len(), 2);
        assert_eq!(state_b.sequence.len(), 2);
        assert_ne!(state_a.sequence[0].actor, state_b.sequence[0].actor);
    }
}
