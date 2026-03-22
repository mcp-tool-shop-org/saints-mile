//! Party argument system — tracks how party members react to charged decisions.
//!
//! Not giant affinity sludge. Explicit memory of who objected, who approved,
//! who complied without approval, and who files it away for later.

use serde::{Deserialize, Serialize};
use crate::types::*;

/// A recorded party argument — one charged decision and its reactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentRecord {
    /// What the argument was about.
    pub id: String,
    /// Which chapter/scene it happened in.
    pub chapter: String,
    /// What the player chose.
    pub player_stance: String,
    /// How each party member reacted.
    pub reactions: Vec<PartyReaction>,
}

/// How a party member reacted to a charged decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyReaction {
    pub character: CharacterId,
    pub response: ReactionType,
    /// Optional one-line summary of their position.
    pub position: String,
}

/// The types of reactions a party member can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionType {
    /// Agreed with the player's choice.
    Approved,
    /// Disagreed but went along.
    Complied,
    /// Openly objected.
    Objected,
    /// Said nothing — filed it away for later.
    Silent,
    /// Suggested this course of action.
    Advocated,
}

/// Build an argument record for the Ch4 water-claim decision.
pub fn water_claim_argument(stance: &str) -> ArgumentRecord {
    let reactions = match stance {
        "force" => vec![
            PartyReaction {
                character: CharacterId::new("rosa"),
                response: ReactionType::Advocated,
                position: "Her family is dying slowly. Cut the diversion.".to_string(),
            },
            PartyReaction {
                character: CharacterId::new("ada"),
                response: ReactionType::Objected,
                position: "Violence brings marshals. The legal position gets worse.".to_string(),
            },
            PartyReaction {
                character: CharacterId::new("eli"),
                response: ReactionType::Complied,
                position: "Direct, but it closes doors I'd rather leave open.".to_string(),
            },
            PartyReaction {
                character: CharacterId::new("miriam"),
                response: ReactionType::Silent,
                position: String::new(), // Miriam isn't here yet
            },
        ],
        "con" => vec![
            PartyReaction {
                character: CharacterId::new("rosa"),
                response: ReactionType::Objected,
                position: "My people don't need tricks. They need someone to stand.".to_string(),
            },
            PartyReaction {
                character: CharacterId::new("ada"),
                response: ReactionType::Complied,
                position: "Cleaner than force. But if the con fails, an innocent family pays.".to_string(),
            },
            PartyReaction {
                character: CharacterId::new("eli"),
                response: ReactionType::Advocated,
                position: "Faster, quieter, and nobody gets shot. Probably.".to_string(),
            },
        ],
        "negotiate" => vec![
            PartyReaction {
                character: CharacterId::new("rosa"),
                response: ReactionType::Complied,
                position: "Talk hasn't worked for years. But I'll let you try.".to_string(),
            },
            PartyReaction {
                character: CharacterId::new("ada"),
                response: ReactionType::Advocated,
                position: "Use the medical evidence. If the water is poisoning both sides, there's common ground.".to_string(),
            },
            PartyReaction {
                character: CharacterId::new("eli"),
                response: ReactionType::Complied,
                position: "Slow, but principled. I can work with principled.".to_string(),
            },
        ],
        _ => vec![],
    };

    ArgumentRecord {
        id: "water_claim".to_string(),
        chapter: "ch4".to_string(),
        player_stance: stance.to_string(),
        reactions,
    }
}
