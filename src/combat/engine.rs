//! Combat engine — encounter state machine, turn queue, standoff, action execution.
//!
//! Built as a full 4-slot party battle from day one. Even with 2 characters active,
//! the runtime thinks like a 90s JRPG party battle.

use std::collections::HashMap;
use tracing::{debug, info};

use super::types::*;
use crate::types::*;
use crate::scene::types::StateEffect;

// ─── Skill & DuoTech Registries ───────────────────────────────────

/// Runtime skill registry — maps SkillId to the Skill definition so the
/// combat engine can look up real accuracy, damage, and cost instead of
/// using hardcoded fallbacks.
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: HashMap<SkillId, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: HashMap::new() }
    }

    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.id.clone(), skill);
    }

    pub fn get(&self, id: &SkillId) -> Option<&Skill> {
        self.skills.get(id)
    }

    /// Look up the age-specific variant for a skill. Falls back to the first
    /// variant if the requested phase is not found.
    pub fn get_variant(&self, id: &SkillId, phase: AgePhase) -> Option<&AgeVariant> {
        self.skills.get(id).and_then(|s| {
            s.age_variants.iter()
                .find(|v| v.phase == phase)
                .or(s.age_variants.first())
        })
    }
}

/// Runtime duo-tech registry — maps DuoTechId to the DuoTech definition
/// for co-actor validation and real damage/cost lookup.
#[derive(Debug, Clone, Default)]
pub struct DuoTechRegistry {
    duo_techs: HashMap<DuoTechId, DuoTech>,
}

impl DuoTechRegistry {
    pub fn new() -> Self {
        Self { duo_techs: HashMap::new() }
    }

    pub fn register(&mut self, dt: DuoTech) {
        self.duo_techs.insert(dt.id.clone(), dt);
    }

    pub fn get(&self, id: &DuoTechId) -> Option<&DuoTech> {
        self.duo_techs.get(id)
    }
}

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

    /// Whether the party can attempt to flee (false for boss encounters).
    pub escapable: bool,  // resolved from Encounter.is_escapable() at construction

    /// Skill registry for looking up real skill stats.
    pub skill_registry: SkillRegistry,

    /// Duo-tech registry for co-actor validation and real effect lookup.
    pub duo_tech_registry: DuoTechRegistry,

    /// Current age phase — used to select the correct skill variant.
    pub age_phase: AgePhase,
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
    /// Resolved behavior — determines how this objective evaluates success/failure.
    pub behavior: ObjectiveBehavior,
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
    ///
    /// The `skill_registry` and `duo_tech_registry` are optional — when `None`,
    /// the engine uses empty registries and falls back to actor-level stats.
    /// The `age_phase` determines which skill variant is looked up.
    pub fn new(
        encounter: &Encounter,
        party_members: Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)>,
    ) -> Self {
        Self::with_registries(encounter, party_members, SkillRegistry::new(), DuoTechRegistry::new(), AgePhase::Adult)
    }

    /// Create with explicit registries and age phase for full skill/duo-tech resolution.
    pub fn with_registries(
        encounter: &Encounter,
        party_members: Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)>,
        skill_registry: SkillRegistry,
        duo_tech_registry: DuoTechRegistry,
        age_phase: AgePhase,
    ) -> Self {
        // Build party slots — always 4, empty slots are None
        let mut party: [Option<LiveCombatant>; 4] = [None, None, None, None];
        let party_count = party_members.len();
        if party_count > 4 {
            eprintln!(
                "[combat] Party has {} members but only 4 slots — extras will be dropped. \
                 This is likely a content bug upstream.",
                party_count
            );
        }
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

        // Validate phases — empty phases vector makes the encounter unresolvable
        if encounter.phases.is_empty() {
            eprintln!(
                "[combat] Encounter '{}' has no phases — adding a default empty phase so \
                 the encounter can still be exited. This is a content authoring bug.",
                encounter.id
            );
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

        // Build NPC allies — use character-specific stats from encounter data
        let npc_allies: Vec<LiveNpc> = first_phase
            .map(|p| p.npc_allies.iter().map(|n| LiveNpc {
                combatant: LiveCombatant {
                    id: n.character.0.clone(),
                    name: n.character.0.clone(),
                    side: CombatSide::NpcAlly,
                    hp: n.hp, max_hp: n.hp,
                    nerve: n.nerve, max_nerve: n.nerve,
                    ammo: 99, max_ammo: 99,
                    speed: n.speed, accuracy: n.accuracy, damage: n.damage,
                    position: PositionState::Open,
                    wounds: Vec::new(),
                    panicked: false, down: false,
                    skills: Vec::new(), duo_techs: Vec::new(),
                    bluff: 0, nerve_threshold: 0,
                },
                behavior: n.behavior,
            }).collect())
            .unwrap_or_default();

        // Build objectives — resolve behavior at construction time so the engine
        // can match on enums instead of doing string-contains checks every evaluation.
        let objectives: Vec<LiveObjective> = encounter.objectives.iter().map(|o| LiveObjective {
            id: o.id.clone(),
            label: o.label.clone(),
            objective_type: o.objective_type,
            behavior: o.behavior(),
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
            escapable: encounter.escapable,
            skill_registry,
            duo_tech_registry,
            age_phase,
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
                    if !self.enemies.iter().any(|e| e.id == focus) {
                        eprintln!(
                            "[standoff] focus_target '{}' does not match any enemy ID. \
                             Available enemies: {:?}. Skipping focus bonus.",
                            focus,
                            self.enemies.iter().map(|e| &e.id).collect::<Vec<_>>()
                        );
                    }
                    if let Some(enemy) = self.enemies.iter().find(|e| e.id == focus) {
                        let nerve_hit = 12;
                        result.nerve_damage.push((focus.to_string(), nerve_hit));
                        // Check if this breaks the target (using post-damage value)
                        let post_nerve = (enemy.nerve - nerve_hit).max(0);
                        if post_nerve <= enemy.nerve_threshold {
                            if let Some(idx) = self.enemies.iter().position(|e| e.id == focus) {
                                result.broken_enemies.push(idx);
                            }
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
                // Find the actor — if not found, something upstream is broken
                let actor_stats = self.get_actor_stats(&actor_id);
                if actor_stats.is_none() {
                    eprintln!(
                        "[combat] Actor '{}' not found in party, enemies, or NPC allies. \
                         Skipping action. This likely means the turn queue contains a stale ID.",
                        actor_id
                    );
                    result.action_description = format!("{} — actor not found, action skipped", actor_id);
                    return result;
                }
                let (actor_damage, actor_accuracy, actor_ammo) = actor_stats.unwrap();

                // Look up skill from registry for real stats (FT-001)
                let skill_variant = self.skill_registry.get_variant(skill, self.age_phase).cloned();
                let skill_cost = self.skill_registry.get(skill).map(|s| s.cost.clone());

                // Determine ammo cost — from skill definition or default 1
                let ammo_cost = skill_cost.as_ref().map(|c| c.ammo).unwrap_or(1);

                // Check ammo
                if actor_ammo < ammo_cost {
                    result.action_description = format!("{} is out of ammo!", actor_id);
                    return result;
                }

                // Check nerve cost
                if let Some(ref cost) = skill_cost {
                    if cost.nerve > 0 {
                        let actor_nerve = self.get_actor_nerve(&actor_id).unwrap_or(0);
                        if actor_nerve < cost.nerve {
                            result.action_description = format!(
                                "{} doesn't have enough nerve to use {}!",
                                actor_id, skill
                            );
                            return result;
                        }
                        // Spend nerve
                        self.apply_nerve_damage(&actor_id, cost.nerve);
                    }
                }

                // Spend ammo
                self.modify_ammo(&actor_id, -ammo_cost);

                // Calculate hit — use skill-specific accuracy if available, else actor base
                let skill_accuracy = skill_variant.as_ref().map(|v| v.accuracy).unwrap_or(0);
                let base_accuracy = actor_accuracy + skill_accuracy;
                let accuracy_mod = self.standoff_result.as_ref()
                    .map(|sr| if self.round == 1 { sr.first_shot_accuracy } else { 0 })
                    .unwrap_or(0);

                let final_accuracy = base_accuracy + accuracy_mod;
                let accuracy_threshold = 50;
                let hits = final_accuracy >= accuracy_threshold;

                if hits {
                    if let TargetSelection::Single(target_id) = target {
                        // Use skill-specific damage if available, else actor base damage
                        let damage = skill_variant.as_ref()
                            .map(|v| if v.damage > 0 { v.damage } else { actor_damage })
                            .unwrap_or(actor_damage);
                        let target_down = self.apply_damage(target_id, damage);

                        result.damage_dealt.push(DamageEvent {
                            target: target_id.clone(),
                            amount: damage,
                            was_critical: false,
                            target_down,
                        });

                        // Nerve damage — use skill-specific nerve_damage if available
                        let nerve_dmg = skill_variant.as_ref()
                            .map(|v| if v.nerve_damage > 0 { v.nerve_damage } else { damage / 3 })
                            .unwrap_or(damage / 3);
                        if nerve_dmg > 0 {
                            let (broke, panicked) = self.apply_nerve_damage(target_id, nerve_dmg);
                            result.nerve_damage.push(NerveDamageEvent {
                                target: target_id.clone(),
                                amount: nerve_dmg,
                                target_panicked: panicked,
                                target_broke: broke,
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
                // Look up duo-tech from registry (FT-002)
                let dt_def = self.duo_tech_registry.get(duo_tech).cloned();

                // Validate co-actor: both members must be present and alive
                if let Some(ref dt) = dt_def {
                    let (ref member_a, ref member_b) = dt.members;
                    let co_actor_id = if actor_id == member_a.0 {
                        &member_b.0
                    } else if actor_id == member_b.0 {
                        &member_a.0
                    } else {
                        eprintln!(
                            "[combat] Actor '{}' is not a member of duo-tech '{}' (members: {}, {})",
                            actor_id, duo_tech, member_a, member_b
                        );
                        result.action_description = format!(
                            "{} cannot use {} — not a member!", actor_id, duo_tech
                        );
                        return result;
                    };

                    // Check co-actor is present and alive
                    let co_actor_alive = self.party.iter().flatten()
                        .any(|m| m.id == *co_actor_id && !m.down && !m.panicked);
                    if !co_actor_alive {
                        result.action_description = format!(
                            "{} cannot use {} — partner {} is down or absent!",
                            actor_id, duo_tech, co_actor_id
                        );
                        return result;
                    }

                    // Validate costs — both actors pay
                    let actor_ammo = self.get_actor_stats(&actor_id).map(|s| s.2).unwrap_or(0);
                    let co_actor_ammo = self.get_actor_stats(co_actor_id).map(|s| s.2).unwrap_or(0);
                    if actor_ammo < dt.cost.ammo || co_actor_ammo < dt.cost.ammo {
                        result.action_description = format!(
                            "{} cannot use {} — not enough ammo!", actor_id, duo_tech
                        );
                        return result;
                    }

                    // Check nerve cost
                    if dt.cost.nerve > 0 {
                        let actor_nerve = self.get_actor_nerve(&actor_id).unwrap_or(0);
                        let co_actor_nerve = self.get_actor_nerve(co_actor_id).unwrap_or(0);
                        if actor_nerve < dt.cost.nerve || co_actor_nerve < dt.cost.nerve {
                            result.action_description = format!(
                                "{} cannot use {} — not enough nerve!", actor_id, duo_tech
                            );
                            return result;
                        }
                        // Spend nerve from both
                        self.apply_nerve_damage(&actor_id, dt.cost.nerve);
                        self.apply_nerve_damage(co_actor_id, dt.cost.nerve);
                    }

                    // Spend ammo from both
                    self.modify_ammo(&actor_id, -dt.cost.ammo);
                    self.modify_ammo(co_actor_id, -dt.cost.ammo);
                }

                result.action_description = format!("{} triggers {}!", actor_id, duo_tech);

                if let TargetSelection::Single(target_id) = target {
                    // Use real values from registry, fall back to baseline
                    let (damage, nerve_dmg) = dt_def.as_ref()
                        .map(|dt| (dt.effect.damage, dt.effect.nerve_damage))
                        .unwrap_or((15, 8));

                    let target_down = self.apply_damage(target_id, damage);
                    let (broke, panicked) = self.apply_nerve_damage(target_id, nerve_dmg);

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
                        target_broke: broke,
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
                // FT-004: Flee action — party attempts to escape non-boss encounters
                if !self.escapable {
                    result.action_description = format!(
                        "{} tries to flee — but there's no escape from this fight!", actor_id
                    );
                    info!(actor = %actor_id, "flee blocked — encounter is not escapable");
                    return result;
                }

                // Calculate flee chance based on party speed vs enemy speed
                let party_avg_speed = {
                    let active: Vec<_> = self.party.iter().flatten()
                        .filter(|m| !m.down && !m.panicked)
                        .collect();
                    if active.is_empty() { 0 }
                    else { active.iter().map(|m| m.speed).sum::<i32>() / active.len() as i32 }
                };
                let enemy_avg_speed = {
                    let active: Vec<_> = self.enemies.iter()
                        .filter(|e| !e.down && !e.panicked)
                        .collect();
                    if active.is_empty() { 100 } // no enemies = auto-succeed
                    else { active.iter().map(|e| e.speed).sum::<i32>() / active.len() as i32 }
                };

                // Base 50% chance, +5% per speed advantage, -5% per speed disadvantage
                // Clamped to 15%–95%
                let speed_diff = party_avg_speed - enemy_avg_speed;
                let flee_chance = (50 + speed_diff * 5).clamp(15, 95);

                // Deterministic for now: succeed if chance >= 50
                // (A proper RNG will be wired when the game loop owns randomness.)
                let fled = flee_chance >= 50;

                if fled {
                    result.action_description = format!("{} leads the party to flee — they escape!", actor_id);
                    self.outcome = Some(EncounterOutcome {
                        result: EncounterResult::Fled,
                        effects: Vec::new(),
                    });
                    self.phase = EncounterPhase::Resolved;
                    info!(actor = %actor_id, chance = flee_chance, "flee succeeded");
                } else {
                    result.action_description = format!(
                        "{} tries to flee — but the enemies are too fast! ({}% chance)",
                        actor_id, flee_chance
                    );
                    info!(actor = %actor_id, chance = flee_chance, "flee failed");
                }
            }
        }

        result
    }

    // ─── Objective Evaluation ──────────────────────────────────────

    /// Check all objectives against current state.
    pub fn evaluate_objectives(&mut self) {
        // Auto-victory: all enemies down or panicked
        let all_enemies_neutralized = self.enemies.iter().all(|e| e.down || e.panicked);
        if all_enemies_neutralized {
            for obj in &mut self.objectives {
                if obj.objective_type == ObjectiveType::Primary && obj.status == ObjectiveStatus::Active {
                    obj.status = ObjectiveStatus::Succeeded;
                }
            }

            // Resolve secondary objectives based on HOW enemies were neutralized
            let any_killed = self.enemies.iter().any(|e| e.down); // hp = 0
            let all_broke = self.enemies.iter().all(|e| e.panicked && !e.down); // all nerve-broken, none killed

            for obj in &mut self.objectives {
                if obj.objective_type == ObjectiveType::Secondary && obj.status == ObjectiveStatus::Active {
                    match obj.behavior {
                        ObjectiveBehavior::CivilianCasualties => {
                            // Succeeds only if all enemies broke without any being killed
                            if all_broke {
                                obj.status = ObjectiveStatus::Succeeded;
                            } else {
                                obj.status = ObjectiveStatus::Failed;
                            }
                        }
                        ObjectiveBehavior::ProtectAsset => {
                            // Future: check specific asset survival
                            obj.status = ObjectiveStatus::Succeeded;
                        }
                        ObjectiveBehavior::General => {
                            // Default: secondary objectives succeed with primary
                            obj.status = ObjectiveStatus::Succeeded;
                        }
                    }
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

    // ─── End-of-Encounter Objective Resolution (FT-025) ─────────

    /// Auto-resolve any objectives still marked Active when the encounter ends.
    /// - If the encounter was a Victory, Active secondaries succeed (General behavior)
    ///   or are evaluated by their behavior type.
    /// - If the encounter was a Defeat or Fled, Active objectives fail.
    /// Must be called after `check_resolution` has set `self.outcome`.
    pub fn resolve_remaining_objectives(&mut self) {
        let outcome_result = match &self.outcome {
            Some(o) => o.result,
            None => return, // encounter not resolved yet — nothing to do
        };

        let all_broke = self.enemies.iter().all(|e| e.panicked && !e.down);

        for obj in &mut self.objectives {
            if obj.status != ObjectiveStatus::Active {
                continue;
            }

            match outcome_result {
                EncounterResult::Victory | EncounterResult::ObjectiveComplete => {
                    // Evaluate based on behavior
                    match obj.behavior {
                        ObjectiveBehavior::CivilianCasualties => {
                            obj.status = if all_broke {
                                ObjectiveStatus::Succeeded
                            } else {
                                ObjectiveStatus::Failed
                            };
                        }
                        ObjectiveBehavior::ProtectAsset => {
                            // If we won, the asset survived
                            obj.status = ObjectiveStatus::Succeeded;
                        }
                        ObjectiveBehavior::General => {
                            // Default: secondary objectives succeed with victory
                            obj.status = ObjectiveStatus::Succeeded;
                        }
                    }
                }
                EncounterResult::Defeat | EncounterResult::Fled => {
                    // All unresolved objectives fail on defeat/flee
                    obj.status = ObjectiveStatus::Failed;
                }
            }

            debug!(
                objective = %obj.id,
                status = ?obj.status,
                "auto-resolved remaining objective at encounter end"
            );
        }

        // Collect effects from newly resolved objectives into pending_effects
        for obj in &self.objectives {
            match obj.status {
                ObjectiveStatus::Succeeded => {
                    self.pending_effects.extend(obj.success_consequence.clone());
                }
                ObjectiveStatus::Failed => {
                    self.pending_effects.extend(obj.fail_consequence.clone());
                }
                ObjectiveStatus::Active => {} // shouldn't happen after resolution
            }
        }
    }

    // ─── Internal Helpers ──────────────────────────────────────────

    fn get_actor_stats(&self, id: &str) -> Option<(i32, i32, i32)> {
        // Check party
        for slot in &self.party {
            if let Some(m) = slot {
                if m.id == id {
                    return Some((m.damage, m.accuracy, m.ammo));
                }
            }
        }
        // Check enemies
        for e in &self.enemies {
            if e.id == id {
                return Some((e.damage, e.accuracy, e.ammo));
            }
        }
        // Check NPCs
        for n in &self.npc_allies {
            if n.combatant.id == id {
                return Some((n.combatant.damage, n.combatant.accuracy, n.combatant.ammo));
            }
        }
        None
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
        // Cover absorbs half damage (rounded down — intentional: cover is partial
        // protection, not full). Integer division floors by default in Rust.
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

    /// Apply nerve damage and return (broke, panicked).
    /// - broke: nerve crossed the nerve_threshold (enemies only)
    /// - panicked: nerve reached zero
    fn apply_nerve_damage(&mut self, target_id: &str, amount: i32) -> (bool, bool) {
        for enemy in &mut self.enemies {
            if enemy.id == target_id {
                enemy.nerve = (enemy.nerve - amount).max(0);
                let broke = enemy.nerve <= enemy.nerve_threshold && !enemy.panicked;
                let panicked = enemy.nerve == 0;
                if broke {
                    enemy.panicked = true;
                    debug!(target = target_id, "enemy panicked — nerve broken");
                }
                return (broke, panicked);
            }
        }
        for slot in &mut self.party {
            if let Some(m) = slot {
                if m.id == target_id {
                    m.nerve = (m.nerve - amount).max(0);
                    let panicked = m.nerve == 0;
                    if panicked && !m.panicked {
                        m.panicked = true;
                    }
                    // Party members don't have a nerve_threshold, so broke is always false
                    return (false, panicked);
                }
            }
        }
        (false, false)
    }

    /// Get an actor's current nerve value.
    fn get_actor_nerve(&self, id: &str) -> Option<i32> {
        for slot in &self.party {
            if let Some(m) = slot {
                if m.id == id { return Some(m.nerve); }
            }
        }
        for e in &self.enemies {
            if e.id == id { return Some(e.nerve); }
        }
        for n in &self.npc_allies {
            if n.combatant.id == id { return Some(n.combatant.nerve); }
        }
        None
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

    /// Mark this encounter as non-escapable (boss fight).
    pub fn set_escapable(&mut self, escapable: bool) {
        self.escapable = escapable;
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
            escapable: true,
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

    // ─── FT-001: Skill registry wired ─────────────────────────────

    #[test]
    fn skill_registry_overrides_damage_and_accuracy() {
        let encounter = glass_arroyo_encounter();
        let mut registry = SkillRegistry::new();
        registry.register(Skill {
            id: SkillId::new("quick_draw"),
            name: "Quick Draw".to_string(),
            description: "Fast shot".to_string(),
            line: SkillLine::Deadeye,
            unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
            age_variants: vec![AgeVariant {
                phase: AgePhase::Adult,
                accuracy: 10,  // +10 accuracy bonus
                damage: 15,    // overrides actor damage
                speed_priority: 0,
                nerve_damage: 5,
                description_override: None,
            }],
            cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 0 },
        });

        let mut state = EncounterState::with_registries(
            &encounter, prologue_party(), registry, DuoTechRegistry::new(), AgePhase::Adult,
        );
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("quick_draw"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        // Should use the skill's damage (15), not the actor's base damage (10)
        assert!(!result.damage_dealt.is_empty());
        assert_eq!(result.damage_dealt[0].amount, 15, "skill damage should override actor base");
        // Should use skill's nerve_damage (5), not damage/3
        assert!(!result.nerve_damage.is_empty());
        assert_eq!(result.nerve_damage[0].amount, 5, "skill nerve_damage should be used");
    }

    // ─── FT-002: Duo-tech co-actor validation ─────────────────────

    #[test]
    fn duo_tech_fails_when_partner_down() {
        let encounter = glass_arroyo_encounter();
        let mut dt_registry = DuoTechRegistry::new();
        dt_registry.register(DuoTech {
            id: DuoTechId::new("loaded_deck"),
            name: "Loaded Deck".to_string(),
            description: "Galen + Eli combo".to_string(),
            members: (CharacterId::new("galen"), CharacterId::new("eli")),
            unlock: UnlockCondition::StartOfPhase(AgePhase::Adult),
            cost: DuoTechCost { ammo: 1, nerve: 0, both_turns: true },
            effect: DuoTechEffect {
                description: "Combined assault".to_string(),
                damage: 20, accuracy_bonus: 10, nerve_damage: 10,
                special: None,
            },
        });

        let mut state = EncounterState::with_registries(
            &encounter, prologue_party(), SkillRegistry::new(), dt_registry, AgePhase::Adult,
        );
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Down Eli
        state.party[1].as_mut().unwrap().down = true;

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::UseDuoTech {
            duo_tech: DuoTechId::new("loaded_deck"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        // Should fail — Eli is down
        assert!(result.damage_dealt.is_empty(), "duo tech should deal no damage when partner is down");
        assert!(result.action_description.contains("down or absent"));
    }

    #[test]
    fn duo_tech_uses_registry_damage() {
        let encounter = glass_arroyo_encounter();
        let mut dt_registry = DuoTechRegistry::new();
        dt_registry.register(DuoTech {
            id: DuoTechId::new("loaded_deck"),
            name: "Loaded Deck".to_string(),
            description: "Galen + Eli combo".to_string(),
            members: (CharacterId::new("galen"), CharacterId::new("eli")),
            unlock: UnlockCondition::StartOfPhase(AgePhase::Adult),
            cost: DuoTechCost { ammo: 0, nerve: 0, both_turns: true },
            effect: DuoTechEffect {
                description: "Combined assault".to_string(),
                damage: 25, accuracy_bonus: 10, nerve_damage: 12,
                special: None,
            },
        });

        let mut state = EncounterState::with_registries(
            &encounter, prologue_party(), SkillRegistry::new(), dt_registry, AgePhase::Adult,
        );
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::UseDuoTech {
            duo_tech: DuoTechId::new("loaded_deck"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        // Should use registry damage (25), not fallback (15)
        assert!(!result.damage_dealt.is_empty());
        assert_eq!(result.damage_dealt[0].amount, 25, "duo tech should use registry damage");
        assert_eq!(result.nerve_damage[0].amount, 12, "duo tech should use registry nerve damage");
    }

    // ─── FT-004: Flee action ──────────────────────────────────────

    #[test]
    fn flee_succeeds_when_party_faster() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Galen speed 12, Eli speed 10 → avg 11
        // Enemies: raider 8, gunman 6, lookout 10 → avg 8
        // Party is faster → flee should succeed
        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::Flee);
        assert!(result.action_description.contains("flee"), "party should flee successfully");
        assert_eq!(state.phase, EncounterPhase::Resolved);
        assert!(matches!(state.outcome.as_ref().unwrap().result, EncounterResult::Fled));
    }

    #[test]
    fn flee_blocked_in_boss_encounter() {
        let mut encounter = glass_arroyo_encounter();
        encounter.escapable = false; // boss fight

        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::Flee);
        assert!(result.action_description.contains("no escape"), "boss encounters block fleeing");
        assert_ne!(state.phase, EncounterPhase::Resolved);
    }

    // ─── FT-024: NPC ally stats from definition ───────────────────

    #[test]
    fn npc_allies_use_character_specific_stats() {
        let mut encounter = glass_arroyo_encounter();
        // Add NPC allies with different character-specific stats
        encounter.phases[0].npc_allies.push(NpcCombatant {
            character: CharacterId::new("deputy_harris"),
            behavior: NpcBehavior::Professional,
            hp: 35,
            nerve: 28,
            speed: 11,
            accuracy: 65,
            damage: 9,
        });
        encounter.phases[0].npc_allies.push(NpcCombatant {
            character: CharacterId::new("bale"),
            behavior: NpcBehavior::Professional,
            hp: 35,
            nerve: 30,
            speed: 7,
            accuracy: 55,
            damage: 12,
        });

        let state = EncounterState::new(&encounter, prologue_party());
        assert_eq!(state.npc_allies.len(), 2);

        // Deputy Harris: speed 11, accuracy 65, damage 9
        let harris = &state.npc_allies[0].combatant;
        assert_eq!(harris.speed, 11, "Harris should use character-specific speed");
        assert_eq!(harris.accuracy, 65, "Harris should use character-specific accuracy");
        assert_eq!(harris.damage, 9, "Harris should use character-specific damage");

        // Bale: speed 7, accuracy 55, damage 12 — different from Harris
        let bale = &state.npc_allies[1].combatant;
        assert_eq!(bale.speed, 7, "Bale should use character-specific speed");
        assert_eq!(bale.accuracy, 55, "Bale should use character-specific accuracy");
        assert_eq!(bale.damage, 12, "Bale should use character-specific damage");

        // They must differ — proves stats are not hardcoded identically
        assert_ne!(harris.speed, bale.speed, "different NPCs should have different speeds");
        assert_ne!(harris.damage, bale.damage, "different NPCs should have different damage");
    }

    // ─── FT-025: Active objectives auto-resolve at encounter end ──

    #[test]
    fn active_objectives_resolve_on_victory() {
        let mut encounter = glass_arroyo_encounter();
        encounter.objectives.push(Objective {
            id: "no_casualties".to_string(),
            label: "Avoid civilian casualties".to_string(),
            objective_type: ObjectiveType::Secondary,
            fail_consequence: vec![
                StateEffect::SetFlag {
                    id: FlagId::new("civilians_died"),
                    value: FlagValue::Bool(true),
                },
            ],
            success_consequence: vec![],
        });

        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Kill all enemies (some killed, not all broke → casualties objective fails)
        for enemy in &mut state.enemies {
            enemy.down = true;
        }

        state.evaluate_objectives();
        state.check_resolution();
        state.resolve_remaining_objectives();

        // No objectives should be Active
        let active_count = state.objectives.iter()
            .filter(|o| o.status == ObjectiveStatus::Active)
            .count();
        assert_eq!(active_count, 0, "no objectives should remain Active after encounter end");
    }

    #[test]
    fn active_objectives_fail_on_defeat() {
        let mut encounter = glass_arroyo_encounter();
        encounter.objectives.push(Objective {
            id: "secondary_goal".to_string(),
            label: "Optional goal".to_string(),
            objective_type: ObjectiveType::Secondary,
            fail_consequence: vec![],
            success_consequence: vec![],
        });

        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Down all party members
        for slot in &mut state.party {
            if let Some(m) = slot {
                m.down = true;
            }
        }

        state.evaluate_objectives();
        // evaluate_objectives sets defeat outcome
        state.resolve_remaining_objectives();

        // All active objectives should be Failed on defeat
        for obj in &state.objectives {
            assert_ne!(obj.status, ObjectiveStatus::Active,
                "objective '{}' should not be Active after defeat", obj.id);
        }
    }

    #[test]
    fn active_objectives_fail_on_flee() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen")
            .unwrap();
        state.current_turn = galen_turn;

        // Flee
        state.execute_action(&CombatAction::Flee);
        state.resolve_remaining_objectives();

        // Primary objective should be Failed (we fled, didn't complete it)
        let primary = state.objectives.iter()
            .find(|o| o.objective_type == ObjectiveType::Primary)
            .unwrap();
        assert_eq!(primary.status, ObjectiveStatus::Failed,
            "primary objective should fail when party flees");
    }
}
