//! Reassembly state — how each ally returns after fifteen years.
//!
//! Not everyone comes back the same way. Each return mode carries
//! a different truth, a different cost, and a different relationship
//! to the road ahead.

use serde::{Deserialize, Serialize};
use crate::types::*;

/// How an ally returns to the party.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReturnMode {
    /// Returns in person, ready to fight.
    Body,
    /// Returns through testimony or evidence, not physically present.
    Testimony,
    /// Refuses to return. Their absence is the statement.
    Refusal,
    /// Returns after delay or conditions.
    Conditional,
    /// Present only in memory — what they left behind still matters.
    MemoryOnly,
}

/// One ally's reassembly state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllyReturn {
    pub character: CharacterId,
    pub mode: ReturnMode,
    /// What truth this ally carries back.
    pub truth_carried: String,
    /// How time changed them.
    pub change: String,
    /// Unresolved tension they bring into the finale.
    pub tension: String,
}

/// Build the reassembly state for Chapter 14.
pub fn chapter_14_reassembly() -> Vec<AllyReturn> {
    vec![
        AllyReturn {
            character: CharacterId::new("eli"),
            mode: ReturnMode::Body,
            truth_carried: "System intelligence. He knows how false permanence \
                           gets maintained. The ledger is still with him.".to_string(),
            change: "Steadier. The Loyalty transformation is fifteen years deep now. \
                    His honesty costs him daily, not just once.".to_string(),
            tension: "He stayed nearest longest. That nearness has weight — he knows \
                     things about Galen's fifteen years that nobody else does.".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("ada"),
            mode: ReturnMode::Body,
            truth_carried: "Medical witness. Body and consequence. She knows exactly \
                           what age cost Galen physically.".to_string(),
            change: "Harsher. More exact. Fifteen years of frontier medicine made \
                    her clinical in a way that is both strength and armor.".to_string(),
            tension: "She sees the hand before she comments on anything else. \
                     That is Ada.".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("rosa"),
            mode: ReturnMode::Conditional,
            truth_carried: "Land truth. Embodied consequence. The cost of paper on \
                           land and blood — she has lived it for fifteen years.".to_string(),
            change: "Authority. She may never have stopped fighting, just changed \
                    what she fought for. The fence held. She held the fence.".to_string(),
            tension: "She is the person least interested in speeches and most \
                     interested in what still stands.".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("miriam"),
            mode: ReturnMode::Body,
            truth_carried: "Public moral frame. Memory in public language. She knows \
                           how communities metabolized the earlier truth.".to_string(),
            change: "She now holds spaces Galen cannot. Civic rooms, assembly \
                    halls, places where talking matters.".to_string(),
            tension: "Her faith is deeper, not answered. The bell question lives \
                     in her still.".to_string(),
        },
        AllyReturn {
            character: CharacterId::new("lucien"),
            mode: ReturnMode::Conditional,
            truth_carried: "The witness to deliberate destruction no clean person \
                           can replace. His expertise is ugly and necessary.".to_string(),
            change: "Less explosive, more exact, more haunted by professional \
                    memory. The crack from the mission never closed.".to_string(),
            tension: "His return still stings. He may be less dangerous and more \
                     useful, but the party's memory of who he was doesn't vanish.".to_string(),
        },
    ]
}

/// Check if the reassembly has enough people for the final approach.
pub fn can_approach_saints_mile(returns: &[AllyReturn]) -> bool {
    let body_count = returns.iter()
        .filter(|r| r.mode == ReturnMode::Body || r.mode == ReturnMode::Conditional)
        .count();
    body_count >= 3 // need at least 3 returning allies + Galen
}
