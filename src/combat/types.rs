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
    /// Whether the party can attempt to flee. Boss encounters set this to false.
    /// Defaults to true for serde deserialization (existing data without this field).
    #[serde(default = "default_escapable")]
    pub escapable: bool,
}

fn default_escapable() -> bool { true }

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
    /// Active status effects: (effect, remaining turns).
    #[serde(default)]
    pub active_effects: Vec<(StatusEffect, u8)>,
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

/// How severe a wound is — determines rest recovery eligibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WoundSeverity {
    /// Minor wounds heal with rest (exhaustion, bruises).
    Minor,
    /// Major wounds require the sawbones (gunshots, nerve shock).
    Major,
}

/// A wound that persists between encounters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wound {
    pub id: InjuryId,
    pub name: String,
    pub description: String,
    pub penalties: Vec<StatPenalty>,
    pub treatable: bool,
    /// Severity determines whether rest alone can heal it.
    /// Defaults to Major for backward compatibility with existing wounds.
    #[serde(default = "default_severity")]
    pub severity: WoundSeverity,
}

fn default_severity() -> WoundSeverity { WoundSeverity::Major }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionState {
    Open,
    InCover,
    /// Partial cover — 25% damage reduction instead of 50%.
    PartialCover,
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
    /// Character-specific speed. Defaults to 10 via serde for legacy encounters.
    #[serde(default = "default_npc_speed")]
    pub speed: i32,
    /// Character-specific accuracy. Defaults to 60 via serde for legacy encounters.
    #[serde(default = "default_npc_accuracy")]
    pub accuracy: i32,
    /// Character-specific damage. Defaults to 8 via serde for legacy encounters.
    #[serde(default = "default_npc_damage")]
    pub damage: i32,
}

fn default_npc_speed() -> i32 { 10 }
fn default_npc_accuracy() -> i32 { 60 }
fn default_npc_damage() -> i32 { 8 }

/// Character-specific NPC combat stats. Used by content authors and the
/// engine to look up real stats for named NPC allies.
#[derive(Debug, Clone)]
pub struct NpcCharacterStats {
    pub speed: i32,
    pub accuracy: i32,
    pub damage: i32,
}

/// Look up character-specific combat stats for an NPC ally.
/// Named characters get distinct stat lines; unknown characters get defaults.
/// Content authors can use this to populate NpcCombatant fields.
pub fn npc_stats_for(character_id: &str) -> NpcCharacterStats {
    match character_id {
        // Deputies — professional, reliable, moderate all-round
        "cal" => NpcCharacterStats { speed: 9, accuracy: 62, damage: 9 },
        "deputy_harris" => NpcCharacterStats { speed: 11, accuracy: 65, damage: 9 },
        // Renata — sharper but more fragile
        "renata" => NpcCharacterStats { speed: 10, accuracy: 68, damage: 7 },
        // Bale — convoy guard, slow but heavy-hitting
        "bale" => NpcCharacterStats { speed: 7, accuracy: 55, damage: 12 },
        // Generic / unknown NPC — baseline stats
        _ => NpcCharacterStats { speed: 10, accuracy: 60, damage: 8 },
    }
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

/// How a secondary objective determines success or failure.
/// Used by the engine to decide outcome evaluation logic instead of
/// fragile string-contains checks on objective IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveBehavior {
    /// Succeeds if all enemies broke (panicked) without any being killed.
    CivilianCasualties,
    /// Succeeds if a specific asset survives. (Future: asset_id field.)
    ProtectAsset,
    /// Default: secondary objectives succeed when the primary succeeds.
    General,
}

impl Objective {
    /// Resolve the behavior for this objective. When objectives are migrated to
    /// carry an explicit `ObjectiveBehavior` field, this method can be replaced
    /// with a direct field read. For now, we derive behavior from the id as a
    /// migration bridge — but using an enum makes the engine logic type-safe.
    pub fn behavior(&self) -> ObjectiveBehavior {
        if self.id.contains("casualties") || self.id.contains("civilian") {
            ObjectiveBehavior::CivilianCasualties
        } else {
            ObjectiveBehavior::General
        }
    }
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

/// Status effects that can be applied to combatants during combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusEffect {
    /// Damage per turn (bleed out).
    Bleeding,
    /// Skip turn entirely.
    Stunned,
    /// Damage boost (+25%).
    Inspired,
    /// Accuracy penalty (-20).
    Suppressed,
}

impl StatusEffect {
    /// Apply the effect's per-turn impact. Returns (hp_damage, nerve_damage, skip_turn).
    pub fn per_turn_impact(&self) -> (i32, i32, bool) {
        match self {
            StatusEffect::Bleeding => (3, 0, false),
            StatusEffect::Stunned => (0, 0, true),
            StatusEffect::Inspired => (0, 0, false),
            StatusEffect::Suppressed => (0, 0, false),
        }
    }
}
