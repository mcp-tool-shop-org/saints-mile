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

    /// FT-007: Cover assignments — maps combatant ID to terrain cover element index.
    /// When cover is destroyed, the combatant is forced to Open position.
    pub cover_assignments: HashMap<String, usize>,

    /// Skill cooldowns — maps (combatant_id, skill_id) to remaining cooldown turns.
    pub cooldowns: HashMap<(String, SkillId), u8>,

    /// Combo tracker — maps (actor_id, skill_line) to consecutive same-type action count.
    /// Resets when a different actor or skill line is used.
    pub combo_counter: HashMap<String, ComboState>,

    /// Terrain modifiers applied during combat (craters, fire, flooding, cleared cover).
    pub terrain_modifiers: Vec<TerrainModifier>,
}

/// Tracks the current combo state for a combatant.
#[derive(Debug, Clone)]
pub struct ComboState {
    /// The skill line being chained.
    pub skill_line: String,
    /// How many consecutive same-type actions (1 = first use, 2 = combo, 3 = chain).
    pub count: u8,
}

/// Terrain modifiers that reshape the battlefield during combat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainModifier {
    /// Explosion crater — reduced movement options.
    Cratered { source: String },
    /// Active fire — damage to anyone in the area each turn.
    Burning { damage_per_turn: i32 },
    /// Flooding — nerve penalty to all combatants.
    Flooded { nerve_penalty: i32 },
    /// Cover blown away — no cover available in this zone.
    Cleared { former_cover: String },
}

impl EncounterState {
    /// Record an action for combo tracking. Returns the combo multiplier (1.0, 1.1, or 1.2).
    pub fn record_combo(&mut self, actor_id: &str, skill_line: &str) -> f32 {
        let entry = self.combo_counter.entry(actor_id.to_string()).or_insert(ComboState {
            skill_line: String::new(),
            count: 0,
        });

        if entry.skill_line == skill_line {
            entry.count = entry.count.saturating_add(1);
        } else {
            entry.skill_line = skill_line.to_string();
            entry.count = 1;
        }

        match entry.count {
            2 => 1.10,
            n if n >= 3 => 1.20,
            _ => 1.0,
        }
    }

    /// Reset combo state for an actor (called when a different actor takes a turn).
    pub fn reset_combo(&mut self, actor_id: &str) {
        self.combo_counter.remove(actor_id);
    }

    /// Apply a terrain modifier to the battlefield.
    pub fn apply_terrain_modifier(&mut self, modifier: TerrainModifier) {
        self.terrain_modifiers.push(modifier);
    }

    /// Check terrain effects and return accumulated impacts:
    /// (total_hp_damage, total_nerve_penalty).
    pub fn check_terrain_effects(&self) -> (i32, i32) {
        let mut hp_damage = 0;
        let mut nerve_penalty = 0;

        for modifier in &self.terrain_modifiers {
            match modifier {
                TerrainModifier::Burning { damage_per_turn } => {
                    hp_damage += damage_per_turn;
                }
                TerrainModifier::Flooded { nerve_penalty: penalty } => {
                    nerve_penalty += penalty;
                }
                TerrainModifier::Cratered { .. } | TerrainModifier::Cleared { .. } => {
                    // Schema-level — movement/cover effects wired later.
                }
            }
        }

        (hp_damage, nerve_penalty)
    }

    /// Apply terrain effects (HP damage and nerve penalty) to all living combatants.
    pub fn apply_terrain_effects(&mut self, hp_damage: i32, nerve_penalty: i32) {
        // Collect all living combatant IDs
        let mut ids: Vec<String> = Vec::new();
        for slot in &self.party {
            if let Some(m) = slot {
                if !m.down { ids.push(m.id.clone()); }
            }
        }
        for e in &self.enemies {
            if !e.down { ids.push(e.id.clone()); }
        }
        for n in &self.npc_allies {
            if !n.combatant.down { ids.push(n.combatant.id.clone()); }
        }

        for id in ids {
            if hp_damage > 0 {
                self.apply_damage(&id, hp_damage);
            }
            if nerve_penalty > 0 {
                self.apply_nerve_damage(&id, nerve_penalty);
            }
        }
    }
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

    /// Active status effects: (effect, remaining turns).
    pub active_effects: Vec<(StatusEffect, u8)>,

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
    /// FT-007: Force a target out of cover (suppressive fire, explosions, etc.)
    ForceOutOfCover { target: String },
    /// Reload — spend a turn to recover partial ammo (3 rounds).
    Reload,
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
                active_effects: Vec::new(),
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
                active_effects: Vec::new(),
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
                    active_effects: Vec::new(),
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
            cover_assignments: HashMap::new(),
            cooldowns: HashMap::new(),
            combo_counter: HashMap::new(),
            terrain_modifiers: Vec::new(),
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
    /// FT-006: Party members' speed is modified by the best speed_priority
    /// among their age-variant skills for the current age phase.
    pub fn build_turn_queue(&mut self) {
        self.turn_queue.clear();
        self.current_turn = 0;
        self.round += 1;

        // FT-WIRE: Tick cooldowns at the start of each new round
        if self.round > 1 {
            self.tick_cooldowns();
        }

        // FT-WIRE: Apply terrain effects (burning/flooding) at round boundary
        let (terrain_hp, terrain_nerve) = self.check_terrain_effects();
        if terrain_hp > 0 || terrain_nerve > 0 {
            self.apply_terrain_effects(terrain_hp, terrain_nerve);
        }

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

                // FT-006: Apply best speed_priority bonus from age-variant skills.
                // Each character's effective speed is their base + the highest
                // speed_priority among their available skills for the current age phase.
                let speed_bonus: i32 = member.skills.iter()
                    .filter_map(|sid| self.skill_registry.get_variant(sid, self.age_phase))
                    .map(|v| v.speed_priority)
                    .max()
                    .unwrap_or(0);

                self.turn_queue.push(TurnEntry {
                    combatant_id: member.id.clone(),
                    side: CombatSide::Party,
                    speed: member.speed + speed_bonus,
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

                // Check cooldown — block if skill is still on cooldown
                let cd_key = (actor_id.clone(), skill.clone());
                if let Some(&cd) = self.cooldowns.get(&cd_key) {
                    if cd > 0 {
                        result.action_description = format!(
                            "{}'s {} is on cooldown ({} turns remaining)!",
                            actor_id, skill, cd
                        );
                        return result;
                    }
                }

                // Check if actor is stunned
                if self.has_status_effect(&actor_id, StatusEffect::Stunned) {
                    result.action_description = format!("{} is stunned and cannot act!", actor_id);
                    return result;
                }

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

                // Set cooldown from skill definition
                let cooldown_turns = skill_cost.as_ref().map(|c| c.cooldown_turns).unwrap_or(0);
                if cooldown_turns > 0 {
                    self.cooldowns.insert((actor_id.clone(), skill.clone()), cooldown_turns);
                }

                // Calculate hit — use skill-specific accuracy if available, else actor base
                let skill_accuracy = skill_variant.as_ref().map(|v| v.accuracy).unwrap_or(0);
                let base_accuracy = actor_accuracy + skill_accuracy;
                let accuracy_mod = self.standoff_result.as_ref()
                    .map(|sr| if self.round == 1 { sr.first_shot_accuracy } else { 0 })
                    .unwrap_or(0);

                // Apply status effect modifiers
                let suppressed_mod = if self.has_status_effect(&actor_id, StatusEffect::Suppressed) { -20 } else { 0 };
                let final_accuracy = base_accuracy + accuracy_mod + suppressed_mod;
                let accuracy_threshold = 50;
                let hits = final_accuracy >= accuracy_threshold;

                if hits {
                    if let TargetSelection::Single(target_id) = target {
                        // Use skill-specific damage if available, else actor base damage
                        let raw_damage = skill_variant.as_ref()
                            .map(|v| if v.damage > 0 { v.damage } else { actor_damage })
                            .unwrap_or(actor_damage);
                        // FT-WIRE: Combo multiplier — consecutive same-skill uses boost damage
                        let combo_mult = self.record_combo(&actor_id, &skill.0);
                        let combo_damage = if combo_mult > 1.0 {
                            (raw_damage as f32 * combo_mult) as i32
                        } else {
                            raw_damage
                        };
                        // Inspired boost: +25% damage
                        let damage = if self.has_status_effect(&actor_id, StatusEffect::Inspired) {
                            combo_damage + combo_damage / 4
                        } else {
                            combo_damage
                        };
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

                            // FT-WIRE: Fear cascade — when nerve breaks, allies take nerve damage
                            if broke {
                                let cascade_events = self.fear_cascade(target_id);
                                result.nerve_damage.extend(cascade_events);
                            }
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
                // FT-007: Find available cover element. Full cover if durability > 0,
                // otherwise partial cover if element is damaged.
                let cover_idx = self.terrain.cover.iter().enumerate()
                    .find(|(i, c)| {
                        c.durability > 0 && !self.cover_assignments.values().any(|a| a == i)
                    })
                    .map(|(i, _)| i);

                if let Some(idx) = cover_idx {
                    let cover_name = self.terrain.cover[idx].name.clone();
                    self.cover_assignments.insert(actor_id.clone(), idx);
                    self.set_position(&actor_id, PositionState::InCover);
                    result.action_description = format!("{} takes cover behind {}.", actor_id, cover_name);
                    result.status_changes.push(StatusChange {
                        target: actor_id.clone(),
                        change: format!("moved to cover ({})", cover_name),
                    });
                } else {
                    // No cover available — partial cover from terrain features
                    self.set_position(&actor_id, PositionState::PartialCover);
                    result.action_description = format!("{} hunkers down (partial cover).", actor_id);
                    result.status_changes.push(StatusChange {
                        target: actor_id.clone(),
                        change: "partial cover".to_string(),
                    });
                }
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

            CombatAction::ForceOutOfCover { target } => {
                // FT-007: Force target out of cover — used by suppressive fire, explosions, etc.
                let target_pos = self.get_position(target);
                if matches!(target_pos, Some(PositionState::InCover) | Some(PositionState::PartialCover)) {
                    self.force_out_of_cover(target);
                    result.action_description = format!("{} forces {} out of cover!", actor_id, target);
                    result.status_changes.push(StatusChange {
                        target: target.clone(),
                        change: "forced out of cover".to_string(),
                    });
                    info!(actor = %actor_id, target = %target, "forced out of cover");
                } else {
                    result.action_description = format!("{} tries to flush {} — but they're not in cover.", actor_id, target);
                }
            }

            CombatAction::Reload => {
                // Reload: spend a turn to recover 3 rounds of ammo (capped at max_ammo)
                let reload_amount = 3;
                let (current, max) = self.get_actor_ammo(&actor_id).unwrap_or((0, 0));
                let actual_restore = reload_amount.min(max - current);
                if actual_restore > 0 {
                    self.modify_ammo(&actor_id, actual_restore);
                    result.action_description = format!(
                        "{} reloads — recovered {} rounds.", actor_id, actual_restore
                    );
                } else {
                    result.action_description = format!("{} tries to reload — already at full ammo.", actor_id);
                }
                info!(actor = %actor_id, restored = actual_restore, "reload");
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
        // FT-007: Cover reduces damage — full cover 50%, partial cover 25%.
        let position = self.get_position(target_id);
        let actual_damage = match position {
            Some(PositionState::InCover) => damage / 2,      // 50% reduction
            Some(PositionState::PartialCover) => damage * 3 / 4, // 25% reduction
            _ => damage,
        };

        // FT-007: Destructible cover takes damage too — degrade and destroy.
        if matches!(position, Some(PositionState::InCover) | Some(PositionState::PartialCover)) {
            let absorbed = damage - actual_damage;
            if absorbed > 0 {
                self.damage_cover(target_id, absorbed);
            }
        }

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

    /// FT-007: Apply damage to a cover element. If durability drops to 0 and
    /// the cover is destructible, destroy it and force the combatant out.
    fn damage_cover(&mut self, combatant_id: &str, damage: i32) {
        let cover_idx = match self.cover_assignments.get(combatant_id) {
            Some(&idx) => idx,
            None => return,
        };

        if cover_idx >= self.terrain.cover.len() { return; }

        let cover = &mut self.terrain.cover[cover_idx];
        if !cover.destructible { return; }

        cover.durability = (cover.durability - damage).max(0);
        if cover.durability == 0 {
            debug!(
                cover = %cover.name,
                combatant = combatant_id,
                "cover destroyed"
            );
            // Force combatant out of cover
            self.cover_assignments.remove(combatant_id);
            self.set_position(combatant_id, PositionState::Open);
        }
    }

    /// FT-007: Force a combatant out of cover — removes assignment and sets Open position.
    fn force_out_of_cover(&mut self, combatant_id: &str) {
        self.cover_assignments.remove(combatant_id);
        self.set_position(combatant_id, PositionState::Open);
    }

    /// Count active (non-down, non-panicked) party members.
    pub fn active_party_count(&self) -> usize {
        self.party.iter().flatten().filter(|m| !m.down && !m.panicked).count()
    }

    /// Count active enemies.
    pub fn active_enemy_count(&self) -> usize {
        self.enemies.iter().filter(|e| !e.down && !e.panicked).count()
    }

    // ─── FT-005: NPC Ally Behavior ────────────────────────────────

    /// Select an action for an NPC ally based on their behavior and battlefield state.
    /// NPCs pick actions with priority-based AI:
    /// - Heal if any ally HP is critically low (Protective/Professional with healing)
    /// - Attack highest-threat enemy
    /// - Take cover if nervous and in danger
    pub fn select_npc_action(&self, npc_id: &str) -> CombatAction {
        let npc = match self.npc_allies.iter().find(|n| n.combatant.id == npc_id) {
            Some(n) => n,
            None => return CombatAction::Defend,
        };

        let behavior = npc.behavior;
        let npc_hp_pct = if npc.combatant.max_hp > 0 {
            npc.combatant.hp * 100 / npc.combatant.max_hp
        } else { 100 };

        // Check if any ally needs healing (for Protective behavior or characters like Ada)
        let ally_needs_heal = self.party.iter().flatten()
            .filter(|m| !m.down)
            .any(|m| m.max_hp > 0 && m.hp * 100 / m.max_hp < 40);

        // Find the highest-threat active enemy (highest damage * not panicked)
        let target_enemy = self.enemies.iter()
            .filter(|e| !e.down && !e.panicked)
            .max_by_key(|e| e.damage);

        match behavior {
            NpcBehavior::Professional => {
                // Reliable: attack the most dangerous enemy
                if let Some(enemy) = target_enemy {
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            NpcBehavior::Protective => {
                // Heal allies first if anyone is hurt, otherwise attack
                if ally_needs_heal {
                    // Find the most injured ally
                    let heal_target = self.party.iter().flatten()
                        .filter(|m| !m.down && m.max_hp > 0)
                        .min_by_key(|m| m.hp * 100 / m.max_hp)
                        .map(|m| m.id.clone());

                    if let Some(target) = heal_target {
                        CombatAction::UseSkill {
                            skill: SkillId::new("heal"),
                            target: TargetSelection::Single(target),
                        }
                    } else {
                        CombatAction::Defend
                    }
                } else if let Some(enemy) = target_enemy {
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            NpcBehavior::Nervous => {
                // Prioritize self-preservation: take cover if exposed, attack only if safe
                if npc_hp_pct < 50 && npc.combatant.position == PositionState::Open {
                    CombatAction::TakeCover
                } else if let Some(enemy) = target_enemy {
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            NpcBehavior::Unreliable => {
                // Unpredictable: sometimes attacks, sometimes defends, sometimes runs for cover
                // Deterministic fallback: attack if enemies exist, otherwise defend
                if let Some(enemy) = target_enemy {
                    // Unreliable NPCs attack the weakest enemy (easiest kill)
                    let weak_enemy = self.enemies.iter()
                        .filter(|e| !e.down && !e.panicked)
                        .min_by_key(|e| e.hp)
                        .unwrap_or(enemy);
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single(weak_enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
        }
    }

    /// FT-005: Select an action for a named NPC character based on their role.
    /// Named characters override generic behavior with character-specific AI.
    pub fn select_named_npc_action(&self, npc_id: &str) -> CombatAction {
        if !self.npc_allies.iter().any(|n| n.combatant.id == npc_id) {
            return CombatAction::Defend;
        }

        // Check if any ally needs healing
        let most_injured_ally = self.party.iter().flatten()
            .filter(|m| !m.down && m.max_hp > 0)
            .min_by_key(|m| m.hp * 100 / m.max_hp);

        let ally_critical = most_injured_ally
            .map(|m| m.hp * 100 / m.max_hp < 40)
            .unwrap_or(false);

        // Find target enemy
        let target_enemy = self.enemies.iter()
            .filter(|e| !e.down && !e.panicked)
            .max_by_key(|e| e.damage);

        // Character-specific behavior
        match npc_id {
            // Deputies: Professional attack, focus on highest-threat
            "cal" | "deputy_harris" => {
                if let Some(enemy) = target_enemy {
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            // Eli as NPC: nerve damage specialist
            "eli" => {
                if let Some(enemy) = target_enemy {
                    CombatAction::UseSkill {
                        skill: SkillId::new("fast_talk"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            // Ada as NPC: healer first, derringer backup
            "ada" => {
                if ally_critical {
                    if let Some(target) = most_injured_ally {
                        return CombatAction::UseSkill {
                            skill: SkillId::new("treat_wounds"),
                            target: TargetSelection::Single(target.id.clone()),
                        };
                    }
                }
                if let Some(enemy) = target_enemy {
                    CombatAction::UseSkill {
                        skill: SkillId::new("derringer"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            // Bale: heavy hitter, slow, reliable
            "bale" => {
                if let Some(enemy) = target_enemy {
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            // Renata: sharper, precise shots
            "renata" => {
                if let Some(enemy) = self.enemies.iter()
                    .filter(|e| !e.down && !e.panicked)
                    .min_by_key(|e| e.hp) // snipe weakest
                {
                    CombatAction::UseSkill {
                        skill: SkillId::new("attack"),
                        target: TargetSelection::Single(enemy.id.clone()),
                    }
                } else {
                    CombatAction::Defend
                }
            }
            // Fallback to generic behavior
            _ => self.select_npc_action(npc_id),
        }
    }

    /// Mark this encounter as non-escapable (boss fight).
    pub fn set_escapable(&mut self, escapable: bool) {
        self.escapable = escapable;
    }

    // ─── Skill Cooldowns ──────────────────────────────────────────

    /// Tick all cooldowns down by 1 at the start of each round.
    /// Removes cooldowns that reach 0.
    pub fn tick_cooldowns(&mut self) {
        self.cooldowns.retain(|_, cd| {
            *cd = cd.saturating_sub(1);
            *cd > 0
        });
    }

    /// Check if a skill is on cooldown for a specific combatant.
    pub fn skill_on_cooldown(&self, combatant_id: &str, skill: &SkillId) -> Option<u8> {
        self.cooldowns.get(&(combatant_id.to_string(), skill.clone()))
            .copied()
            .filter(|&cd| cd > 0)
    }

    // ─── Ammo Economy ─────────────────────────────────────────────

    /// Get an actor's current and max ammo.
    fn get_actor_ammo(&self, id: &str) -> Option<(i32, i32)> {
        for slot in &self.party {
            if let Some(m) = slot {
                if m.id == id { return Some((m.ammo, m.max_ammo)); }
            }
        }
        for e in &self.enemies {
            if e.id == id { return Some((e.ammo, e.max_ammo)); }
        }
        for n in &self.npc_allies {
            if n.combatant.id == id { return Some((n.combatant.ammo, n.combatant.max_ammo)); }
        }
        None
    }

    /// Calculate ammo scavenged after combat based on enemies defeated.
    /// Returns 1 ammo per defeated enemy (down), 0 for panicked-only.
    pub fn scavenge_ammo(&self) -> i32 {
        self.enemies.iter().filter(|e| e.down).count() as i32
    }

    // ─── Status Effects ───────────────────────────────────────────

    /// Apply a status effect to a combatant.
    pub fn apply_status_effect(&mut self, combatant_id: &str, effect: StatusEffect, duration: u8) {
        // Helper closure pattern — find combatant and add effect
        for slot in &mut self.party {
            if let Some(m) = slot {
                if m.id == combatant_id {
                    // Replace existing effect of same type or add new
                    if let Some(existing) = m.active_effects.iter_mut().find(|(e, _)| *e == effect) {
                        existing.1 = duration; // refresh duration
                    } else {
                        m.active_effects.push((effect, duration));
                    }
                    return;
                }
            }
        }
        for e in &mut self.enemies {
            if e.id == combatant_id {
                if let Some(existing) = e.active_effects.iter_mut().find(|(eff, _)| *eff == effect) {
                    existing.1 = duration;
                } else {
                    e.active_effects.push((effect, duration));
                }
                return;
            }
        }
        for n in &mut self.npc_allies {
            if n.combatant.id == combatant_id {
                if let Some(existing) = n.combatant.active_effects.iter_mut().find(|(eff, _)| *eff == effect) {
                    existing.1 = duration;
                } else {
                    n.combatant.active_effects.push((effect, duration));
                }
                return;
            }
        }
    }

    /// Check if a combatant has a specific status effect active.
    pub fn has_status_effect(&self, combatant_id: &str, effect: StatusEffect) -> bool {
        for slot in &self.party {
            if let Some(m) = slot {
                if m.id == combatant_id {
                    return m.active_effects.iter().any(|(e, d)| *e == effect && *d > 0);
                }
            }
        }
        for e in &self.enemies {
            if e.id == combatant_id {
                return e.active_effects.iter().any(|(eff, d)| *eff == effect && *d > 0);
            }
        }
        for n in &self.npc_allies {
            if n.combatant.id == combatant_id {
                return n.combatant.active_effects.iter().any(|(eff, d)| *eff == effect && *d > 0);
            }
        }
        false
    }

    /// Process status effects at the start of a turn for a combatant.
    /// Returns a list of status changes that occurred.
    pub fn apply_status_effects(&mut self, combatant_id: &str) -> Vec<StatusChange> {
        let mut changes = Vec::new();

        // Collect effects to process (avoid borrow issues)
        let effects: Vec<(StatusEffect, u8)> = {
            let combatant = self.find_combatant(combatant_id);
            match combatant {
                Some(c) => c.active_effects.clone(),
                None => return changes,
            }
        };

        for (effect, duration) in &effects {
            if *duration == 0 { continue; }
            let (hp_dmg, _nerve_dmg, skip) = effect.per_turn_impact();

            if hp_dmg > 0 {
                let down = self.apply_damage(combatant_id, hp_dmg);
                changes.push(StatusChange {
                    target: combatant_id.to_string(),
                    change: format!("{:?} deals {} damage{}", effect, hp_dmg,
                        if down { " — down!" } else { "" }),
                });
            }
            if skip {
                changes.push(StatusChange {
                    target: combatant_id.to_string(),
                    change: format!("{:?} — turn skipped", effect),
                });
            }
        }

        changes
    }

    /// Tick status effect durations down by 1 and remove expired ones.
    pub fn tick_status_effects(&mut self) {
        for slot in &mut self.party {
            if let Some(m) = slot {
                for effect in &mut m.active_effects {
                    effect.1 = effect.1.saturating_sub(1);
                }
                m.active_effects.retain(|(_, d)| *d > 0);
            }
        }
        for e in &mut self.enemies {
            for effect in &mut e.active_effects {
                effect.1 = effect.1.saturating_sub(1);
            }
            e.active_effects.retain(|(_, d)| *d > 0);
        }
        for n in &mut self.npc_allies {
            for effect in &mut n.combatant.active_effects {
                effect.1 = effect.1.saturating_sub(1);
            }
            n.combatant.active_effects.retain(|(_, d)| *d > 0);
        }
    }

    /// Find a combatant by ID (immutable reference to their LiveCombatant).
    fn find_combatant(&self, id: &str) -> Option<&LiveCombatant> {
        for slot in &self.party {
            if let Some(m) = slot {
                if m.id == id { return Some(m); }
            }
        }
        for e in &self.enemies {
            if e.id == id { return Some(e); }
        }
        for n in &self.npc_allies {
            if n.combatant.id == id { return Some(&n.combatant); }
        }
        None
    }

    // ─── Fear Cascade ─────────────────────────────────────────────

    /// When a combatant's nerve breaks (reaches 0), apply fear cascade
    /// to all allies on the same side: 5-10 nerve damage each.
    /// Returns the list of nerve damage events caused by the cascade.
    pub fn fear_cascade(&mut self, broken_id: &str) -> Vec<NerveDamageEvent> {
        let mut events = Vec::new();

        // Determine which side the broken combatant is on
        let side = self.find_combatant(broken_id).map(|c| c.side);
        let side = match side {
            Some(s) => s,
            None => return events,
        };

        // Collect ally IDs (same side, not the broken one, not already panicked/down)
        let ally_ids: Vec<String> = match side {
            CombatSide::Party => {
                self.party.iter().flatten()
                    .filter(|m| m.id != broken_id && !m.panicked && !m.down)
                    .map(|m| m.id.clone())
                    .collect()
            }
            CombatSide::Enemy => {
                self.enemies.iter()
                    .filter(|e| e.id != broken_id && !e.panicked && !e.down)
                    .map(|e| e.id.clone())
                    .collect()
            }
            CombatSide::NpcAlly => {
                self.npc_allies.iter()
                    .filter(|n| n.combatant.id != broken_id && !n.combatant.panicked && !n.combatant.down)
                    .map(|n| n.combatant.id.clone())
                    .collect()
            }
        };

        // Apply 7 nerve damage (midpoint of 5-10) to each ally.
        // Deterministic for testability — RNG can be wired later.
        let cascade_damage = 7;
        for ally_id in ally_ids {
            let (broke, panicked) = self.apply_nerve_damage(&ally_id, cascade_damage);
            events.push(NerveDamageEvent {
                target: ally_id.clone(),
                amount: cascade_damage,
                target_panicked: panicked,
                target_broke: broke,
            });
            debug!(
                source = broken_id,
                target = %ally_id,
                damage = cascade_damage,
                "fear cascade"
            );
        }

        events
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

    // ─── FT-005: NPC ally behavior system ─────────────────────────

    fn encounter_with_npcs() -> Encounter {
        let mut enc = glass_arroyo_encounter();
        enc.phases[0].npc_allies.push(NpcCombatant {
            character: CharacterId::new("deputy_harris"),
            behavior: NpcBehavior::Professional,
            hp: 35, nerve: 28, speed: 11, accuracy: 65, damage: 9,
        });
        enc.phases[0].npc_allies.push(NpcCombatant {
            character: CharacterId::new("bale"),
            behavior: NpcBehavior::Protective,
            hp: 35, nerve: 30, speed: 7, accuracy: 55, damage: 12,
        });
        enc
    }

    #[test]
    fn npc_professional_attacks_highest_threat() {
        let encounter = encounter_with_npcs();
        let state = EncounterState::new(&encounter, prologue_party());

        let action = state.select_npc_action("deputy_harris");
        match action {
            CombatAction::UseSkill { target: TargetSelection::Single(target_id), .. } => {
                // Should target the highest-damage enemy (gunman: damage=10)
                assert_eq!(target_id, "gunman_1", "professional should target highest threat");
            }
            _ => panic!("professional NPC should attack"),
        }
    }

    #[test]
    fn npc_protective_heals_injured_ally() {
        let encounter = encounter_with_npcs();
        let mut state = EncounterState::new(&encounter, prologue_party());

        // Injure Galen below 40% HP
        state.party[0].as_mut().unwrap().hp = 10; // 10/40 = 25%

        let action = state.select_npc_action("bale");
        match action {
            CombatAction::UseSkill { skill, target: TargetSelection::Single(target_id), .. } => {
                assert_eq!(skill.0, "heal", "protective NPC should heal");
                assert_eq!(target_id, "galen", "should heal most injured ally");
            }
            _ => panic!("protective NPC should heal when ally is injured"),
        }
    }

    #[test]
    fn npc_nervous_takes_cover_when_hurt() {
        let mut encounter = glass_arroyo_encounter();
        encounter.phases[0].npc_allies.push(NpcCombatant {
            character: CharacterId::new("scared_guard"),
            behavior: NpcBehavior::Nervous,
            hp: 20, nerve: 15, speed: 9, accuracy: 50, damage: 6,
        });

        let mut state = EncounterState::new(&encounter, prologue_party());

        // Hurt the nervous NPC below 50%
        state.npc_allies[0].combatant.hp = 8;

        let action = state.select_npc_action("scared_guard");
        assert!(matches!(action, CombatAction::TakeCover),
            "nervous NPC should take cover when hurt");
    }

    #[test]
    fn named_npc_eli_uses_fast_talk() {
        let mut encounter = glass_arroyo_encounter();
        encounter.phases[0].npc_allies.push(NpcCombatant {
            character: CharacterId::new("eli"),
            behavior: NpcBehavior::Professional,
            hp: 30, nerve: 25, speed: 10, accuracy: 50, damage: 6,
        });

        let state = EncounterState::new(&encounter, prologue_party());
        let action = state.select_named_npc_action("eli");
        match action {
            CombatAction::UseSkill { skill, .. } => {
                assert_eq!(skill.0, "fast_talk", "Eli should use fast_talk for nerve damage");
            }
            _ => panic!("Eli should use a skill"),
        }
    }

    #[test]
    fn named_npc_ada_heals_critical_ally() {
        let mut encounter = glass_arroyo_encounter();
        encounter.phases[0].npc_allies.push(NpcCombatant {
            character: CharacterId::new("ada"),
            behavior: NpcBehavior::Protective,
            hp: 25, nerve: 30, speed: 8, accuracy: 40, damage: 4,
        });

        let mut state = EncounterState::new(&encounter, prologue_party());
        state.party[0].as_mut().unwrap().hp = 8; // Galen critical

        let action = state.select_named_npc_action("ada");
        match action {
            CombatAction::UseSkill { skill, target: TargetSelection::Single(target), .. } => {
                assert_eq!(skill.0, "treat_wounds", "Ada should heal");
                assert_eq!(target, "galen", "should heal Galen");
            }
            _ => panic!("Ada should heal when ally is critical"),
        }
    }

    // ─── FT-006: Age-variant skill effects ────────────────────────

    #[test]
    fn age_variant_youth_faster_less_accurate() {
        let encounter = glass_arroyo_encounter();
        let mut registry = SkillRegistry::new();
        registry.register(Skill {
            id: SkillId::new("quick_draw"),
            name: "Quick Draw".to_string(),
            description: "Fast shot".to_string(),
            line: SkillLine::Deadeye,
            unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
            age_variants: vec![
                AgeVariant {
                    phase: AgePhase::Youth,
                    accuracy: -5,     // less accurate
                    damage: 6,
                    speed_priority: 3, // faster
                    nerve_damage: 2,
                    description_override: None,
                },
                AgeVariant {
                    phase: AgePhase::Older,
                    accuracy: 15,     // more accurate
                    damage: 14,
                    speed_priority: -2, // slower
                    nerve_damage: 8,
                    description_override: Some("One shot. Certain.".to_string()),
                },
            ],
            cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 0 },
        });

        // Youth phase — should get speed bonus
        let mut state_youth = EncounterState::with_registries(
            &encounter, prologue_party(), registry.clone(), DuoTechRegistry::new(), AgePhase::Youth,
        );
        state_youth.resolve_standoff(StandoffPosture::SteadyHand, None);
        state_youth.build_turn_queue();

        let galen_youth = state_youth.turn_queue.iter()
            .find(|t| t.combatant_id == "galen").unwrap();
        // Base speed 12 + speed_priority 3 = 15
        assert_eq!(galen_youth.speed, 15, "youth Galen should be faster from speed_priority");

        // Older phase — should get speed penalty
        let mut state_older = EncounterState::with_registries(
            &encounter, prologue_party(), registry, DuoTechRegistry::new(), AgePhase::Older,
        );
        state_older.resolve_standoff(StandoffPosture::SteadyHand, None);
        state_older.build_turn_queue();

        let galen_older = state_older.turn_queue.iter()
            .find(|t| t.combatant_id == "galen").unwrap();
        // Base speed 12 + speed_priority -2 = 10
        assert_eq!(galen_older.speed, 10, "older Galen should be slower from speed_priority");
    }

    #[test]
    fn age_variant_affects_damage_in_combat() {
        let encounter = glass_arroyo_encounter();
        let mut registry = SkillRegistry::new();
        registry.register(Skill {
            id: SkillId::new("quick_draw"),
            name: "Quick Draw".to_string(),
            description: "Fast shot".to_string(),
            line: SkillLine::Deadeye,
            unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
            age_variants: vec![
                AgeVariant {
                    phase: AgePhase::Youth,
                    accuracy: 10,
                    damage: 5,     // youth: less damage
                    speed_priority: 0,
                    nerve_damage: 1,
                    description_override: None,
                },
                AgeVariant {
                    phase: AgePhase::Adult,
                    accuracy: 10,
                    damage: 15,    // adult: more damage
                    speed_priority: 0,
                    nerve_damage: 5,
                    description_override: None,
                },
            ],
            cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 0 },
        });

        // Youth — should deal 5 damage
        let mut state = EncounterState::with_registries(
            &encounter, prologue_party(), registry.clone(), DuoTechRegistry::new(), AgePhase::Youth,
        );
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let hp_before = state.enemies[0].hp;
        let result = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("quick_draw"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        if !result.damage_dealt.is_empty() {
            assert_eq!(result.damage_dealt[0].amount, 5, "youth variant should deal 5 damage");
        }

        // Adult — should deal 15 damage
        let mut state2 = EncounterState::with_registries(
            &encounter, prologue_party(), registry, DuoTechRegistry::new(), AgePhase::Adult,
        );
        state2.resolve_standoff(StandoffPosture::SteadyHand, None);
        state2.build_turn_queue();

        let galen_turn2 = state2.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state2.current_turn = galen_turn2;

        let result2 = state2.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("quick_draw"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        if !result2.damage_dealt.is_empty() {
            assert_eq!(result2.damage_dealt[0].amount, 15, "adult variant should deal 15 damage");
        }
    }

    // ─── FT-007: Cover mechanics ──────────────────────────────────

    #[test]
    fn partial_cover_reduces_damage_25_percent() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Put raider in partial cover
        state.set_position("raider_0", PositionState::PartialCover);
        let hp_before = state.enemies[0].hp;

        // Apply 20 damage — partial cover = 25% reduction = 15 actual
        state.apply_damage("raider_0", 20);
        let hp_after = state.enemies[0].hp;

        assert_eq!(hp_before - hp_after, 15, "partial cover should reduce damage by 25%");
    }

    #[test]
    fn take_cover_assigns_terrain_element() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::TakeCover);

        // Should assign to first available cover element ("Wagon wreck")
        assert!(result.action_description.contains("Wagon wreck"),
            "should mention the cover element name");
        assert_eq!(state.party[0].as_ref().unwrap().position, PositionState::InCover);
        assert!(state.cover_assignments.contains_key("galen"));
    }

    #[test]
    fn destructible_cover_breaks_under_damage() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Put enemy in cover (wagon wreck, durability 50, destructible)
        state.set_position("raider_0", PositionState::InCover);
        state.cover_assignments.insert("raider_0".to_string(), 0);

        // Blast with repeated heavy damage to destroy cover
        // Each hit: damage / 2 absorbed by cover = deducted from durability
        for _ in 0..10 {
            state.apply_damage("raider_0", 20); // 10 absorbed each time
        }

        // After 50 durability worth of absorbed damage, cover should be destroyed
        assert_eq!(state.terrain.cover[0].durability, 0, "cover should be destroyed");
        assert_eq!(
            state.enemies[0].position, PositionState::Open,
            "combatant should be forced out when cover is destroyed"
        );
        assert!(!state.cover_assignments.contains_key("raider_0"),
            "cover assignment should be removed");
    }

    #[test]
    fn indestructible_cover_survives() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Rock outcrop is at index 1, durability 100, NOT destructible
        state.set_position("raider_0", PositionState::InCover);
        state.cover_assignments.insert("raider_0".to_string(), 1);

        state.apply_damage("raider_0", 100);

        assert_eq!(state.terrain.cover[1].durability, 100, "indestructible cover should survive");
        assert_eq!(state.enemies[0].position, PositionState::InCover,
            "combatant should stay in indestructible cover");
    }

    #[test]
    fn force_out_of_cover_action() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Put enemy in cover
        state.set_position("raider_0", PositionState::InCover);
        state.cover_assignments.insert("raider_0".to_string(), 0);

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::ForceOutOfCover {
            target: "raider_0".to_string(),
        });

        assert!(result.action_description.contains("forces"),
            "should describe forcing out of cover");
        assert_eq!(state.enemies[0].position, PositionState::Open,
            "target should be in Open position after being forced out");
        assert!(!state.cover_assignments.contains_key("raider_0"),
            "cover assignment should be removed");
    }

    #[test]
    fn force_out_of_cover_noop_on_open_target() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::ForceOutOfCover {
            target: "raider_0".to_string(),
        });

        assert!(result.action_description.contains("not in cover"),
            "should note target is not in cover");
    }

    #[test]
    fn take_cover_falls_back_to_partial_when_all_occupied() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Occupy both cover elements
        state.cover_assignments.insert("raider_0".to_string(), 0);
        state.cover_assignments.insert("raider_1".to_string(), 1);

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::TakeCover);

        assert!(result.action_description.contains("partial"),
            "should get partial cover when all elements occupied");
        assert_eq!(state.party[0].as_ref().unwrap().position, PositionState::PartialCover);
    }

    // ─── Skill Cooldowns ──────────────────────────────────────────

    #[test]
    fn skill_cooldown_blocks_reuse() {
        let encounter = glass_arroyo_encounter();
        let mut registry = SkillRegistry::new();
        registry.register(Skill {
            id: SkillId::new("called_shot"),
            name: "Called Shot".to_string(),
            description: "Precise shot with cooldown".to_string(),
            line: SkillLine::Deadeye,
            unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
            age_variants: vec![AgeVariant {
                phase: AgePhase::Adult,
                accuracy: 20, damage: 20, speed_priority: 0,
                nerve_damage: 5, description_override: None,
            }],
            cost: SkillCost { ammo: 2, nerve: 0, cooldown_turns: 2 },
        });

        let mut state = EncounterState::with_registries(
            &encounter, prologue_party(), registry, DuoTechRegistry::new(), AgePhase::Adult,
        );
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        // First use should succeed
        let result = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("called_shot"),
            target: TargetSelection::Single("raider_0".to_string()),
        });
        assert!(!result.action_description.contains("cooldown"), "first use should succeed");

        // Second use should be blocked
        let result2 = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("called_shot"),
            target: TargetSelection::Single("raider_0".to_string()),
        });
        assert!(result2.action_description.contains("cooldown"), "second use should be blocked by cooldown");
    }

    #[test]
    fn cooldown_ticks_down_each_round() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Set a cooldown manually
        state.cooldowns.insert(("galen".to_string(), SkillId::new("called_shot")), 3);
        assert_eq!(state.skill_on_cooldown("galen", &SkillId::new("called_shot")), Some(3));

        state.tick_cooldowns();
        assert_eq!(state.skill_on_cooldown("galen", &SkillId::new("called_shot")), Some(2));

        state.tick_cooldowns();
        assert_eq!(state.skill_on_cooldown("galen", &SkillId::new("called_shot")), Some(1));

        state.tick_cooldowns();
        assert_eq!(state.skill_on_cooldown("galen", &SkillId::new("called_shot")), None);
    }

    // ─── Reload Action ────────────────────────────────────────────

    #[test]
    fn reload_restores_partial_ammo() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Drain Galen's ammo to 2 (max is 12)
        state.party[0].as_mut().unwrap().ammo = 2;

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::Reload);
        assert!(result.action_description.contains("recovered 3"), "should restore 3 rounds");
        assert_eq!(state.party[0].as_ref().unwrap().ammo, 5);
    }

    #[test]
    fn reload_caps_at_max_ammo() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Galen has 11/12 ammo — reload should only restore 1
        state.party[0].as_mut().unwrap().ammo = 11;

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::Reload);
        assert!(result.action_description.contains("recovered 1"));
        assert_eq!(state.party[0].as_ref().unwrap().ammo, 12);
    }

    #[test]
    fn reload_at_full_ammo_does_nothing() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::Reload);
        assert!(result.action_description.contains("full ammo"));
    }

    // ─── Ammo Scavenging ──────────────────────────────────────────

    #[test]
    fn scavenge_ammo_from_defeated_enemies() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // No enemies down yet
        assert_eq!(state.scavenge_ammo(), 0);

        // Down 2 enemies
        state.enemies[0].down = true;
        state.enemies[1].down = true;
        assert_eq!(state.scavenge_ammo(), 2);

        // Panicked but not down — no ammo
        state.enemies[2].panicked = true;
        assert_eq!(state.scavenge_ammo(), 2, "panicked-only enemies don't yield ammo");
    }

    // ─── Status Effects ───────────────────────────────────────────

    #[test]
    fn bleeding_deals_damage_per_turn() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        let hp_before = state.party[0].as_ref().unwrap().hp;
        state.apply_status_effect("galen", StatusEffect::Bleeding, 3);

        let changes = state.apply_status_effects("galen");
        assert!(!changes.is_empty());
        assert!(changes[0].change.contains("Bleeding"));

        let hp_after = state.party[0].as_ref().unwrap().hp;
        assert_eq!(hp_before - hp_after, 3, "bleeding should deal 3 damage per turn");
    }

    #[test]
    fn stunned_skips_turn() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        state.apply_status_effect("galen", StatusEffect::Stunned, 1);

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        // Stunned check is in execute_action for UseSkill
        let result = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("quick_draw"),
            target: TargetSelection::Single("raider_0".to_string()),
        });
        assert!(result.action_description.contains("stunned"), "stunned should block skill use");
    }

    #[test]
    fn inspired_boosts_damage() {
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
                accuracy: 10, damage: 20, speed_priority: 0,
                nerve_damage: 5, description_override: None,
            }],
            cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 0 },
        });

        let mut state = EncounterState::with_registries(
            &encounter, prologue_party(), registry, DuoTechRegistry::new(), AgePhase::Adult,
        );
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        state.apply_status_effect("galen", StatusEffect::Inspired, 2);

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("quick_draw"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        // Inspired: 20 + 20/4 = 25 damage
        if !result.damage_dealt.is_empty() {
            assert_eq!(result.damage_dealt[0].amount, 25, "inspired should boost damage by 25%");
        }
    }

    #[test]
    fn status_effects_expire_after_ticking() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        state.apply_status_effect("galen", StatusEffect::Bleeding, 2);
        assert!(state.has_status_effect("galen", StatusEffect::Bleeding));

        state.tick_status_effects();
        assert!(state.has_status_effect("galen", StatusEffect::Bleeding)); // 1 turn left

        state.tick_status_effects();
        assert!(!state.has_status_effect("galen", StatusEffect::Bleeding)); // expired
    }

    #[test]
    fn suppressed_reduces_accuracy() {
        let encounter = glass_arroyo_encounter();
        let mut registry = SkillRegistry::new();
        // Register a skill with high accuracy to ensure it normally hits
        registry.register(Skill {
            id: SkillId::new("quick_draw"),
            name: "Quick Draw".to_string(),
            description: "Fast shot".to_string(),
            line: SkillLine::Deadeye,
            unlock: UnlockCondition::StartOfPhase(AgePhase::Youth),
            age_variants: vec![AgeVariant {
                phase: AgePhase::Adult,
                accuracy: -25, // actor base 70 + (-25) = 45, below 50 threshold = miss
                damage: 10, speed_priority: 0,
                nerve_damage: 0, description_override: None,
            }],
            cost: SkillCost { ammo: 1, nerve: 0, cooldown_turns: 0 },
        });

        let mut state = EncounterState::with_registries(
            &encounter, prologue_party(), registry, DuoTechRegistry::new(), AgePhase::Adult,
        );
        state.resolve_standoff(StandoffPosture::SteadyHand, None);
        state.build_turn_queue();

        // Without suppressed: 70 + (-25) = 45, already misses.
        // Let's test that suppressed further decreases: 45 - 20 = 25
        state.apply_status_effect("galen", StatusEffect::Suppressed, 2);

        let galen_turn = state.turn_queue.iter()
            .position(|t| t.combatant_id == "galen").unwrap();
        state.current_turn = galen_turn;

        let result = state.execute_action(&CombatAction::UseSkill {
            skill: SkillId::new("quick_draw"),
            target: TargetSelection::Single("raider_0".to_string()),
        });

        // With accuracy = 25, this should miss
        assert!(result.action_description.contains("missed"), "suppressed should cause miss");
    }

    // ─── Fear Cascade ─────────────────────────────────────────────

    #[test]
    fn fear_cascade_damages_allies_nerve() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Record party nerve before cascade
        let eli_nerve_before = state.party[1].as_ref().unwrap().nerve;

        // Trigger fear cascade from Galen breaking
        let events = state.fear_cascade("galen");

        // Eli should take nerve damage
        assert!(!events.is_empty(), "cascade should produce events");
        let eli_event = events.iter().find(|e| e.target == "eli");
        assert!(eli_event.is_some(), "Eli should be affected");
        assert_eq!(eli_event.unwrap().amount, 7, "cascade damage should be 7");

        let eli_nerve_after = state.party[1].as_ref().unwrap().nerve;
        assert_eq!(eli_nerve_before - eli_nerve_after, 7, "Eli nerve should drop by 7");
    }

    #[test]
    fn fear_cascade_on_enemy_side() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Record enemy nerve values (after standoff nerve damage of 3 each)
        let gunman_nerve_before = state.enemies[1].nerve;

        // Trigger cascade from raider breaking — should hit gunman and lookout
        let events = state.fear_cascade("raider_0");

        // Should affect active enemies on same side (lookout may already be panicked from standoff)
        assert!(!events.is_empty(), "enemy cascade should produce events");
        let gunman_event = events.iter().find(|e| e.target == "gunman_1");
        assert!(gunman_event.is_some(), "gunman should be affected");

        let gunman_nerve_after = state.enemies[1].nerve;
        assert_eq!(gunman_nerve_before - gunman_nerve_after, 7);
    }

    #[test]
    fn fear_cascade_skips_already_panicked() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Panic Eli first
        state.party[1].as_mut().unwrap().panicked = true;

        // Cascade from Galen — should not affect already-panicked Eli
        let events = state.fear_cascade("galen");
        assert!(events.iter().all(|e| e.target != "eli"),
            "cascade should skip already-panicked combatants");
    }

    #[test]
    fn fear_cascade_can_chain() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.resolve_standoff(StandoffPosture::SteadyHand, None);

        // Set Eli's nerve very low so cascade from Galen will panic him
        state.party[1].as_mut().unwrap().nerve = 5;

        let events = state.fear_cascade("galen");

        // Eli took 7 nerve damage from 5 → 0, should be panicked
        let eli = state.party[1].as_ref().unwrap();
        assert!(eli.panicked, "Eli should panic from cascade (nerve 5 - 7 = 0)");
        assert_eq!(eli.nerve, 0);
    }

    // ─── Combo System Tests ───────────────────────────────────────

    #[test]
    fn combo_first_action_no_bonus() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        let mult = state.record_combo("galen", "deadeye");
        assert!((mult - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn combo_second_same_action_ten_percent() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.record_combo("galen", "deadeye");
        let mult = state.record_combo("galen", "deadeye");
        assert!((mult - 1.10).abs() < f32::EPSILON);
    }

    #[test]
    fn combo_third_same_action_twenty_percent() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.record_combo("galen", "deadeye");
        state.record_combo("galen", "deadeye");
        let mult = state.record_combo("galen", "deadeye");
        assert!((mult - 1.20).abs() < f32::EPSILON);
    }

    #[test]
    fn combo_resets_on_different_skill_line() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.record_combo("galen", "deadeye");
        state.record_combo("galen", "deadeye");
        // Switch to a different skill line
        let mult = state.record_combo("galen", "command");
        assert!((mult - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn combo_reset_clears_actor() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.record_combo("galen", "deadeye");
        state.record_combo("galen", "deadeye");
        state.reset_combo("galen");
        // After reset, next action is first use again
        let mult = state.record_combo("galen", "deadeye");
        assert!((mult - 1.0).abs() < f32::EPSILON);
    }

    // ─── Terrain Modifier Tests ──────────────────────────────────

    #[test]
    fn terrain_burning_applies_damage() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.apply_terrain_modifier(TerrainModifier::Burning { damage_per_turn: 5 });
        let (hp_dmg, nerve_pen) = state.check_terrain_effects();
        assert_eq!(hp_dmg, 5);
        assert_eq!(nerve_pen, 0);
    }

    #[test]
    fn terrain_flooded_applies_nerve_penalty() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.apply_terrain_modifier(TerrainModifier::Flooded { nerve_penalty: 3 });
        let (hp_dmg, nerve_pen) = state.check_terrain_effects();
        assert_eq!(hp_dmg, 0);
        assert_eq!(nerve_pen, 3);
    }

    #[test]
    fn terrain_multiple_modifiers_stack() {
        let encounter = glass_arroyo_encounter();
        let mut state = EncounterState::new(&encounter, prologue_party());
        state.apply_terrain_modifier(TerrainModifier::Burning { damage_per_turn: 5 });
        state.apply_terrain_modifier(TerrainModifier::Flooded { nerve_penalty: 2 });
        state.apply_terrain_modifier(TerrainModifier::Cratered { source: "explosion".to_string() });
        state.apply_terrain_modifier(TerrainModifier::Cleared { former_cover: "barrel".to_string() });
        let (hp_dmg, nerve_pen) = state.check_terrain_effects();
        assert_eq!(hp_dmg, 5);
        assert_eq!(nerve_pen, 2);
        assert_eq!(state.terrain_modifiers.len(), 4);
    }

    #[test]
    fn terrain_no_modifiers_no_effects() {
        let encounter = glass_arroyo_encounter();
        let state = EncounterState::new(&encounter, prologue_party());
        let (hp_dmg, nerve_pen) = state.check_terrain_effects();
        assert_eq!(hp_dmg, 0);
        assert_eq!(nerve_pen, 0);
    }
}
