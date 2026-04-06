//! Wound system — persistent injuries that carry between encounters.
//!
//! Wounds are not just HP damage. They are stat penalties that persist
//! until treated by the sawbones. The wound system makes Ada load-bearing.

use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::combat::types::{Wound, WoundSeverity};

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
        severity: WoundSeverity::Major,
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
        severity: WoundSeverity::Minor,
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
        severity: WoundSeverity::Minor,
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
        severity: WoundSeverity::Major,
    }
}

// ─── Wound Recovery ───────────────────────────────────────────────

/// Remove a specific wound by index if the sawbones (Ada) is in the party.
/// Returns true if the wound was successfully removed.
///
/// Ada is the only character who can treat wounds in the field.
/// Without her, wounds persist until the next rest with a sawbones available.
pub fn recover_wound(wounds: &mut Vec<Wound>, wound_idx: usize, ada_present: bool) -> bool {
    if !ada_present {
        return false;
    }
    if wound_idx >= wounds.len() {
        return false;
    }
    if !wounds[wound_idx].treatable {
        return false;
    }
    wounds.remove(wound_idx);
    true
}

/// Rest recovery — heals the oldest minor wound during camp/rest scenes.
/// Only minor wounds (exhaustion, bruises) heal with rest alone.
/// Major wounds (gunshots, nerve shock) require Ada's treatment.
pub fn rest_recovery(wounds: &mut Vec<Wound>) -> Option<InjuryId> {
    // Find the first (oldest) minor, treatable wound
    let idx = wounds.iter().position(|w| {
        w.treatable && w.severity == WoundSeverity::Minor
    });
    idx.map(|i| {
        let healed = wounds.remove(i);
        healed.id
    })
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
                    "nerve" => nerve_restored += penalty.amount.abs(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recover_wound_with_ada() {
        let mut wounds = vec![gunshot_wound(), exhaustion()];
        assert!(recover_wound(&mut wounds, 0, true));
        assert_eq!(wounds.len(), 1);
        assert_eq!(wounds[0].id.0, "exhaustion");
    }

    #[test]
    fn recover_wound_without_ada_fails() {
        let mut wounds = vec![gunshot_wound()];
        assert!(!recover_wound(&mut wounds, 0, false));
        assert_eq!(wounds.len(), 1);
    }

    #[test]
    fn recover_wound_out_of_bounds() {
        let mut wounds = vec![gunshot_wound()];
        assert!(!recover_wound(&mut wounds, 5, true));
        assert_eq!(wounds.len(), 1);
    }

    #[test]
    fn recover_wound_untreatable() {
        let mut wounds = vec![Wound {
            id: InjuryId::new("permanent_scar"),
            name: "Permanent Scar".to_string(),
            description: "This never heals.".to_string(),
            penalties: vec![],
            treatable: false,
            severity: WoundSeverity::Major,
        }];
        assert!(!recover_wound(&mut wounds, 0, true));
        assert_eq!(wounds.len(), 1);
    }

    #[test]
    fn rest_recovery_heals_oldest_minor() {
        let mut wounds = vec![gunshot_wound(), exhaustion(), blunt_trauma()];
        // gunshot is Major, exhaustion is Minor (oldest minor), blunt is Minor
        let healed = rest_recovery(&mut wounds);
        assert_eq!(healed, Some(InjuryId::new("exhaustion")));
        assert_eq!(wounds.len(), 2);
        // gunshot and blunt_trauma remain
        assert_eq!(wounds[0].id.0, "gunshot");
        assert_eq!(wounds[1].id.0, "blunt_trauma");
    }

    #[test]
    fn rest_recovery_skips_major_wounds() {
        let mut wounds = vec![gunshot_wound(), nerve_shock()];
        let healed = rest_recovery(&mut wounds);
        assert_eq!(healed, None);
        assert_eq!(wounds.len(), 2);
    }

    #[test]
    fn rest_recovery_empty_wounds() {
        let mut wounds: Vec<Wound> = vec![];
        let healed = rest_recovery(&mut wounds);
        assert_eq!(healed, None);
    }
}
