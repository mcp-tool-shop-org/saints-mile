//! Shared types and ID newtypes used across all modules.

use serde::{Deserialize, Serialize};

/// Generate a newtype wrapper around String for type-safe IDs.
macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(s: impl Into<String>) -> Self {
                Self(s.into())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

// All ID types in one place — used across scene, combat, state, pressure.
id_type!(SceneId);
id_type!(LocationId);
id_type!(BeatId);
id_type!(ChapterId);
id_type!(SpeakerId);
id_type!(FlagId);
id_type!(EncounterId);
id_type!(CharacterId);
id_type!(SkillId);
id_type!(WitnessId);
id_type!(EvidenceId);
id_type!(MemoryObjectId);
id_type!(InjuryId);
id_type!(DuoTechId);

/// Flag values — simple typed values for game state flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlagValue {
    Bool(bool),
    Int(i32),
    Text(String),
}

/// Comparison operators for conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Gt,
    Gte,
    Lt,
    Lte,
    Eq,
    Neq,
}

/// Reputation axes — the web, not a score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReputationAxis {
    TownLaw,
    Railroad,
    Rancher,
}

/// The three relay rescue branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayBranch {
    Tom,
    Nella,
    Papers,
}

/// The prologue Beat 5 choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrologueChoice {
    TownDirect,
    HomesteadFirst,
}

/// Trail resource kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Water,
    Ammo,
    HorseStamina,
}

/// The four life phases — age changes the command menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgePhase {
    Youth,
    YoungMan,
    Adult,
    Older,
}
