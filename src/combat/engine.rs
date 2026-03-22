//! Combat engine — encounter state machine, turn queue, standoff, action execution.
//!
//! Built as a full 4-slot party battle from day one. Even with 2 characters active,
//! the runtime thinks like a 90s JRPG party battle.

use tracing::{debug, info};

use super::types::*;
use crate::types::*;
use crate::scene::types::StateEffect;

// ─── Encounter State Machine ───────────────────────────────────────

/// The phases an encounter moves through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterPhase {
    /// Standoff pre-phase — posture selection, nerve testing, initiative earned.
    Standoff,
    /// Active combat — turn-based party battle.
    Combat,
    /// Encounter resolved — victory, defeat, or objective-based outcome.
    Resolved,
}

/// Live state of an encounter in progress.
#[derive(Debug)]
pub struct EncounterState {
    /// Current phase of the encounter.
    pub phase: EncounterPhase,
    /// Which combat phase (for multi-phase encounters like the relay).
    pub combat_phase_index: usize,

    /// Party combatants — always 4 slots. Empty slots are None.
    pub party: [Option<LiveCombatant>; 4],
    /// Enemy combatants.
    pub enemies: Vec<LiveCombatant>,
    /// NPC allies (uncontrollable).
    pub npc_allies: Vec<LiveNpc>,

    /// Turn queue — ordered by speed, modified by standoff.
    pub turn_queue: Vec<TurnEntry>,
    /// Current turn index within the queue.
    pub current_turn: usize,
    /// Round number.
    pub round: u32,

    /// Standoff result (if standoff phase completed).
    pub standoff_result: Option<StandoffResult>,

    /// Active objectives.
    pub objectives: Vec<LiveObjective>,

    /// Terrain state.
    pub terrain: Terrain,

    /// Accumulated state effects to apply after the encounter.
    pub pending_effects: Vec<StateEffect>,

    /// Whether the encounter is over and why.
    pub outcome: Option<EncounterOutcome>,
}

/// A combatant during live combat.
#[derive(Debug, Clone)]
pub struct LiveCombatant {
    pub id: String,
    pub name: String,
    pub side: CombatSide,

    pub hp: i32,
    pub max_hp: i32,
    pub nerve: i32,
    pub max_nerve: i32,
    pub ammo: i32,
    pub max_ammo: i32,

    pub speed: i32,
    pub accuracy: i32,
    pub damage: i32,

    /// Current position/cover state.
    pub position: PositionState,
    /// Active wounds.
    pub wounds: Vec<Wound>,
    /// Whether this combatant has panicked (nerve = 0).
    pub panicked: bool,
    /// Whether this combatant is down (hp = 0).
    pub down: bool,

    /// Skills available to this combatant (empty for enemies).
    pub skills: Vec<SkillId>,
    /// Duo techs available (checked against active party).
    pub duo_techs: Vec<DuoTechId>,

    /// Bluff stat (for standoff reads). Enemies only.
    pub bluff: i32,
    /// Nerve threshold (how close to breaking). Enemies only.
    pub nerve_threshold: i32,
}

/// Which side a combatant is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSide {
    Party,
    Enemy,
    NpcAlly,
}

/// An NPC ally in combat — acts on their own.
#[derive(Debug, Clone)]
pub struct LiveNpc {
    pub combatant: LiveCombatant,
    pub behavior: NpcBehavior,
}

/// An entry in the turn queue.
#[derive(Debug, Clone)]
pub struct TurnEntry {
    pub combatant_id: String,
    pub side: CombatSide,
    pub speed: i32,
    /// Modified by standoff result.
    pub initiative_mod: i32,
}

impl TurnEntry {
    pub fn effective_speed(&self) -> i32 {
        self.speed + self.initiative_mod
    }
}

/// A live objective being tracked during combat.
#[derive(Debug, Clone)]
pub struct LiveObjective {
    pub id: String,
    pub label: String,
    pub objective_type: ObjectiveType,
    pub status: ObjectiveStatus,
    pub fail_consequence: Vec<StateEffect>,
    pub success_consequence: Vec<StateEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveStatus {
    Active,
    Succeeded,
    Failed,
}

/// How the encounter ended.
#[derive(Debug, Clone)]
pub struct EncounterOutcome {
    pub result: EncounterResult,
    pub effects: Vec<StateEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterResult {
    Victory,
    Defeat,
    Fled,
    ObjectiveComplete,
}

// ─── Combat Actions ────────────────────────────────────────────────

/// An action a combatant can take on their turn.
#[derive(Debug, Clone)]
pub enum CombatAction {
    /// Use a skill on a target.
    UseSkill { skill: SkillId, target: TargetSelection },
    /// Use a duo technique (both members spend their turn).
    UseDuoTech { duo_tech: DuoTechId, target: TargetSelection },
    /// Take cover — move to cover position.
    TakeCover,
    /// Pass / defend.
    Defend,
    /// Attempt to flee.
    Flee,
}

/// Target selection for an action.
#[derive(Debug, Clone)]
pub enum TargetSelection {
    Single(String),
    AllEnemies,
    AllAllies,
    Self_,
}

/// The result of executing an action.
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub actor: String,
    pub action_description: String,
    pub damage_dealt: Vec<DamageEvent>,
    pub nerve_damage: Vec<NerveDamageEvent>,
    pub healing: Vec<HealEvent>,
    pub status_changes: Vec<StatusChange>,
    pub skill_unlocks: Vec<(CharacterId, SkillId)>,
}

#[derive(Debug, Clone)]
pub struct DamageEvent {
    pub target: String,
    pub amount: i32,
    pub was_critical: bool,
    pub target_down: bool,
}

#[derive(Debug, Clone)]
pub struct NerveDamageEvent {
    pub target: String,
    pub amount: i32,
    pub target_panicked: bool,
    pub target_broke: bool,
}

#[derive(Debug, Clone)]
pub struct HealEvent {
    pub target: String,
    pub amount: i32,
}

#[derive(Debug, Clone)]
pub struct StatusChange {
    pub target: String,
    pub change: String,
}

// ─── Engine Implementation ─────────────────────────────────────────

impl EncounterState {
    /// Create a new encounter from a definition and the party's current state.
    pub fn new(
        encounter: &Encounter,
        party_members: Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)>,
    ) -> Self {
        // Build party slots — always 4, empty slots are None
        let mut party: [Option<LiveCombatant>; 4] = [None, None, None, None];
        for (i, (id, name, hp, nerve, ammo, speed, accuracy, damage, skills, duo_techs, wounds)) in
            party_members.into_iter().enumerate()
        {
            if i >= 4 { break; }
            party[i] = Some(LiveCombatant {
                id,
                name,
                side: CombatSide::Party,
                hp, max_hp: hp,
                nerve, max_nerve: nerve,
                ammo, max_ammo: ammo,
                speed, accuracy, damage,
                position: PositionState::Open,
                wounds,
                panicked: false,
                down: false,
                skills,
                duo_techs,
                bluff: 0,
                nerve_threshold: 0,
            });
        }

        // Build enemies from the first phase
        let first_phase = encounter.phases.first();
        let enemies: Vec<LiveCombatant> = first_phase
            .map(|p| p.enemies.iter().enumerate().map(|(i, e)| LiveCombatant {
                id: format!("{}_{}", e.id, i),
                name: e.name.clone(),
                side: CombatSide::Enemy,
                hp: e.hp, max_hp: e.hp,
                nerve: e.nerve, max_nerve: e.nerve,
                ammo: 99, max_ammo: 99, // enemies don't track ammo the same way
                speed: e.speed, accuracy: e.accuracy, damage: e.damage,
                position: PositionState::Open,
                wounds: Vec::new(),
                panicked: false, down: false,
                skills: Vec::new(), duo_techs: Vec::new(),
                bluff: e.bluff,
                nerve_threshold: e.nerve_threshold,
            }).collect())
            .unwrap_or_default();

        // Build NPC allies
        let npc_allies: Vec<LiveNpc> = first_phase
            .map(|p| p.npc_allies.iter().map(|n| LiveNpc {
                combatant: LiveCombatant {
                    id: n.character.0.clone(),
                    name: n.character.0.clone(),
                    side: CombatSide::NpcAlly,
                    hp: n.hp, max_hp: n.hp,
                    nerve: n.nerve, max_nerve: n.nerve,
                    ammo: 99, max_ammo: 99,
                    speed: 10, accuracy: 60, damage: 8,
                    position: PositionState::Open,
                    wounds: Vec::new(),
                    panicked: false, down: false,
                    skills: Vec::new(), duo_techs: Vec::new(),
                    bluff: 0, nerve_threshold: 0,
                },
                behavior: n.behavior,
            }).collect())
            .unwrap_or_default();

        // Build objectives
        let objectives: Vec<LiveObjective> = encounter.objectives.iter().map(|o| LiveObjective {
            id: o.id.clone(),
            label: o.label.clone(),
            objective_type: o.objective_type,
            status: ObjectiveStatus::Active,
            fail_consequence: o.fail_consequence.clone(),
            success_consequence: o.success_consequence.clone(),
        }).collect();

        let phase = if encounter.standoff.is_some() {
            EncounterPhase::Standoff
        } else {
            EncounterPhase::Combat
        };

        EncounterState {
            phase,
            combat_phase_index: 0,
            party,
            enemies,
            npc_allies,
            turn_queue: Vec::new(),
            current_turn: 0,
            round: 0,
            standoff_result: None,
            objectives,
            terrain: encounter.terrain.clone(),
            pending_effects: Vec::new(),
            outcome: None,
        }
    }

    // ─── Standoff Resolution ───────────────────────────────────────

    /// Resolve the standoff phase given the player's choices.
    pub fn resolve_standoff(
        &mut self,
        posture: StandoffPosture,
        focus_target: Option<&str>,
    ) -> StandoffResult {
        info!(
            posture = ?posture,
            focus = ?focus_target,
            "standoff resolved"
        );

        let mut result = StandoffResult::default();

        // Posture effects on initiative and nerve
        match posture {
            StandoffPosture::EarlyDraw => {
                // Player acts first but less accurately. High nerve damage to enemies.
                for entry in self.party.iter().flatten() {
                    result.initiative_mods.push((entry.id.clone(), 5));
                }
                result.first_shot_accuracy = -15;
                for enemy in &self.enemies {
                    result.nerve_damage.push((enemy.id.clone(), 8));
                }
            }
            StandoffPosture::SteadyHand => {
                // Balanced. Moderate initiative, no accuracy penalty.
                for entry in self.party.iter().flatten() {
                    result.initiative_mods.push((entry.id.clone(), 2));
                }
                result.first_shot_accuracy = 0;
                for enemy in &self.enemies {
                    result.nerve_damage.push((enemy.id.clone(), 3));
                }
            }
            StandoffPosture::Bait => {
                // Provoke. Risk taking a hit, but target exposed.
                for entry in self.party.iter().flatten() {
                    result.initiative_mods.push((entry.id.clone(), -2));
                }
                result.first_shot_accuracy = 10;
                // Focused target takes extra nerve damage and may break
                if let Some(focus) = focus_target {
                    if let Some(enemy) = self.enemies.iter().find(|e| e.id == focus) {
                        let nerve_hit = 12;
                        result.nerve_damage.push((focus.to_string(), nerve_hit));
                        // Check if this breaks the target
                        if enemy.nerve - nerve_hit <= enemy.nerve_threshold {
                            result.broken_enemies.push(
                                self.enemies.iter().position(|e| e.id == focus).unwrap_or(0)
                            );
                        }
                    }
                }
            }
        }

        // Apply nerve damage from standoff
        for (target_id, amount) in &result.nerve_damage {
            if let Some(enemy) = self.enemies.iter_mut().find(|e| e.id == *target_id) {
                enemy.nerve = (enemy.nerve - amount).max(0);
                if enemy.nerve <= enemy.nerve_threshold {
                    enemy.panicked = true;
                    debug!(target = %target_id, "enemy broke during standoff");
                }
            }
        }

        // Mark broken enemies
        for &idx in &result.broken_enemies {
            if let Some(enemy) = self.enemies.get_mut(idx) {
                enemy.panicked = true;
            }
        }

        self.standoff_result = Some(result.clone());
        self.phase = EncounterPhase::Combat;
        result
    }

    // ─── Turn Queue ────────────────────────────────────────────────

    /// Build the turn queue for a new round, ordered by effective speed.
    pub fn build_turn_queue(&mut self) {
        self.turn_queue.clear();
        self.current_turn = 0;
        self.round += 1;

        let standoff_mods = self.standoff_result.as_ref();

        // Add party members
        for slot in &self.party {
            if let Some(member) = slot {
                if member.down || member.panicked { continue; }
                let init_mod = standoff_mods
                    .and_then(|sr| sr.initiative_mods.iter()
                        .find(|(id, _)| id == &member.id)
                        .map(|(_, m)| *m))
                    .unwrap_or(0);

                self.turn_queue.push(TurnEntry {
                    combatant_id: member.id.clone(),
                    side: CombatSide::Party,
                    speed: member.speed,
                    initiative_mod: init_mod,
                });
            }
        }

        // Add enemies
        for enemy in &self.enemies {
            if enemy.down || enemy.panicked { continue; }
            self.turn_queue.push(TurnEntry {
                combatant_id: enemy.id.clone(),
                side: CombatSide::Enemy,
                speed: enemy.speed,
                initiative_mod: 0,
            });
        }

        // Add NPC allies
        for npc in &self.npc_allies {
            if npc.combatant.down || npc.combatant.panicked { continue; }
            self.turn_queue.push(TurnEntry {
                combatant_id: npc.combatant.id.clone(),
                side: CombatSide::NpcAlly,
                speed: npc.combatant.speed,
                initiative_mod: 0,
            });
        }

        // Sort by effective speed, descending (fastest acts first)
        self.turn_queue.sort_by(|a, b| b.effective_speed().cmp(&a.effective_speed()));

        debug!(
            round = self.round,
            queue_size = self.turn_queue.len(),
            first = ?self.turn_queue.first().map(|t| &t.combatant_id),
            "turn queue built"
        );
    }

    /// Get the current turn entry.
    pub fn current_turn_entry(&self) -> Option<&TurnEntry> {
        self.turn_queue.get(self.current_turn)
    }

    /// Advance to the next turn. Returns false if the round is over.
    pub fn advance_turn(&mut self) -> bool {
        self.current_turn += 1;
        if self.current_turn >= self.turn_queue.len() {
            debug!(round = self.round, "round complete");
            false
        } else {
            true
        }
    }

    // ─── Action Execution ──────────────────────────────────────────

    /// Execute a combat action for the current turn's combatant.
    pub fn execute_action(&mut self, action: &CombatAction) -> ActionResult {
        let actor_id = self.current_turn_entry()
            .map(|t| t.combatant_id.clone())
            .unwrap_or_default();

        let mut result = ActionResult {
            actor: actor_id.clone(),
            action_description: String::new(),
            damage_dealt: Vec::new(),
            nerve_damage: Vec::new(),
            healing: Vec::new(),
            status_changes: Vec::new(),
            skill_unlocks: Vec::new(),
        };

        match action {
            CombatAction::UseSkill { skill, target } => {
                // Find the actor
                let (actor_damage, actor_accuracy, actor_ammo) = self.get_actor_stats(&actor_id);

                // Check ammo
                if actor_ammo <= 0 {
                    result.action_description = format!("{} is out of ammo!", actor_id);
                    return result;
                }

                // Spend ammo
                self.modify_ammo(&actor_id, -1);

                // Calculate hit
                let accuracy_mod = self.standoff_result.as_ref()
                    .map(|sr| if self.round == 1 { sr.first_shot_accuracy } else { 0 })
                    .unwrap_or(0);

                let final_accuracy = actor_accuracy + accuracy_mod;
                let hits = final_accuracy >= 50; // simplified for skeleton

                if hits {
                    if let TargetSelection::Single(target_id) = target {
                        let damage = actor_damage;
                        let target_down = self.apply_damage(target_id, damage);

                        result.damage_dealt.push(DamageEvent {
                            target: target_id.clone(),
                            amount: damage,
                            was_critical: false,
                            target_down,
                        });

                        // Nerve damage on hit
                        let nerve_dmg = damage / 3;
                        if nerve_dmg > 0 {
                            let panicked = self.apply_nerve_damage(target_id, nerve_dmg);
                            result.nerve_damage.push(NerveDamageEvent {
                                target: target_id.clone(),
                                amount: nerve_dmg,
                                target_panicked: panicked,
                                target_broke: panicked,
                            });
                        }
                    }

                    result.action_description = format!(
                        "{} uses {} — hit!", actor_id, skill
                    );
                } else {
                    result.action_description = format!(
                        "{} uses {} — missed.", actor_id, skill
                    );
                }

                info!(
                    actor = %actor_id,
                    skill = %skill,
                    hit = hits,
                    "skill used"
                );
            }

            CombatAction::UseDuoTech { duo_tech, target } => {
                // Duo tech: both members act, combined effect
                result.action_description = format!("{} triggers {}!", actor_id, duo_tech);

                if let TargetSelection::Single(target_id) = target {
                    // Duo techs deal increased damage and nerve damage
                    let damage = 15; // placeholder — will be data-driven
                    let nerve_dmg = 8;

                    let target_down = self.apply_damage(target_id, damage);
                    let panicked = self.apply_nerve_damage(target_id, nerve_dmg);

                    result.damage_dealt.push(DamageEvent {
                        target: target_id.clone(),
                        amount: damage,
                        was_critical: true,
                        target_down,
                    });
                    result.nerve_damage.push(NerveDamageEvent {
                        target: target_id.clone(),
                        amount: nerve_dmg,
                        target_panicked: panicked,
                        target_broke: panicked,
                    });
                }

                info!(actor = %actor_id, duo_tech = %duo_tech, "duo tech used");
            }

            CombatAction::TakeCover => {
                self.set_position(&actor_id, PositionState::InCover);
                result.action_description = format!("{} takes cover.", actor_id);
                result.status_changes.push(StatusChange {
                    target: actor_id.clone(),
                    change: "moved to cover".to_string(),
                });
            }

            CombatAction::Defend => {
                result.action_description = format!("{} defends.", actor_id);
            }

            CombatAction::Flee => {
                result.action_description = format!("{} attempts to flee.", actor_id);
                // Flee logic would check conditions
            }
        }

        result
    }

    // ─── Objective Evaluation ──────────────────────────────────────

    /// Check all objectives against current state.
    pub fn evaluate_objectives(&mut self) {
        // Auto-victory: all enemies down
        let all_enemies_down = self.enemies.iter().all(|e| e.down || e.panicked);
        if all_enemies_down {
            for obj in &mut self.objectives {
                if obj.objective_type == ObjectiveType::Primary && obj.status == ObjectiveStatus::Active {
                    obj.status = ObjectiveStatus::Succeeded;
                }
            }
        }

        // Auto-defeat: all party members down
        let all_party_down = self.party.iter().flatten().all(|p| p.down);
        if all_party_down {
            self.outcome = Some(EncounterOutcome {
                result: EncounterResult::Defeat,
                effects: Vec::new(),
            });
        }
    }

    /// Check if the encounter is resolved.
    pub fn check_resolution(&mut self) -> Option<&EncounterOutcome> {
        if self.outcome.is_some() {
            return self.outcome.as_ref();
        }

        // Check if all primary objectives are resolved
        let primaries: Vec<_> = self.objectives.iter()
            .filter(|o| o.objective_type == ObjectiveType::Primary)
            .collect();

        if primaries.iter().all(|o| o.status == ObjectiveStatus::Succeeded) {
            // Collect effects from succeeded objectives
            let mut effects = Vec::new();
            for obj in &self.objectives {
                match obj.status {
                    ObjectiveStatus::Succeeded => effects.extend(obj.success_consequence.clone()),
                    ObjectiveStatus::Failed => effects.extend(obj.fail_consequence.clone()),
                    ObjectiveStatus::Active => {}
                }
            }

            self.outcome = Some(EncounterOutcome {
                result: EncounterResult::Victory,
                effects,
            });
            self.phase = EncounterPhase::Resolved;
        }

        self.outcome.as_ref()
    }

    // ─── Internal Helpers ──────────────────────────────────────────

    fn get_actor_stats(&self, id: &str) -> (i32, i32, i32) {
        // Check party
        for slot in &self.party {
            if let Some(m) = slot {
                if m.id == id {
                    return (m.damage, m.accuracy, m.ammo);
                }
            }
        }
        // Check enemies
        for e in &self.enemies {
            if e.id == id {
                return (e.damage, e.accuracy, e.ammo);
            }
        }
        // Check NPCs
        for n in &self.npc_allies {
            if n.combatant.id == id {
                return (n.combatant.damage, n.combatant.accuracy, n.combatant.ammo);
            }
        }
        (0, 0, 0)
    }

    fn modify_ammo(&mut self, id: &str, delta: i32) {
        for slot in &mut self.party {
            if let Some(m) = slot {
                if m.id == id {
                    m.ammo = (m.ammo + delta).max(0);
                    return;
                }
            }
        }
    }

    fn apply_damage(&mut self, target_id: &str, damage: i32) -> bool {
        // Check cover reduction
        let in_cover = self.get_position(target_id) == Some(PositionState::InCover);
        let actual_damage = if in_cover { damage / 2 } else { damage };

        // Apply to enemies
        for enemy in &mut self.enemies {
            if enemy.id == target_id {
                enemy.hp = (enemy.hp - actual_damage).max(0);
                if enemy.hp == 0 {
                    enemy.down = true;
                    debug!(target = target_id, "enemy down");
                }
                return enemy.down;
            }
        }
        // Apply to party
        for slot in &mut self.party {
            if let Some(m) = slot {
                if m.id == target_id {
                    m.hp = (m.hp - actual_damage).max(0);
                    if m.hp == 0 {
                        m.down = true;
                    }
                    return m.down;
                }
            }
        }
        false
    }

    fn apply_nerve_damage(&mut self, target_id: &str, amount: i32) -> bool {
        for enemy in &mut self.enemies {
            if enemy.id == target_id {
                enemy.nerve = (enemy.nerve - amount).max(0);
                if enemy.nerve <= enemy.nerve_threshold && !enemy.panicked {
                    enemy.panicked = true;
                    debug!(target = target_id, "enemy panicked — nerve broken");
                    return true;
                }
                return false;
            }
        }
        for slot in &mut self.party {
            if let Some(m) = slot {
                if m.id == target_id {
                    m.nerve = (m.nerve - amount).max(0);
                    if m.nerve == 0 && !m.panicked {
                        m.panicked = true;
                        return true;
                    }
                    return false;
                }
            }
        }
        false
    }

    fn get_position(&self, id: &str) -> Option<PositionState> {
        for slot in &self.party {
            if let Some(m) = slot {
                if m.id == id { return Some(m.position); }
            }
        }
        for e in &self.enemies {
            if e.id == id { return Some(e.position); }
        }
        None
    }

    fn set_position(&mut self, id: &str, pos: PositionState) {
        for slot in &mut self.party {
            if let Some(m) = slot {
                if m.id == id { m.position = pos; return; }
            }
        }
        for e in &mut self.enemies {
            if e.id == id { e.position = pos; return; }
        }
    }

    /// Count active (non-down, non-panicked) party members.
    pub fn active_party_count(&self) -> usize {
        self.party.iter().flatten().filter(|m| !m.down && !m.panicked).count()
    }

    /// Count active enemies.
    pub fn active_enemy_count(&self) -> usize {
        self.enemies.iter().filter(|e| !e.down && !e.panicked).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glass_arroyo_encounter() -> Encounter {
        Encounter {
            id: EncounterId::new("glass_arroyo"),
            phases: vec![CombatPhase {
                id: "main".to_string(),
                description: "Armed group at the wagon wreck".to_string(),
                enemies: vec![
                    EnemyTemplate {
                        id: "raider".to_string(),
                        name: "Ridge Raider".to_string(),
                        hp: 25, nerve: 20, damage: 8, accuracy: 55,
                        speed: 8, bluff: 30, nerve_threshold: 5,
                    },
                    EnemyTemplate {
                        id: "gunman".to_string(),
                        name: "Hired Gunman".to_string(),
                        hp: 30, nerve: 25, damage: 10, accuracy: 60,
                        speed: 6, bluff: 15, nerve_threshold: 8,
                    },
                    EnemyTemplate {
                        id: "lookout".to_string(),
                        name: "Nervous Lookout".to_string(),
                        hp: 15, nerve: 10, damage: 5, accuracy: 40,
                        speed: 10, bluff: 50, nerve_threshold: 8,
                    },
                ],
                npc_allies: vec![],
                entry_conditions: vec![],
                phase_effects: vec![],
            }],
            standoff: Some(Standoff {
                postures: vec![
                    StandoffPosture::EarlyDraw,
                    StandoffPosture::SteadyHand,
                    StandoffPosture::Bait,
                ],
                allow_focus: true,
                eli_influence: true,
            }),
            party_slots: 4,
            terrain: Terrain {
                name: "Glass Arroyo".to_string(),
                cover: vec![
                    CoverElement { name: "Wagon wreck".to_string(), durability: 50, destructible: true },
                    CoverElement { name: "Rock outcrop".to_string(), durability: 100, destructible: false },
                ],
                hazards: vec![],
            },
            objectives: vec![Objective {
                id: "survive".to_string(),
                label: "Survive the ambush".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![],
                success_consequence: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("glass_arroyo_survived"),
                        value: FlagValue::Bool(true),
                    },
                ],
            }],
            outcome_effects: vec![],
        }
    }

    fn prologue_party() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
        vec![
            (
                "galen".to_string(), "Galen Rook".to_string(),
                40, 30, 12, // hp, nerve, ammo
                12, 70, 10, // speed, accuracy, damage
                vec![
                    SkillId::new("quick_draw"),
                    SkillId::new("called_shot"),
                    SkillId::new("take_cover"),
                    SkillId::new("rally"),
                    SkillId::new("setup_shot"),
                    SkillId::new("overwatch"),
                ],
                vec![DuoTechId::new("loaded_deck")],
                vec![],
            ),
            (
                "eli".to_string(), "Eli Winter".to_string(),
                30, 25, 8,
                10, 50, 6,
                vec![
                    SkillId::new("sidearm"),
                    SkillId::new("fast_talk"),
                    SkillId::new("bluff"),
                    SkillId::new("dirty_trick"),
                    SkillId::new("patch_up"),
                ],
                vec![DuoTechId::new("loaded_deck")],
                vec![],
            ),
        ]
    }

    #[test]
    fn encounter_creates_with_4_slots() {
        let encounter = glass_arroyo_encounter();
        let state = EncounterState::new(&encounter, prologue_party());

        // 4 slots always exist
        assert!(state.party[0].is_some()); // Galen
        assert!(state.party[1].is_some()); // Eli
        assert!(state.party[2].is_none()); // Empty slot 3
        assert!(state.party[3].is_none()); // Empty slot 4

        assert_eq!(state.enemies.len(), 3);
        assert_eq!(state.phase, EncounterPhase::Standoff);
    }

    #[test]
    fn standoff_early_draw() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        let result = state.resolve_standoff(StandoffPosture::EarlyDraw, None);

        // Party gets initiative boost
        assert!(result.initiative_mods.iter().any(|(id, m)| id == "galen" && *m > 0));
        // Accuracy penalty on first shot
        assert!(result.first_shot_accuracy < 0);
        // Enemies take nerve damage
        assert!(!result.nerve_damage.is_empty());
        // Phase transitions to combat
        assert_eq!(state.phase, EncounterPhase::Combat);
    }

    #[test]
    fn standoff_bait_can_break_target() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        // Bait focused on the nervous lookout (low nerve, high threshold)
        let result = state.resolve_standoff(StandoffPosture::Bait, Some("lookout_2"));

        // Lookout should break — nerve 10, threshold 8, we deal 12
        let lookout = state.enemies.iter().find(|e| e.id == "lookout_2").unwrap();
        assert!(lookout.panicked, "lookout should have panicked");
        assert!(!result.broken_enemies.is_empty());
    }

    #[test]
    fn turn_queue_speed_ordered() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Queue should be non-empty and ordered by effective speed descending
        assert!(!state.turn_queue.is_empty());
        for i in 1..state.turn_queue.len() {
            assert!(
                state.turn_queue[i - 1].effective_speed() >= state.turn_queue[i].effective_speed(),
                "turn queue not ordered: {} >= {} failed",
                state.turn_queue[i - 1].effective_speed(),
                state.turn_queue[i].effective_speed(),
            );
        }

        // Galen (speed 12 + 2 init) should be near the top
        let galen_pos = state.turn_queue.iter().position(|t| t.combatant_id == "galen").unwrap();
        assert!(galen_pos <= 1, "Galen should be in top 2, was at {}", galen_pos);
    }

    #[test]
    fn action_execution_uses_ammo() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Find Galen's turn
        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        let ammo_before = state.party[0].as_ref().unwrap().ammo;

        let _result = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("quick_draw"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        let ammo_after = state.party[0].as_ref().unwrap().ammo;
        assert_eq!(ammo_after, ammo_before - 1, "ammo should decrease by 1");
    }

    #[test]
    fn cover_reduces_damage() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Put raider in cover
        state.set_position("raider_0", PositionState::InCover);
        let hp_before = state.enemies[0].hp;

        // Hit them with 10 damage — cover should halve it to 5
        state.apply_damage("raider_0", 10);
        let hp_after = state.enemies[0].hp;

        assert_eq!(hp_before - hp_after, 5, "cover should halve damage");
    }

    #[test]
    fn nerve_break_causes_panic() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Lookout has nerve 10, threshold 8. After standoff nerve damage (3),
        // nerve is 7. Apply 5 more to bring to 2 (below threshold 8).
        // Wait — threshold means panic when nerve <= threshold.
        // After standoff: nerve = 10 - 3 = 7. Threshold = 8.
        // 7 <= 8, so lookout should already be panicked from standoff.
        let lookout = state.enemies.iter().find(|e| e.id == "lookout_2").unwrap();
        // Nerve was 10, standoff dealt 3, so 7. Threshold is 8. 7 <= 8 → panicked.
        assert!(lookout.panicked, "lookout should be panicked after standoff");
    }

    #[test]
    fn objective_resolution() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Kill all enemies
        for enemy in &mut state.enemies {
            enemy.down = true;
        }

        state.evaluate_objectives();
        let outcome = state.check_resolution();

        assert!(outcome.is_some());
        let outcome = outcome.unwrap();
        assert_eq!(outcome.result, EncounterResult::Victory);
        assert!(outcome.effects.iter().any(|e| matches!(e,
            StateEffect::SetFlag { id, value: FlagValue::Bool(true) }
            if id.0 == "glass_arroyo_survived"
        )));
    }

    #[test]
    fn panicked_enemies_excluded_from_turn_queue() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        // Break the lookout during standoff
        state.resolve_standoff(StandoffPosture::Bait, Some("lookout_2"));
        state.build_turn_queue();

        // Panicked lookout should not be in the turn queue
        assert!(
            !state.turn_queue.iter().any(|t| t.combatant_id == "lookout_2"),
            "panicked enemy should be excluded from turn queue"
        );
    }

    #[test]
    fn full_combat_round_flows() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        // Standoff
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        assert_eq!(state.phase, EncounterPhase::Combat);

        // Build queue
        state.build_turn_queue();
        assert!(!state.turn_queue.is_empty());
        assert_eq!(state.round, 1);

        // Execute turns until round ends
        let mut actions_taken = 0;
        loop {
            let entry = state.current_turn_entry().cloned();
            if entry.is_none() { break; }
            let entry = entry.unwrap();

            let action = match entry.side {
                CombatSide::Party => CombatAction::UseSkill {
                    skill: SkillId::new("quick_draw"),
                    target: TargetSelection::Single(
                        state.enemies.iter()
                            .find(|e| !e.down && !e.panicked)
                            .map(|e| e.id.clone())
                            .unwrap_or_default()
                    ),
                },
                CombatSide::Enemy => CombatAction::UseSkill {
                    skill: SkillId::new("attack"),
                    target: TargetSelection::Single("galen".to_string()),
                },
                CombatSide::NpcAlly => CombatAction::Defend,
            };

            let _result = state.execute_action(&action);
            actions_taken += 1;

            if !state.advance_turn() { break; }
        }

        assert!(actions_taken > 0, "at least one action should have been taken");

        // Check objectives
        state.evaluate_objectives();
        // May or may not be resolved after one round — that's fine
    }

    #[test]
    fn duo_tech_execution() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());

        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Find Galen's turn
        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        let enemy_hp_before = state.enemies[0].hp;

        let result = state.execute_action(&CombatAction::UseDuoTech {
            duo_tech: DuoTechId::new("loaded_deck"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        // Duo tech should deal damage
        assert!(!result.damage_dealt.is_empty());
        assert!(result.damage_dealt[0].amount > 0);
        assert!(state.enemies[0].hp < enemy_hp_before);

        // And nerve damage
        assert!(!result.nerve_damage.is_empty());
    }
}
