//! Ending resolution — what Saint's Mile means after the credits.
//!
//! The final choice shapes legacy, not just survival.
//! Four axes: justice, exposure, peace, burden.

use serde::{Deserialize, Serialize};
use crate::types::*;

/// The four ending axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndingAxis {
    /// Kill Voss. The man dies. The system survives. Personal closure, institutional escape.
    Justice,
    /// Expose Voss fully. Force the territory to carry the uglier version. No comfortable history.
    Exposure,
    /// Spare Voss under conditions. Preserve peace but leave contamination in the record.
    Peace,
    /// Bury some truth for stability. The world gets a livable version. Galen carries what was real alone.
    Burden,
}

/// What the ending decides about the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndingState {
    pub axis: EndingAxis,

    /// What enters the public record.
    pub public_record: String,
    /// What becomes rumor.
    pub rumor: String,
    /// What Galen carries alone.
    pub private_burden: String,
    /// What happens to Voss.
    pub voss_fate: String,
    /// What version of Saint's Mile hardens after the credits.
    pub legacy: String,
}

/// Resolve the ending based on the player's choice and run state.
pub fn resolve_ending(axis: EndingAxis, run_state: &EndingRunState) -> EndingState {
    match axis {
        EndingAxis::Justice => EndingState {
            axis,
            public_record: "Voss was killed by the man he framed. The territory \
                           records it as frontier violence, not reckoning.".to_string(),
            rumor: "The road says Rook settled it. Whether that's justice or \
                   vengeance depends who's telling it.".to_string(),
            private_burden: "Galen carries the knowledge that killing Voss solved \
                            the man but not the machine.".to_string(),
            voss_fate: "Dead. His version of events dies with him — but the \
                       institution that formed him survives.".to_string(),
            legacy: "Saint's Mile becomes a story about one man's reckoning. \
                    The deeper truth — the mission, the re-grant, the forty-year \
                    fraud — gets narrower, not wider.".to_string(),
        },

        EndingAxis::Exposure => EndingState {
            axis,
            public_record: format!(
                "The full conspiracy is exposed. Mission fire, re-grant fraud, \
                 payroll manipulation, medical diversion. {}'s testimony \
                 anchors the public version.",
                if run_state.eli_loyalty_active { "Eli's" } else { "Documentary" }
            ),
            rumor: "The territory carries the uglier truth. Some people are \
                   grateful. Some wish they didn't know.".to_string(),
            private_burden: "Galen carries nothing alone. The truth is public now. \
                            That is its own kind of weight — shared, examined, \
                            and argued over by people who weren't there.".to_string(),
            voss_fate: "Exposed. Stripped of authority. Living with the territory's \
                       knowledge of what he did. Not killed — forced to exist \
                       inside the truth.".to_string(),
            legacy: "Saint's Mile becomes a scar the territory has to look at. \
                    Not comfortable. Not forgotten. The plaque gets rewritten. \
                    Nobody likes the new version better.".to_string(),
        },

        EndingAxis::Peace => EndingState {
            axis,
            public_record: "A negotiated settlement. Voss retires. Some records \
                           are sealed. The worst of the fraud is acknowledged \
                           in private.".to_string(),
            rumor: "The road knows more happened than the settlement says. \
                   But the fighting stopped. That counts for something.".to_string(),
            private_burden: "Galen carries the compromise. He knows the full truth \
                            and chose to let the world have a livable version. \
                            That choice has a taste.".to_string(),
            voss_fate: "Retired. Diminished. Not destroyed. The machine he built \
                       is damaged but his name survives in a smaller form.".to_string(),
            legacy: "Saint's Mile becomes a frontier incident that was 'resolved.' \
                    The deeper truth is known by fewer people. The peace holds. \
                    The silence costs.".to_string(),
        },

        EndingAxis::Burden => EndingState {
            axis,
            public_record: "The official version stands. Some details are corrected. \
                           The deeper structure is never fully exposed.".to_string(),
            rumor: "The road remembers. The plaque stays. The people who were there \
                   carry what the record doesn't.".to_string(),
            private_burden: "Galen carries the truth alone. Or with whoever stayed \
                            close enough to hear it. The world gets stability. \
                            He gets the weight.".to_string(),
            voss_fate: "Untouched. The system he served continues. His name on the \
                       plaque committee stands. History remembers him as the man \
                       who kept order.".to_string(),
            legacy: "Saint's Mile becomes the version polite people tell. The truth \
                    lives in private memory, road memory, and whatever the bell \
                    still says to anyone who listens.".to_string(),
        },
    }
}

/// Run state that the ending reads.
#[derive(Debug, Clone)]
pub struct EndingRunState {
    pub eli_loyalty_active: bool,
    pub hand_wounded: bool,
    pub relay_branch: String,
    pub tom_alive: bool,
    pub nella_alive: bool,
    pub party_reassembled: bool,
    pub lucien_present: bool,
    pub public_truth_level: i32,
}

impl EndingRunState {
    /// Build from game state flags.
    pub fn from_flags(flags: &std::collections::HashMap<String, FlagValue>) -> Self {
        let get_bool = |key: &str| -> bool {
            flags.get(key).map_or(false, |v| matches!(v, FlagValue::Bool(true)))
        };
        let get_text = |key: &str| -> String {
            flags.get(key).and_then(|v| match v {
                FlagValue::Text(s) => Some(s.clone()),
                _ => None,
            }).unwrap_or_default()
        };

        Self {
            eli_loyalty_active: get_bool("loyalty_line_unlocked"),
            hand_wounded: get_bool("hand_wounded"),
            relay_branch: get_text("relay_branch"),
            tom_alive: !get_bool("tom_died"),
            nella_alive: !get_bool("nella_died"),
            party_reassembled: get_bool("party_reassembled"),
            lucien_present: get_bool("lucien_captured"),
            public_truth_level: if get_bool("public_truth_established") { 70 } else { 30 },
        }
    }
}
