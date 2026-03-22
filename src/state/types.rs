//! Game state types — the complete memory of a playthrough.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::scene::types::{Condition, StateEffect};

/// The complete game state — serialized to RON for save/load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub chapter: ChapterId,
    pub beat: BeatId,
    pub age_phase: AgePhase,
    pub reputation: ReputationWeb,
    pub evidence: Vec<EvidenceItem>,
    pub witness_states: HashMap<String, WitnessState>,
    pub rumor_state: HashMap<String, RumorStrength>,
    pub party: PartyState,
    pub prologue_choice: Option<PrologueChoice>,
    pub relay_branch: Option<RelayBranch>,
    pub cask_alive: Option<bool>,
    pub nella_alive: Option<bool>,
    pub tom_alive: Option<bool>,
    pub flags: HashMap<String, FlagValue>,
    pub memory_objects: Vec<MemoryObject>,
    pub resources: TrailResources,
}

impl GameState {
    pub fn new_game() -> Self {
        Self {
            chapter: ChapterId::new("prologue"),
            beat: BeatId::new("p1"),
            age_phase: AgePhase::Adult,
            reputation: ReputationWeb::default(),
            evidence: Vec::new(),
            witness_states: HashMap::new(),
            rumor_state: HashMap::new(),
            party: PartyState::new_prologue(),
            prologue_choice: None,
            relay_branch: None,
            cask_alive: None,
            nella_alive: None,
            tom_alive: None,
            flags: HashMap::new(),
            memory_objects: Vec::new(),
            resources: TrailResources::default(),
        }
    }

    pub fn check_condition(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Flag { id, value } => {
                self.flags.get(&id.0).map_or(false, |v| v == value)
            }
            Condition::Reputation { axis, op, threshold } => {
                let val = self.reputation.get(*axis);
                match op {
                    CompareOp::Gt => val > *threshold,
                    CompareOp::Gte => val >= *threshold,
                    CompareOp::Lt => val < *threshold,
                    CompareOp::Lte => val <= *threshold,
                    CompareOp::Eq => val == *threshold,
                    CompareOp::Neq => val != *threshold,
                }
            }
            Condition::PartyMember { character, present } => {
                self.party.has_member(character) == *present
            }
            Condition::Witness { id, alive } => {
                self.witness_states.get(&id.0).map_or(false, |w| w.alive == *alive)
            }
            Condition::HasMemoryObject(obj_id) => {
                self.memory_objects.iter().any(|o| o.id == *obj_id)
            }
            Condition::RelayBranch(branch) => self.relay_branch == Some(*branch),
            Condition::PrologueChoice(choice) => self.prologue_choice == Some(*choice),
            Condition::HasEvidence(evidence_id) => {
                self.evidence.iter().any(|e| e.id == *evidence_id)
            }
            Condition::HasSkill { character, skill } => {
                self.party.has_skill(character, skill)
            }
        }
    }

    pub fn check_all(&self, conditions: &[Condition]) -> bool {
        conditions.iter().all(|c| self.check_condition(c))
    }

    pub fn apply_effect(&mut self, effect: &StateEffect) {
        match effect {
            StateEffect::SetFlag { id, value } => {
                self.flags.insert(id.0.clone(), value.clone());
            }
            StateEffect::AdjustReputation { axis, delta } => {
                self.reputation.adjust(*axis, *delta);
            }
            StateEffect::AddPartyMember(char_id) => {
                self.party.add_member(char_id.clone());
            }
            StateEffect::RemovePartyMember(char_id) => {
                self.party.remove_member(char_id);
            }
            StateEffect::UnlockSkill { character, skill } => {
                self.party.unlock_skill(character, skill);
            }
            StateEffect::AddEvidence(id) => {
                if !self.evidence.iter().any(|e| e.id == *id) {
                    self.evidence.push(EvidenceItem {
                        id: id.clone(),
                        evidence_type: EvidenceType::Documentary,
                        source_chapter: self.chapter.clone(),
                        integrity: 100,
                        verified_against: Vec::new(),
                    });
                }
            }
            StateEffect::SetWitnessState { id, alive, integrity } => {
                self.witness_states.insert(id.0.clone(), WitnessState {
                    alive: *alive,
                    integrity: *integrity,
                    relationship_to_galen: 0,
                    has_testified: false,
                    location: None,
                });
            }
            StateEffect::AddMemoryObject(id) => {
                if !self.memory_objects.iter().any(|o| o.id == *id) {
                    self.memory_objects.push(MemoryObject {
                        id: id.clone(),
                        state: "active".to_string(),
                    });
                }
            }
            StateEffect::TransformMemoryObject { id, new_state } => {
                if let Some(obj) = self.memory_objects.iter_mut().find(|o| o.id == *id) {
                    obj.state = new_state.clone();
                }
            }
            StateEffect::AdjustResource { resource, delta } => {
                self.resources.adjust(*resource, *delta);
            }
            StateEffect::SetRelationship { a, b, value } => {
                self.party.set_relationship(a, b, *value);
            }
            StateEffect::ApplyInjury { character, injury } => {
                self.party.apply_injury(character, injury.clone());
            }
        }
    }

    pub fn apply_all(&mut self, effects: &[StateEffect]) {
        for effect in effects {
            self.apply_effect(effect);
        }
    }
}

/// Reputation web — multiple independent axes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReputationWeb {
    axes: HashMap<String, i32>,
}

impl ReputationWeb {
    pub fn get(&self, axis: ReputationAxis) -> i32 {
        let key = format!("{:?}", axis);
        *self.axes.get(&key).unwrap_or(&0)
    }

    pub fn adjust(&mut self, axis: ReputationAxis, delta: i32) {
        let key = format!("{:?}", axis);
        let val = self.axes.entry(key).or_insert(0);
        *val += delta;
    }
}

/// Party state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyState {
    pub members: Vec<PartyMemberState>,
    pub relationships: HashMap<String, i32>,
}

impl PartyState {
    pub fn new_prologue() -> Self {
        Self {
            members: vec![
                PartyMemberState::new("galen", "Galen Rook"),
                PartyMemberState::new("eli", "Eli Winter"),
            ],
            relationships: HashMap::new(),
        }
    }

    pub fn has_member(&self, id: &CharacterId) -> bool {
        self.members.iter().any(|m| m.id == *id)
    }

    pub fn add_member(&mut self, id: CharacterId) {
        if !self.has_member(&id) {
            self.members.push(PartyMemberState::new(&id.0, &id.0));
        }
    }

    pub fn remove_member(&mut self, id: &CharacterId) {
        self.members.retain(|m| m.id != *id);
    }

    pub fn has_skill(&self, character: &CharacterId, skill: &SkillId) -> bool {
        self.members.iter()
            .find(|m| m.id == *character)
            .map_or(false, |m| m.unlocked_skills.contains(skill))
    }

    pub fn unlock_skill(&mut self, character: &CharacterId, skill: &SkillId) {
        if let Some(member) = self.members.iter_mut().find(|m| m.id == *character) {
            if !member.unlocked_skills.contains(skill) {
                member.unlocked_skills.push(skill.clone());
            }
        }
    }

    pub fn set_relationship(&mut self, a: &CharacterId, b: &CharacterId, value: i32) {
        let key = format!("{}:{}", a.0, b.0);
        self.relationships.insert(key, value);
    }

    pub fn apply_injury(&mut self, character: &CharacterId, injury: InjuryId) {
        if let Some(member) = self.members.iter_mut().find(|m| m.id == *character) {
            member.injuries.push(injury);
        }
    }
}

/// State for a single party member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyMemberState {
    pub id: CharacterId,
    pub name: String,
    pub unlocked_skills: Vec<SkillId>,
    pub injuries: Vec<InjuryId>,
    pub hand_state: HandState,
}

impl PartyMemberState {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: CharacterId::new(id),
            name: name.to_string(),
            unlocked_skills: Vec::new(),
            injuries: Vec::new(),
            hand_state: HandState::Healthy,
        }
    }
}

/// Galen's hand state — first-class, not a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandState {
    Healthy,
    Damaged,
    Adapted,
}

/// A piece of evidence in the truth inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: EvidenceId,
    pub evidence_type: EvidenceType,
    pub source_chapter: ChapterId,
    pub integrity: i32,
    pub verified_against: Vec<EvidenceId>,
}

/// The five truth types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    Human,
    Structural,
    Documentary,
    Historical,
    System,
}

/// Witness state — tracked across the campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessState {
    pub alive: bool,
    pub integrity: i32,
    pub relationship_to_galen: i32,
    pub has_testified: bool,
    pub location: Option<LocationId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RumorStrength {
    Whisper,
    Talk,
    Known,
    Established,
}

/// A memory object — biscuit cloth, flask, poster, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryObject {
    pub id: MemoryObjectId,
    pub state: String,
}

/// Trail resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailResources {
    pub water: i32,
    pub ammo: i32,
    pub horse_stamina: i32,
}

impl Default for TrailResources {
    fn default() -> Self {
        Self { water: 100, ammo: 30, horse_stamina: 100 }
    }
}

impl TrailResources {
    pub fn adjust(&mut self, resource: ResourceKind, delta: i32) {
        match resource {
            ResourceKind::Water => self.water = (self.water + delta).max(0),
            ResourceKind::Ammo => self.ammo = (self.ammo + delta).max(0),
            ResourceKind::HorseStamina => self.horse_stamina = (self.horse_stamina + delta).max(0),
        }
    }
}
