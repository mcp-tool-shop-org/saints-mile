//! Convoy runtime — moving world state, escort targets, phase scripting.
//!
//! The convoy is not a combat wrapper. It is a temporary town on wheels.
//! This module tracks what's alive, what's intact, and what the player
//! is protecting across multiple encounters and camp scenes.

use serde::{Deserialize, Serialize};

use crate::types::*;
use crate::scene::types::StateEffect;

/// Live state of a convoy in transit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvoyState {
    /// Named convoy members and their status.
    pub members: Vec<ConvoyMember>,
    /// Wagons and cargo with integrity tracking.
    pub assets: Vec<ConvoyAsset>,
    /// Current day of travel (1-indexed).
    pub day: u8,
    /// Whether it's currently a camp night.
    pub is_night: bool,
    /// Player's formation choice for the current day.
    pub formation: Option<FormationChoice>,
    /// Accumulated convoy tension (rises with incidents, affects camp mood).
    pub tension: i32,
}

/// A named person on the convoy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvoyMember {
    pub id: String,
    pub name: String,
    pub role: ConvoyRole,
    pub alive: bool,
    pub trust_toward_galen: i32,
    /// Whether this person has been talked to at camp.
    pub spoken_to: bool,
}

/// What role someone fills in the moving town.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvoyRole {
    /// Security lead (Bale).
    SecurityLead,
    /// Payroll clerk (Hester).
    Clerk,
    /// Senior teamster (Tom).
    Teamster,
    /// Cook/camp hand (Nella).
    CampHand,
    /// Surveyor/witness (Cask).
    Surveyor,
    /// Hired irregular (Eli).
    Irregular,
    /// Generic crew.
    Crew,
}

/// A wagon or cargo element with integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvoyAsset {
    pub id: String,
    pub name: String,
    pub integrity: i32,
    pub max_integrity: i32,
    /// What happens to game state if this asset is destroyed.
    pub loss_effects: Vec<StateEffect>,
}

/// Where Galen rides in formation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormationChoice {
    /// Near the water cart — best positioned to save it, worst for ridge shooters.
    WaterCart,
    /// Near the payroll coach — good combat position, water cart exposed.
    PayrollCoach,
    /// Forward scout — Trail Eye benefits, separated from convoy when shooting starts.
    ForwardScout,
    /// Rear guard — protects passenger wagon, arrives late to main fight.
    RearGuard,
}

/// A scripted phase transition for multi-phase encounters like the relay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseScript {
    /// What triggers this phase.
    pub trigger: PhaseTrigger,
    /// Description shown to the player.
    pub narration: String,
    /// Objectives that change when this phase fires.
    pub objective_changes: Vec<ObjectiveChange>,
    /// State effects applied when this phase fires.
    pub effects: Vec<StateEffect>,
}

/// What triggers a phase transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseTrigger {
    /// All enemies in current phase are neutralized.
    AllEnemiesDown,
    /// A specific round number is reached.
    Round(u32),
    /// A specific flag is set (by a scripted action).
    FlagSet(String),
    /// An asset's integrity drops below a threshold.
    AssetDamaged { asset_id: String, below: i32 },
    /// A convoy member is killed.
    MemberDied(String),
}

/// How an objective changes during a phase transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectiveChange {
    /// Add a new objective.
    Add { id: String, label: String, is_primary: bool },
    /// Remove an objective.
    Remove(String),
    /// Change an objective's label.
    Relabel { id: String, new_label: String },
    /// Flip an ally to enemy (the relay turn).
    FlipAlly { member_id: String },
}

impl ConvoyState {
    /// Create the Saint's Mile convoy.
    pub fn new_saints_mile_convoy() -> Self {
        Self {
            members: vec![
                ConvoyMember {
                    id: "bale".to_string(),
                    name: "Captain Orrin Bale".to_string(),
                    role: ConvoyRole::SecurityLead,
                    alive: true, trust_toward_galen: 5, spoken_to: false,
                },
                ConvoyMember {
                    id: "hester".to_string(),
                    name: "Hester Vale".to_string(),
                    role: ConvoyRole::Clerk,
                    alive: true, trust_toward_galen: 0, spoken_to: false,
                },
                ConvoyMember {
                    id: "tom".to_string(),
                    name: "Tom Reed".to_string(),
                    role: ConvoyRole::Teamster,
                    alive: true, trust_toward_galen: 3, spoken_to: false,
                },
                ConvoyMember {
                    id: "nella".to_string(),
                    name: "Nella Creed".to_string(),
                    role: ConvoyRole::CampHand,
                    alive: true, trust_toward_galen: 5, spoken_to: false,
                },
                ConvoyMember {
                    id: "cask".to_string(),
                    name: "Old Cask Fen".to_string(),
                    role: ConvoyRole::Surveyor,
                    alive: true, trust_toward_galen: 0, spoken_to: false,
                },
                ConvoyMember {
                    id: "eli_convoy".to_string(),
                    name: "Eli Winter".to_string(),
                    role: ConvoyRole::Irregular,
                    alive: true, trust_toward_galen: 0, spoken_to: false,
                },
            ],
            assets: vec![
                ConvoyAsset {
                    id: "water_cart".to_string(),
                    name: "Water Cart".to_string(),
                    integrity: 100, max_integrity: 100,
                    loss_effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("water_cart_lost"),
                            value: FlagValue::Bool(true),
                        },
                        StateEffect::AdjustResource {
                            resource: ResourceKind::Water,
                            delta: -50,
                        },
                    ],
                },
                ConvoyAsset {
                    id: "payroll_coach".to_string(),
                    name: "Payroll Coach".to_string(),
                    integrity: 100, max_integrity: 100,
                    loss_effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("payroll_lost"),
                            value: FlagValue::Bool(true),
                        },
                    ],
                },
                ConvoyAsset {
                    id: "powder_wagon".to_string(),
                    name: "Powder Wagon".to_string(),
                    integrity: 80, max_integrity: 80,
                    loss_effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("powder_wagon_exploded"),
                            value: FlagValue::Bool(true),
                        },
                    ],
                },
                ConvoyAsset {
                    id: "passenger_wagon".to_string(),
                    name: "Passenger Wagon".to_string(),
                    integrity: 60, max_integrity: 60,
                    loss_effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("passenger_wagon_lost"),
                            value: FlagValue::Bool(true),
                        },
                    ],
                },
            ],
            day: 1,
            is_night: false,
            formation: None,
            tension: 0,
        }
    }

    /// Advance to the next day.
    pub fn advance_day(&mut self) {
        self.day += 1;
        self.is_night = false;
    }

    /// Set to nighttime (camp).
    pub fn set_night(&mut self) {
        self.is_night = true;
    }

    /// Get a member by ID.
    pub fn member(&self, id: &str) -> Option<&ConvoyMember> {
        self.members.iter().find(|m| m.id == id)
    }

    /// Get a mutable member by ID.
    pub fn member_mut(&mut self, id: &str) -> Option<&mut ConvoyMember> {
        self.members.iter_mut().find(|m| m.id == id)
    }

    /// Mark a member as spoken to at camp.
    pub fn speak_to(&mut self, id: &str) {
        if let Some(m) = self.member_mut(id) {
            m.spoken_to = true;
        }
    }

    /// Kill a convoy member.
    pub fn kill_member(&mut self, id: &str) {
        if let Some(m) = self.member_mut(id) {
            m.alive = false;
        }
    }

    /// Damage an asset. Returns loss effects if destroyed.
    pub fn damage_asset(&mut self, id: &str, amount: i32) -> Vec<StateEffect> {
        if let Some(asset) = self.assets.iter_mut().find(|a| a.id == id) {
            asset.integrity = (asset.integrity - amount).max(0);
            if asset.integrity == 0 {
                return asset.loss_effects.clone();
            }
        }
        Vec::new()
    }

    /// Get asset integrity.
    pub fn asset_integrity(&self, id: &str) -> Option<i32> {
        self.assets.iter().find(|a| a.id == id).map(|a| a.integrity)
    }

    /// Count alive members.
    pub fn alive_count(&self) -> usize {
        self.members.iter().filter(|m| m.alive).count()
    }

    /// Check if a specific member is alive.
    pub fn is_alive(&self, id: &str) -> bool {
        self.member(id).map_or(false, |m| m.alive)
    }

    /// Write the convoy's final state into game state effects.
    pub fn finalize(&self) -> Vec<StateEffect> {
        let mut effects = Vec::new();

        // Record member survival
        for member in &self.members {
            effects.push(StateEffect::SetFlag {
                id: FlagId::new(format!("convoy_{}_alive", member.id)),
                value: FlagValue::Bool(member.alive),
            });
        }

        // Record asset integrity
        for asset in &self.assets {
            if asset.integrity == 0 {
                effects.extend(asset.loss_effects.clone());
            }
        }

        effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convoy_creates_with_full_cast() {
        let convoy = ConvoyState::new_saints_mile_convoy();
        assert_eq!(convoy.members.len(), 6);
        assert_eq!(convoy.assets.len(), 4);
        assert!(convoy.is_alive("bale"));
        assert!(convoy.is_alive("nella"));
        assert!(convoy.is_alive("tom"));
        assert_eq!(convoy.day, 1);
    }

    #[test]
    fn asset_damage_and_destruction() {
        let mut convoy = ConvoyState::new_saints_mile_convoy();

        // Damage water cart
        let effects = convoy.damage_asset("water_cart", 60);
        assert!(effects.is_empty()); // not destroyed yet
        assert_eq!(convoy.asset_integrity("water_cart"), Some(40));

        // Destroy it
        let effects = convoy.damage_asset("water_cart", 50);
        assert!(!effects.is_empty()); // loss effects fire
        assert_eq!(convoy.asset_integrity("water_cart"), Some(0));
    }

    #[test]
    fn member_death_and_finalize() {
        let mut convoy = ConvoyState::new_saints_mile_convoy();

        convoy.kill_member("bale");
        assert!(!convoy.is_alive("bale"));
        assert_eq!(convoy.alive_count(), 5);

        let effects = convoy.finalize();
        // Should have survival flags for all members
        assert!(effects.iter().any(|e| matches!(e,
            StateEffect::SetFlag { id, value: FlagValue::Bool(false) }
            if id.0 == "convoy_bale_alive"
        )));
        assert!(effects.iter().any(|e| matches!(e,
            StateEffect::SetFlag { id, value: FlagValue::Bool(true) }
            if id.0 == "convoy_nella_alive"
        )));
    }

    #[test]
    fn day_night_progression() {
        let mut convoy = ConvoyState::new_saints_mile_convoy();
        assert_eq!(convoy.day, 1);
        assert!(!convoy.is_night);

        convoy.set_night();
        assert!(convoy.is_night);

        convoy.advance_day();
        assert_eq!(convoy.day, 2);
        assert!(!convoy.is_night);
    }
}
