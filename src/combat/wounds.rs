//! Wound system — persistent injuries that carry between encounters.
//!
//! Wounds are not just HP damage. They are stat penalties that persist
//! until treated by the sawbones. The wound system makes Ada load-bearing.

use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::combat::types::Wound;

/// Pre-defined wound types.
pub fn gunshot_wound() -> Wound {
    Wound {
        id: InjuryId::new("gunshot"),
        name: "Gunshot Wound".to_string(),
        description: "Bullet lodged or grazed. Accuracy and speed reduced.".to_string(),
        penalties: vec![
            super::types::StatPenalty { stat: "accuracy".to_string(), amount: -10 },
            super::types::StatPenalty { stat: "speed".to_string(), amount: -2 },
        ],
        treatable: true,
    }
}

pub fn blunt_trauma() -> Wound {
    Wound {
        id: InjuryId::new("blunt_trauma"),
        name: "Blunt Trauma".to_string(),
        description: "Took a hard hit. Nerve and speed reduced.".to_string(),
        penalties: vec![
            super::types::StatPenalty { stat: "nerve".to_string(), amount: -5 },
            super::types::StatPenalty { stat: "speed".to_string(), amount: -1 },
        ],
        treatable: true,
    }
}

pub fn exhaustion() -> Wound {
    Wound {
        id: InjuryId::new("exhaustion"),
        name: "Exhaustion".to_string(),
        description: "Running on fumes. All stats slightly reduced.".to_string(),
        penalties: vec![
            super::types::StatPenalty { stat: "accuracy".to_string(), amount: -5 },
            super::types::StatPenalty { stat: "speed".to_string(), amount: -1 },
            super::types::StatPenalty { stat: "nerve".to_string(), amount: -3 },
        ],
        treatable: true,
    }
}

pub fn nerve_shock() -> Wound {
    Wound {
        id: InjuryId::new("nerve_shock"),
        name: "Nerve Shock".to_string(),
        description: "Seen too much. Nerve max reduced until treated.".to_string(),
        penalties: vec![
            super::types::StatPenalty { stat: "nerve".to_string(), amount: -8 },
        ],
        treatable: true,
    }
}

/// Triage result — what Ada's treatment achieves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    /// Wounds removed.
    pub healed: Vec<InjuryId>,
    /// HP restored.
    pub hp_restored: i32,
    /// Nerve restored.
    pub nerve_restored: i32,
    /// Time cost (affects chapter pacing — pursuing clues vs treating patients).
    pub time_cost: i32,
}

/// Apply triage to a set of wounds. Returns what was healed and what it cost.
///
/// The care-versus-pursuit tension: treating wounds takes time.
/// Time spent healing is time not spent chasing the sheriff's trail.
pub fn triage(wounds: &[Wound], thorough: bool) -> TriageResult {
    let mut healed = Vec::new();
    let mut hp_restored = 0;
    let mut nerve_restored = 0;
    let mut time_cost = 0;

    for wound in wounds {
        if wound.treatable {
            healed.push(wound.id.clone());
            // Calculate what treating this wound restores
            for penalty in &wound.penalties {
                match penalty.stat.as_str() {
                    "nerve" => nerve_restored += penalty.amount.unsigned_abs() as i32,
                    _ => hp_restored += 5, // base HP restoration per wound treated
                }
            }
            time_cost += if thorough { 2 } else { 1 };
        }
    }

    if thorough {
        hp_restored = (hp_restored as f32 * 1.5) as i32;
        nerve_restored = (nerve_restored as f32 * 1.5) as i32;
    }

    TriageResult { healed, hp_restored, nerve_restored, time_cost }
}
