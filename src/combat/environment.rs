//! Environmental combat — destructible terrain, fuse charges, chain reactions.
//!
//! After Chapter 6, the player knows the field itself can change. Cover is
//! not permanent. Structures collapse. Fuses burn. The ground is not guaranteed.

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Live environmental state during combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    /// Destructible cover elements with remaining durability.
    pub cover: Vec<LiveCover>,
    /// Active fuse charges counting down to detonation.
    pub fuse_charges: Vec<FuseCharge>,
    /// Structural elements that can collapse.
    pub structures: Vec<Structure>,
    /// Blast events that occurred this encounter (for post-combat narrative).
    pub blast_log: Vec<BlastEvent>,
}

/// A piece of cover during live combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCover {
    pub id: String,
    pub name: String,
    pub durability: i32,
    pub max_durability: i32,
    pub destroyed: bool,
}

/// A fuse charge counting down.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuseCharge {
    pub id: String,
    pub turns_remaining: u8,
    pub blast_damage: i32,
    pub blast_radius: u8,
    /// Whether this is the structural charge (brings the whole thing down).
    pub is_structural: bool,
    /// Whether this charge has been disarmed.
    pub disarmed: bool,
    /// Whether this charge has detonated.
    pub detonated: bool,
}

/// A structural element that can collapse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: String,
    pub name: String,
    pub integrity: i32,
    pub collapse_threshold: i32,
    pub collapsed: bool,
    /// What happens to combatants when this collapses.
    pub collapse_damage: i32,
}

/// A recorded blast event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastEvent {
    pub source: String,
    pub damage: i32,
    pub cover_destroyed: Vec<String>,
    pub structure_damaged: Vec<String>,
    pub chain_reaction: bool,
}

/// Actions that can be taken on the environment.
#[derive(Debug, Clone)]
pub enum EnvironmentAction {
    /// Shoot a fuse to disarm (requires high accuracy).
    ShootFuse { charge_id: String, accuracy: i32 },
    /// Physically reach and disarm a charge.
    DisarmCharge { charge_id: String },
    /// Destroy cover element.
    DestroyCover { cover_id: String, damage: i32 },
    /// Trigger a controlled detonation.
    ControlledDetonate { charge_id: String },
    /// Collapse a structure deliberately.
    CollapseStructure { structure_id: String },
}

impl EnvironmentState {
    pub fn new() -> Self {
        Self {
            cover: Vec::new(),
            fuse_charges: Vec::new(),
            structures: Vec::new(),
            blast_log: Vec::new(),
        }
    }

    /// Advance all fuse charges by one turn. Returns detonation events.
    pub fn tick_fuses(&mut self) -> Vec<BlastEvent> {
        let mut events = Vec::new();

        // Collect detonation params before mutating
        let detonations: Vec<(String, i32, bool)> = self.fuse_charges.iter()
            .filter(|c| !c.disarmed && !c.detonated && c.turns_remaining <= 1)
            .map(|c| (c.id.clone(), c.blast_damage, c.is_structural))
            .collect();

        // Mark as detonated
        for (id, _, _) in &detonations {
            if let Some(charge) = self.fuse_charges.iter_mut().find(|c| c.id == *id) {
                charge.detonated = true;
            }
        }

        // Process detonations
        for (id, damage, is_structural) in detonations {
            let event = self.detonate_charge(id, damage, is_structural);
            events.push(event);
        }

        // Tick remaining charges
        for charge in &mut self.fuse_charges {
            if !charge.disarmed && !charge.detonated && charge.turns_remaining > 0 {
                charge.turns_remaining -= 1;
                debug!(charge = %charge.id, remaining = charge.turns_remaining, "fuse ticking");
            }
        }

        events
    }

    /// Process a detonation — destroy cover, damage structures, chain reactions.
    fn detonate_charge(&mut self, source: String, damage: i32, is_structural: bool) -> BlastEvent {
        let mut cover_destroyed = Vec::new();
        let mut structure_damaged = Vec::new();
        let mut chain = false;

        // Blast damages nearby cover
        for cover in &mut self.cover {
            if !cover.destroyed {
                cover.durability -= damage / 2;
                if cover.durability <= 0 {
                    cover.destroyed = true;
                    cover_destroyed.push(cover.name.clone());
                }
            }
        }

        // Structural charge damages structures
        if is_structural {
            for structure in &mut self.structures {
                if !structure.collapsed {
                    structure.integrity -= damage;
                    structure_damaged.push(structure.name.clone());
                    if structure.integrity <= structure.collapse_threshold {
                        structure.collapsed = true;
                        debug!(structure = %structure.name, "structure collapsed!");
                    }
                }
            }
        }

        // Chain reaction: if a charge detonates near another un-detonated charge
        for other in &mut self.fuse_charges {
            if !other.disarmed && !other.detonated && other.turns_remaining > 1 {
                other.turns_remaining = 1; // accelerate
                chain = true;
                debug!(charge = %other.id, "chain reaction — fuse accelerated");
            }
        }

        let event = BlastEvent {
            source,
            damage,
            cover_destroyed,
            structure_damaged,
            chain_reaction: chain,
        };
        self.blast_log.push(event.clone());
        event
    }

    /// Execute an environment action. Returns success/failure.
    pub fn execute_action(&mut self, action: &EnvironmentAction) -> EnvironmentActionResult {
        match action {
            EnvironmentAction::ShootFuse { charge_id, accuracy } => {
                if let Some(charge) = self.fuse_charges.iter_mut().find(|c| c.id == *charge_id) {
                    if charge.disarmed || charge.detonated {
                        return EnvironmentActionResult::Failed("already resolved".to_string());
                    }
                    // High accuracy required — miss hits the charge itself
                    if *accuracy >= 65 {
                        charge.disarmed = true;
                        debug!(charge = %charge_id, "fuse shot — disarmed");
                        EnvironmentActionResult::Success(format!("Shot the fuse on {} — disarmed", charge_id))
                    } else {
                        // Miss — accelerate the fuse
                        if charge.turns_remaining > 1 {
                            charge.turns_remaining -= 1;
                        }
                        EnvironmentActionResult::Failed(format!("Missed the fuse — charge accelerated"))
                    }
                } else {
                    EnvironmentActionResult::Failed("charge not found".to_string())
                }
            }

            EnvironmentAction::DisarmCharge { charge_id } => {
                if let Some(charge) = self.fuse_charges.iter_mut().find(|c| c.id == *charge_id) {
                    if charge.disarmed || charge.detonated {
                        return EnvironmentActionResult::Failed("already resolved".to_string());
                    }
                    charge.disarmed = true;
                    EnvironmentActionResult::Success(format!("Physically disarmed {}", charge_id))
                } else {
                    EnvironmentActionResult::Failed("charge not found".to_string())
                }
            }

            EnvironmentAction::DestroyCover { cover_id, damage } => {
                if let Some(cover) = self.cover.iter_mut().find(|c| c.id == *cover_id) {
                    cover.durability -= damage;
                    if cover.durability <= 0 {
                        cover.destroyed = true;
                        EnvironmentActionResult::Success(format!("{} destroyed", cover.name))
                    } else {
                        EnvironmentActionResult::Success(format!("{} damaged ({} remaining)", cover.name, cover.durability))
                    }
                } else {
                    EnvironmentActionResult::Failed("cover not found".to_string())
                }
            }

            EnvironmentAction::ControlledDetonate { charge_id } => {
                // Collect params before mutating
                let params = self.fuse_charges.iter()
                    .find(|c| c.id == *charge_id)
                    .map(|c| (c.detonated, c.blast_damage, c.is_structural));

                match params {
                    Some((true, _, _)) =>
                        EnvironmentActionResult::Failed("already detonated".to_string()),
                    Some((false, damage, is_structural)) => {
                        if let Some(charge) = self.fuse_charges.iter_mut().find(|c| c.id == *charge_id) {
                            charge.detonated = true;
                        }
                        let event = self.detonate_charge(charge_id.clone(), damage, is_structural);
                        EnvironmentActionResult::Detonation(event)
                    }
                    None => EnvironmentActionResult::Failed("charge not found".to_string()),
                }
            }

            EnvironmentAction::CollapseStructure { structure_id } => {
                if let Some(structure) = self.structures.iter_mut().find(|s| s.id == *structure_id) {
                    structure.collapsed = true;
                    EnvironmentActionResult::Success(format!("{} collapsed", structure.name))
                } else {
                    EnvironmentActionResult::Failed("structure not found".to_string())
                }
            }
        }
    }

    /// Count active (non-disarmed, non-detonated) charges.
    pub fn active_charges(&self) -> usize {
        self.fuse_charges.iter().filter(|c| !c.disarmed && !c.detonated).count()
    }

    /// Count intact cover.
    pub fn intact_cover(&self) -> usize {
        self.cover.iter().filter(|c| !c.destroyed).count()
    }

    /// Check if any structure has collapsed.
    pub fn any_collapsed(&self) -> bool {
        self.structures.iter().any(|s| s.collapsed)
    }
}

/// Result of an environment action.
#[derive(Debug, Clone)]
pub enum EnvironmentActionResult {
    Success(String),
    Failed(String),
    Detonation(BlastEvent),
}

/// Build a trestle environment for the Chapter 6 set piece.
pub fn trestle_environment() -> EnvironmentState {
    EnvironmentState {
        cover: vec![
            LiveCover {
                id: "pylon_crossbeam".to_string(),
                name: "Pylon crossbeam".to_string(),
                durability: 40, max_durability: 40, destroyed: false,
            },
            LiveCover {
                id: "supply_crate".to_string(),
                name: "Supply crate".to_string(),
                durability: 20, max_durability: 20, destroyed: false,
            },
            LiveCover {
                id: "rail_car".to_string(),
                name: "Rail car".to_string(),
                durability: 80, max_durability: 80, destroyed: false,
            },
        ],
        fuse_charges: vec![
            FuseCharge {
                id: "pylon_charge_1".to_string(),
                turns_remaining: 3,
                blast_damage: 20,
                blast_radius: 2,
                is_structural: false,
                disarmed: false,
                detonated: false,
            },
            FuseCharge {
                id: "pylon_charge_2".to_string(),
                turns_remaining: 4,
                blast_damage: 20,
                blast_radius: 2,
                is_structural: false,
                disarmed: false,
                detonated: false,
            },
            FuseCharge {
                id: "structural_charge".to_string(),
                turns_remaining: 5,
                blast_damage: 35,
                blast_radius: 4,
                is_structural: true,
                disarmed: false,
                detonated: false,
            },
        ],
        structures: vec![
            Structure {
                id: "trestle_span".to_string(),
                name: "Trestle main span".to_string(),
                integrity: 100,
                collapse_threshold: 30,
                collapsed: false,
                collapse_damage: 25,
            },
        ],
        blast_log: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuse_ticks_and_detonates() {
        let mut env = trestle_environment();
        assert_eq!(env.active_charges(), 3);

        // Tick 3 times — first charge should detonate
        env.tick_fuses(); // 3→2, 4→3, 5→4
        env.tick_fuses(); // 2→1, 3→2, 4→3
        let events = env.tick_fuses(); // 1→BOOM, 2→1, 3→2

        assert!(!events.is_empty(), "charge should have detonated");
        assert_eq!(env.active_charges(), 2);
    }

    #[test]
    fn shoot_fuse_requires_accuracy() {
        let mut env = trestle_environment();

        // Low accuracy misses and accelerates
        let result = env.execute_action(&EnvironmentAction::ShootFuse {
            charge_id: "pylon_charge_1".to_string(),
            accuracy: 50,
        });
        assert!(matches!(result, EnvironmentActionResult::Failed(_)));

        let charge = env.fuse_charges.iter().find(|c| c.id == "pylon_charge_1").unwrap();
        assert_eq!(charge.turns_remaining, 2, "miss should accelerate fuse");

        // High accuracy disarms
        let result = env.execute_action(&EnvironmentAction::ShootFuse {
            charge_id: "pylon_charge_2".to_string(),
            accuracy: 70,
        });
        assert!(matches!(result, EnvironmentActionResult::Success(_)));

        let charge = env.fuse_charges.iter().find(|c| c.id == "pylon_charge_2").unwrap();
        assert!(charge.disarmed);
    }

    #[test]
    fn structural_charge_collapses_trestle() {
        let mut env = trestle_environment();

        // Detonate the structural charge
        let result = env.execute_action(&EnvironmentAction::ControlledDetonate {
            charge_id: "structural_charge".to_string(),
        });

        assert!(matches!(result, EnvironmentActionResult::Detonation(_)));

        // Trestle should be damaged/collapsed
        let trestle = env.structures.iter().find(|s| s.id == "trestle_span").unwrap();
        assert!(trestle.integrity < 100);
    }

    #[test]
    fn chain_reaction_accelerates_other_charges() {
        let mut env = trestle_environment();

        // Detonate charge 1 — should accelerate others
        env.execute_action(&EnvironmentAction::ControlledDetonate {
            charge_id: "pylon_charge_1".to_string(),
        });

        // Other charges should be accelerated
        let charge_2 = env.fuse_charges.iter().find(|c| c.id == "pylon_charge_2").unwrap();
        assert_eq!(charge_2.turns_remaining, 1, "chain reaction should accelerate");
    }

    #[test]
    fn cover_destruction() {
        let mut env = trestle_environment();
        assert_eq!(env.intact_cover(), 3);

        env.execute_action(&EnvironmentAction::DestroyCover {
            cover_id: "supply_crate".to_string(),
            damage: 25,
        });

        assert_eq!(env.intact_cover(), 2);
        let crate_cover = env.cover.iter().find(|c| c.id == "supply_crate").unwrap();
        assert!(crate_cover.destroyed);
    }

    #[test]
    fn physical_disarm() {
        let mut env = trestle_environment();

        let result = env.execute_action(&EnvironmentAction::DisarmCharge {
            charge_id: "structural_charge".to_string(),
        });
        assert!(matches!(result, EnvironmentActionResult::Success(_)));
        assert_eq!(env.active_charges(), 2);
    }
}
