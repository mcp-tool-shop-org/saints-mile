//! Combat data types — encounters, standoffs, party members, skills, duo techs.

use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::scene::types::{Condition, StateEffect};

/// A complete encounter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub id: EncounterId,
    pub phases: Vec<CombatPhase>,
    pub standoff: Option<Standoff>,
    /// Always 4, even if only 2 are filled.
    pub party_slots: u8,
    pub terrain: Terrain,
    pub objectives: Vec<Objective>,
    pub outcome_effects: Vec<OutcomeEffect>,
}

/// A single phase of a multi-phase encounter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatPhase {
    pub id: String,
    pub description: String,
    pub enemies: Vec<EnemyTemplate>,
    #[serde(default)]
    pub npc_allies: Vec<NpcCombatant>,
    #[serde(default)]
    pub entry_conditions: Vec<Condition>,
    #[serde(default)]
    pub phase_effects: Vec<StateEffect>,
}

/// The standoff pre-phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Standoff {
    pub postures: Vec<StandoffPosture>,
    pub allow_focus: bool,
    pub eli_influence: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandoffPosture {
    EarlyDraw,
    SteadyHand,
    Bait,
}

/// Result of the standoff phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StandoffResult {
    pub initiative_mods: Vec<(String, i32)>,
    pub nerve_damage: Vec<(String, i32)>,
    pub broken_enemies: Vec<usize>,
    pub first_shot_accuracy: i32,
}

/// A combatant's live state during combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatantState {
    pub character: CharacterId,
    pub hp: i32,
    pub max_hp: i32,
    pub nerve: i32,
    pub max_nerve: i32,
    pub ammo: i32,
    pub max_ammo: i32,
    pub wounds: Vec<Wound>,
    pub position: PositionState,
    pub available_skills: Vec<SkillId>,
    pub available_duo_techs: Vec<DuoTechId>,
}

/// A skill definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub line: SkillLine,
    pub unlock: UnlockCondition,
    pub age_variants: Vec<AgeVariant>,
    pub cost: SkillCost,
}

/// Age-specific variant of a skill — the menu carries biography.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeVariant {
    pub phase: AgePhase,
    pub accuracy: i32,
    pub damage: i32,
    pub speed_priority: i32,
    pub nerve_damage: i32,
    pub description_override: Option<String>,
}

/// Skill lines — each character has three.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLine {
    // Galen
    Deadeye, Trailcraft, Command,
    // Eli
    Hustle, Deceit,
    /// Grayed out in the menu before Chapter 10.
    Loyalty,
    // Ada
    Triage, Tonics, Diagnosis,
    // Rosa
    Lariat, Guard, Break,
    // Miriam
    Hymn, Witness, Intercession,
    // Lucien
    Charges, Smoke, Collapse,
}

/// How a skill is unlocked — narrative, not numeric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnlockCondition {
    StartOfPhase(AgePhase),
    StoryEvent(FlagId),
    TurningPoint(FlagId),
    Bond { character_a: CharacterId, character_b: CharacterId, threshold: i32 },
    Ordeal(FlagId),
}

/// Duo technique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuoTech {
    pub id: DuoTechId,
    pub name: String,
    pub description: String,
    pub members: (CharacterId, CharacterId),
    pub unlock: UnlockCondition,
    pub cost: DuoTechCost,
    pub effect: DuoTechEffect,
}

/// A wound that persists between encounters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wound {
    pub id: InjuryId,
    pub name: String,
    pub description: String,
    pub penalties: Vec<StatPenalty>,
    pub treatable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionState {
    Open,
    InCover,
    Elevated,
    FrontLine,
    BackLine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terrain {
    pub name: String,
    pub cover: Vec<CoverElement>,
    #[serde(default)]
    pub hazards: Vec<EnvironmentalHazard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverElement {
    pub name: String,
    pub durability: i32,
    pub destructible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentalHazard {
    FuseCharge { turns_to_detonate: u8, blast_radius: u8 },
    CollapseRisk { trigger_damage: i32 },
    CrowdSurge { collective_nerve: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyTemplate {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub nerve: i32,
    pub damage: i32,
    pub accuracy: i32,
    pub speed: i32,
    pub bluff: i32,
    pub nerve_threshold: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcCombatant {
    pub character: CharacterId,
    pub behavior: NpcBehavior,
    pub hp: i32,
    pub nerve: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcBehavior {
    Professional,
    Unreliable,
    Protective,
    Nervous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub id: String,
    pub label: String,
    pub objective_type: ObjectiveType,
    pub fail_consequence: Vec<StateEffect>,
    pub success_consequence: Vec<StateEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveType {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeEffect {
    pub condition: OutcomeCondition,
    pub effects: Vec<StateEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutcomeCondition {
    Victory,
    Defeat,
    ObjectiveComplete(String),
    ObjectiveFailed(String),
    SkillUsed { character: CharacterId, skill: SkillId },
    SkillNotUsed { character: CharacterId, skill: SkillId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCost {
    pub ammo: i32,
    pub nerve: i32,
    pub cooldown_turns: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuoTechCost {
    pub ammo: i32,
    pub nerve: i32,
    pub both_turns: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuoTechEffect {
    pub description: String,
    pub damage: i32,
    pub accuracy_bonus: i32,
    pub nerve_damage: i32,
    pub special: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatPenalty {
    pub stat: String,
    pub amount: i32,
}
