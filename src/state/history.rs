//! Historical ossification — how truth hardens into the wrong shape.
//!
//! Five layers of memory in tension. The return arc exists because
//! the wrong version is becoming permanent.

use serde::{Deserialize, Serialize};

/// The five hardened layers of memory after fifteen years.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalState {
    /// What towns say happened. Newspapers, gossip, reputation.
    pub public_version: MemoryLayer,
    /// What records say happened. Archives, filings, court transcripts.
    pub institutional_version: MemoryLayer,
    /// What workers, camps, and routes remember.
    pub road_version: MemoryLayer,
    /// What only the people there know.
    pub private_version: MemoryLayer,
    /// What Saint's Mile itself seems to keep. The bell, the ground.
    pub place_version: MemoryLayer,
}

/// A single layer of historical memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLayer {
    pub id: String,
    /// What this layer says about Galen.
    pub galen_narrative: String,
    /// What this layer says about the conspiracy.
    pub conspiracy_narrative: String,
    /// How entrenched this version is (0-100).
    pub entrenchment: i32,
    /// Whether this version is being actively maintained.
    pub actively_maintained: bool,
}

impl HistoricalState {
    /// Build the default fifteen-years-later state.
    /// The public and institutional versions are winning.
    pub fn fifteen_years_later() -> Self {
        Self {
            public_version: MemoryLayer {
                id: "public".to_string(),
                galen_narrative: "A frontier disruption figure. Possibly criminal, \
                                  possibly wronged. The poster was never fully rescinded.".to_string(),
                conspiracy_narrative: "Institutional irregularities were addressed. \
                                       Some officials were removed. The system corrected itself.".to_string(),
                entrenchment: 75,
                actively_maintained: true,
            },
            institutional_version: MemoryLayer {
                id: "institutional".to_string(),
                galen_narrative: "A wanted fugitive whose charges were partially reduced \
                                  but never fully cleared. His name exists in legal limbo.".to_string(),
                conspiracy_narrative: "The Briar Line's operations were restructured. \
                                       Some contracts were voided. The land claims stand.".to_string(),
                entrenchment: 80,
                actively_maintained: true,
            },
            road_version: MemoryLayer {
                id: "road".to_string(),
                galen_narrative: "The man who held Breakwater. Dangerous to stand near, \
                                  but not empty. Road workers remember.".to_string(),
                conspiracy_narrative: "The system was wrong. Not everyone agrees on how \
                                       wrong, but the people who were there know.".to_string(),
                entrenchment: 40,
                actively_maintained: false,
            },
            private_version: MemoryLayer {
                id: "private".to_string(),
                galen_narrative: "The man who chose correctly and paid with his hand. \
                                  The party knows what the record doesn't.".to_string(),
                conspiracy_narrative: "The machine inherited a structure of violence. \
                                       The mission fire was deliberate. The re-grant was fraud.".to_string(),
                entrenchment: 30,
                actively_maintained: false,
            },
            place_version: MemoryLayer {
                id: "place".to_string(),
                galen_narrative: "The bell rang when he was there. That is all the \
                                  ground will say.".to_string(),
                conspiracy_narrative: "The mission remembers. Whether that memory is \
                                       geological, psychological, or something else, \
                                       the game does not answer.".to_string(),
                entrenchment: 100, // place memory doesn't fade
                actively_maintained: false,
            },
        }
    }

    /// Check if the wrong version is winning.
    pub fn wrong_version_winning(&self) -> bool {
        self.institutional_version.entrenchment > self.private_version.entrenchment &&
        self.public_version.actively_maintained
    }

    /// The gap between truth and history.
    pub fn truth_gap(&self) -> i32 {
        self.institutional_version.entrenchment - self.private_version.entrenchment
    }
}
