//! Scene data types — the schema for towns, campfires, investigations, and consequence scenes.

use serde::{Deserialize, Serialize};
use crate::types::*;

/// A complete scene: one playable conversation, exploration, or consequence moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: SceneId,
    pub location: LocationId,
    pub beat: BeatId,
    pub lines: Vec<SceneLine>,
    pub choices: Vec<Choice>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    pub state_effects: Vec<StateEffect>,
    pub pacing: PacingTag,
    #[serde(default)]
    pub memory_refs: Vec<MemoryRef>,
}

/// A single line of dialogue or narration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLine {
    pub speaker: SpeakerId,
    pub text: String,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    pub emotion: Option<EmotionTag>,
}

/// A player choice within a scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub label: String,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    pub effects: Vec<StateEffect>,
    pub next: SceneTransition,
}

/// Where a choice leads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneTransition {
    Scene(SceneId),
    Beat(BeatId),
    Combat(EncounterId),
    End,
}

/// A condition that gates scene/line/choice availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    Flag { id: FlagId, value: FlagValue },
    Reputation { axis: ReputationAxis, op: CompareOp, threshold: i32 },
    PartyMember { character: CharacterId, present: bool },
    Witness { id: WitnessId, alive: bool },
    HasMemoryObject(MemoryObjectId),
    RelayBranch(RelayBranch),
    PrologueChoice(PrologueChoice),
    HasEvidence(EvidenceId),
    HasSkill { character: CharacterId, skill: SkillId },
}

/// A state change produced by a scene, choice, or combat outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateEffect {
    SetFlag { id: FlagId, value: FlagValue },
    AdjustReputation { axis: ReputationAxis, delta: i32 },
    AddPartyMember(CharacterId),
    RemovePartyMember(CharacterId),
    UnlockSkill { character: CharacterId, skill: SkillId },
    AddEvidence(EvidenceId),
    SetWitnessState { id: WitnessId, alive: bool, integrity: i32 },
    AddMemoryObject(MemoryObjectId),
    TransformMemoryObject { id: MemoryObjectId, new_state: String },
    AdjustResource { resource: ResourceKind, delta: i32 },
    SetRelationship { a: CharacterId, b: CharacterId, value: i32 },
    ApplyInjury { character: CharacterId, injury: InjuryId },
}

/// A reference to a memory object that may echo in future scenes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRef {
    pub object: MemoryObjectId,
    pub callback_type: MemoryCallbackType,
    pub target_chapter: Option<ChapterId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryCallbackType {
    Echo,
    Carry,
    Transform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacingTag {
    Exploration,
    Pressure,
    Intimate,
    Crisis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionTag {
    Neutral,
    Warm,
    Tense,
    Bitter,
    Dry,
    Grief,
    Quiet,
}
