//! Game state types — the complete memory of a playthrough.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::types::*;
use crate::scene::types::{Condition, StateEffect};
use super::investigation::InvestigationState;

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
    /// Active investigation state (e.g., Burned Mission multi-domain assembly).
    pub investigation: Option<InvestigationState>,
    /// Evidence IDs the player has collected — cross-referenced by relay branch gating.
    pub collected_evidence: Vec<String>,
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
            investigation: None,
            collected_evidence: Vec::new(),
        }
    }

    /// Collect a piece of evidence by ID (no-op if already collected).
    pub fn collect_evidence(&mut self, evidence_id: &str) {
        if !self.collected_evidence.contains(&evidence_id.to_string()) {
            self.collected_evidence.push(evidence_id.to_string());
        }
    }

    /// Check if a specific piece of evidence has been collected.
    pub fn has_collected(&self, evidence_id: &str) -> bool {
        self.collected_evidence.iter().any(|e| e == evidence_id)
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
                // Clone needed: key/value are moved into HashMap
                self.flags.insert(id.0.clone(), value.clone());
            }
            StateEffect::AdjustReputation { axis, delta } => {
                self.reputation.adjust(*axis, *delta);
            }
            StateEffect::AddPartyMember(char_id) => {
                // Clone needed: id is moved into PartyMemberState
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
                    // Clones needed: id and chapter are moved into the new EvidenceItem
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
                // Clone needed: key is moved into HashMap
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
                    // Clone needed: id is moved into the new MemoryObject
                    self.memory_objects.push(MemoryObject {
                        id: id.clone(),
                        state: "active".to_string(),
                    });
                }
            }
            StateEffect::TransformMemoryObject { id, new_state } => {
                if let Some(obj) = self.memory_objects.iter_mut().find(|o| o.id == *id) {
                    // Clone needed: new_state is moved into the object's field
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
                // Clone needed: injury is moved into the injuries Vec
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

// --- Relay branch evidence gating ---

/// Returns which evidence categories are accessible based on the relay branch.
///
/// - Tom branch: structural/route evidence (he knows the road, the layout).
/// - Nella branch: human witness/community evidence (she knows the people).
/// - Papers branch: documentary/filing evidence (the records survived).
///
/// Used by content/scene code to gate which evidence the player can discover.
pub fn relay_evidence_available(state: &GameState) -> Vec<&'static str> {
    match state.relay_branch {
        Some(RelayBranch::Tom) => vec![
            "structural",
            "route",
            "route_manifest_sm",
            "sheriff_security_ref",
            "medical_routing",
        ],
        Some(RelayBranch::Nella) => vec![
            "human_witness",
            "community",
            "payroll_ledger_convoy",
            "double_payroll",
            "medical_routing",
            "sheriff_security_ref",
        ],
        Some(RelayBranch::Papers) => vec![
            "documentary",
            "filing",
            "contract_demolition",
            "land_acquisition_chain",
            "double_payroll",
            "medical_routing",
            "sheriff_security_ref",
        ],
        None => vec![],
    }
}

// --- Party dispersal enforcement (Ch12+) ---

/// The canonical party member IDs and their departure/return flag names.
const DEPARTURE_FLAGS: &[(&str, &str, &str)] = &[
    // (character_id, departure_flag, return_flag)
    ("ada",    "ada_departed",    "ada_returned"),
    ("rosa",   "rosa_departed",   "rosa_returned"),
    ("eli",    "eli_departed",    "eli_returned"),
    ("miriam", "miriam_departed", "miriam_returned"),
    ("lucien", "lucien_departed", "lucien_returned"),
];

/// Returns the list of party member IDs who are currently available for combat
/// and party activities. Characters with `_departed` flags set to true are
/// excluded unless their `_returned` flag is also true (Ch14 reunions).
///
/// Galen is always available — he is the protagonist.
pub fn available_party_members(state: &GameState) -> Vec<CharacterId> {
    let mut available = Vec::new();

    for member in &state.party.members {
        // Galen is never removed from availability
        if member.id.0 == "galen" {
            available.push(member.id.clone());
            continue;
        }

        // Check departure/return flags for known party members
        let is_departed = DEPARTURE_FLAGS.iter().find(|(id, _, _)| *id == member.id.0);

        match is_departed {
            Some((_, depart_flag, return_flag)) => {
                let departed = state.flags.get(*depart_flag)
                    .map_or(false, |v| *v == FlagValue::Bool(true));
                let returned = state.flags.get(*return_flag)
                    .map_or(false, |v| *v == FlagValue::Bool(true));

                if !departed || returned {
                    available.push(member.id.clone());
                }
            }
            // Unknown party members (no departure tracking) are always available
            None => {
                available.push(member.id.clone());
            }
        }
    }

    available
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_evidence_deduplicates() {
        let mut state = GameState::new_game();
        state.collect_evidence("relay_manifest");
        state.collect_evidence("relay_manifest");
        assert_eq!(state.collected_evidence.len(), 1);
    }

    #[test]
    fn has_collected_works() {
        let mut state = GameState::new_game();
        assert!(!state.has_collected("relay_manifest"));
        state.collect_evidence("relay_manifest");
        assert!(state.has_collected("relay_manifest"));
    }

    #[test]
    fn relay_evidence_tom_branch() {
        let mut state = GameState::new_game();
        state.relay_branch = Some(RelayBranch::Tom);
        let available = relay_evidence_available(&state);
        assert!(available.contains(&"structural"));
        assert!(available.contains(&"route"));
        assert!(!available.contains(&"documentary"));
        assert!(!available.contains(&"human_witness"));
    }

    #[test]
    fn relay_evidence_nella_branch() {
        let mut state = GameState::new_game();
        state.relay_branch = Some(RelayBranch::Nella);
        let available = relay_evidence_available(&state);
        assert!(available.contains(&"human_witness"));
        assert!(available.contains(&"community"));
        assert!(!available.contains(&"structural"));
        assert!(!available.contains(&"filing"));
    }

    #[test]
    fn relay_evidence_papers_branch() {
        let mut state = GameState::new_game();
        state.relay_branch = Some(RelayBranch::Papers);
        let available = relay_evidence_available(&state);
        assert!(available.contains(&"documentary"));
        assert!(available.contains(&"filing"));
        assert!(!available.contains(&"structural"));
        assert!(!available.contains(&"human_witness"));
    }

    #[test]
    fn relay_evidence_none_returns_empty() {
        let state = GameState::new_game(); // no relay branch set
        assert!(relay_evidence_available(&state).is_empty());
    }

    #[test]
    fn available_party_no_departures() {
        let mut state = GameState::new_game();
        // Add ada and rosa to party
        state.party.add_member(CharacterId::new("ada"));
        state.party.add_member(CharacterId::new("rosa"));
        let available = available_party_members(&state);
        // galen, eli (from new_game), ada, rosa
        assert_eq!(available.len(), 4);
    }

    #[test]
    fn departed_member_excluded() {
        let mut state = GameState::new_game();
        state.party.add_member(CharacterId::new("ada"));
        state.flags.insert("ada_departed".to_string(), FlagValue::Bool(true));
        let available = available_party_members(&state);
        assert!(!available.iter().any(|id| id.0 == "ada"));
        // galen and eli still present
        assert_eq!(available.len(), 2);
    }

    #[test]
    fn returned_member_included() {
        let mut state = GameState::new_game();
        state.party.add_member(CharacterId::new("ada"));
        state.flags.insert("ada_departed".to_string(), FlagValue::Bool(true));
        state.flags.insert("ada_returned".to_string(), FlagValue::Bool(true));
        let available = available_party_members(&state);
        assert!(available.iter().any(|id| id.0 == "ada"));
        assert_eq!(available.len(), 3);
    }

    #[test]
    fn galen_never_departing() {
        let state = GameState::new_game();
        // Even if someone mistakenly sets a departure flag for galen
        // he's not in the DEPARTURE_FLAGS list, so he stays available
        let available = available_party_members(&state);
        assert!(available.iter().any(|id| id.0 == "galen"));
    }

    #[test]
    fn multiple_departures_ch12() {
        let mut state = GameState::new_game();
        state.party.add_member(CharacterId::new("ada"));
        state.party.add_member(CharacterId::new("rosa"));
        state.party.add_member(CharacterId::new("miriam"));

        // Ch12: multiple departures
        state.flags.insert("ada_departed".to_string(), FlagValue::Bool(true));
        state.flags.insert("rosa_departed".to_string(), FlagValue::Bool(true));

        let available = available_party_members(&state);
        assert!(available.iter().any(|id| id.0 == "galen"));
        assert!(available.iter().any(|id| id.0 == "eli"));
        assert!(available.iter().any(|id| id.0 == "miriam"));
        assert!(!available.iter().any(|id| id.0 == "ada"));
        assert!(!available.iter().any(|id| id.0 == "rosa"));
        assert_eq!(available.len(), 3);
    }

    #[test]
    fn investigation_wired_to_state() {
        use super::super::investigation::burned_mission_investigation;
        let mut state = GameState::new_game();
        assert!(state.investigation.is_none());

        // Wire investigation into state
        state.investigation = Some(burned_mission_investigation());
        assert!(!state.investigation.as_ref().unwrap().convergence_reached);
        assert_eq!(state.investigation.as_ref().unwrap().fragments.len(), 6);

        // Read a fragment through state
        let inv = state.investigation.as_mut().unwrap();
        let result = inv.read_fragment("land_grants", &CharacterId::new("galen"));
        assert!(result.is_some());
        assert_eq!(inv.domains_read.len(), 1);
    }
}
