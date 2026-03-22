//! Pressure encounter types — escort, crowd, reckoning, transmission.

use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::scene::types::{Condition, StateEffect};

/// A pressure encounter — nonstandard combat-adjacent gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureEncounter {
    pub id: String,
    pub pressure_type: PressureType,
    pub pressure_bars: Vec<PressureBar>,
    pub party_actions: Vec<PressurePartyAction>,
    pub success_threshold: PressureCondition,
    pub failure_threshold: PressureCondition,
    pub outcome_effects: Vec<StateEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PressureType {
    Escort { cargo: Vec<CargoItem> },
    Crowd { collective_nerve: i32, ringleaders: Vec<String> },
    PublicReckoning {
        room_credibility: i32,
        crowd_nerve: i32,
        witness_integrity: i32,
        evidence_continuity: i32,
        procedural_control: i32,
    },
    WitnessProtection { witness: WitnessId, integrity_drain: i32 },
    TransmissionRace { channels: Vec<Channel>, time_remaining: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureBar {
    pub id: String,
    pub label: String,
    pub current: i32,
    pub max: i32,
    pub fail_at: i32,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PressureCondition {
    BarReached { bar_id: String, threshold: i32 },
    AllBarsAboveFail,
    TimeExpired,
    FlagSet(FlagId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressurePartyAction {
    pub character: CharacterId,
    pub actions: Vec<PressureAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureAction {
    pub id: String,
    pub label: String,
    pub description: String,
    pub target_bar: String,
    pub delta: i32,
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoItem {
    pub id: String,
    pub name: String,
    pub integrity: i32,
    pub max_integrity: i32,
    pub loss_effect: Vec<StateEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub controlled_by: ChannelOwner,
    pub relay_points: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelOwner {
    Party,
    Enemy,
    Neutral,
}
