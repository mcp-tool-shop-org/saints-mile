//! State store — RON-backed persistence with versioned save envelope.
//!
//! Hard rule: never let load logic "repair" morally important ambiguity
//! into certainty. If a witness is compromised, partial, unverified, or
//! branch-dependent, the save/load layer preserves that exact mess.

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::types::GameState;
use crate::scene::types::StateEffect;

/// Current save format version. Bump when GameState shape changes.
pub const SAVE_VERSION: u32 = 1;

/// Versioned save envelope — wraps GameState with metadata.
/// The envelope is what gets serialized to disk. GameState is the truth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveEnvelope {
    /// Schema version — checked on load.
    pub version: u32,
    /// Human-readable label for the save slot.
    pub label: String,
    /// When this save was written (unix timestamp).
    pub timestamp: u64,
    /// The actual game state — the biography.
    pub state: GameState,
}

impl SaveEnvelope {
    /// Create a new save envelope from current game state.
    pub fn new(state: GameState, label: impl Into<String>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: SAVE_VERSION,
            label: label.into(),
            timestamp,
            state,
        }
    }
}

/// The state store — owns the live GameState and handles persistence.
#[derive(Debug)]
pub struct StateStore {
    /// The live game state. This is the single source of truth.
    state: GameState,
    /// Save directory path.
    save_dir: PathBuf,
}

impl StateStore {
    /// Create a new store with a fresh game state.
    pub fn new_game(save_dir: impl Into<PathBuf>) -> Self {
        let state = GameState::new_game();
        info!(chapter = %state.chapter, beat = %state.beat, "new game started");
        Self {
            state,
            save_dir: save_dir.into(),
        }
    }

    /// Create a store from an existing state (for fixtures/quickstart).
    pub fn from_state(state: GameState, save_dir: impl Into<PathBuf>) -> Self {
        Self {
            state,
            save_dir: save_dir.into(),
        }
    }

    /// Load from a save file. Returns an error if the version is incompatible.
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read save file: {}", path.display()))?;

        let envelope: SaveEnvelope = ron::from_str(&contents)
            .with_context(|| "save file is corrupt or uses an unknown format")?;

        if envelope.version != SAVE_VERSION {
            anyhow::bail!(
                "save version mismatch: file is v{}, game expects v{}. \
                 Save migration is not yet supported.",
                envelope.version,
                SAVE_VERSION,
            );
        }

        let save_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();

        info!(
            version = envelope.version,
            label = %envelope.label,
            chapter = %envelope.state.chapter,
            beat = %envelope.state.beat,
            "save loaded"
        );

        Ok(Self {
            state: envelope.state,
            save_dir,
        })
    }

    /// Save current state to a file.
    pub fn save(&self, slot_name: &str) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.save_dir)
            .with_context(|| format!("failed to create save directory: {}", self.save_dir.display()))?;

        let label = format!(
            "{} — {} ({})",
            self.state.chapter, self.state.beat, self.state.age_phase_label()
        );

        let envelope = SaveEnvelope::new(self.state.clone(), label);

        let path = self.save_dir.join(format!("{}.ron", slot_name));
        let serialized = ron::ser::to_string_pretty(&envelope, ron::ser::PrettyConfig::default())
            .context("failed to serialize game state")?;

        std::fs::write(&path, &serialized)
            .with_context(|| format!("failed to write save file: {}", path.display()))?;

        info!(slot = slot_name, path = %path.display(), "game saved");
        Ok(path)
    }

    // --- State access ---

    /// Get immutable reference to the live state.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Get mutable reference to the live state.
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    // --- State mutation with audit trail ---

    /// Apply a single state effect with tracing.
    pub fn apply_effect(&mut self, effect: &StateEffect) {
        tracing::debug!(?effect, "applying state effect");
        self.state.apply_effect(effect);
    }

    /// Apply multiple state effects with tracing.
    pub fn apply_effects(&mut self, effects: &[StateEffect]) {
        for effect in effects {
            self.apply_effect(effect);
        }
    }

    /// Check a condition against current state.
    pub fn check(&self, condition: &crate::scene::types::Condition) -> bool {
        self.state.check_condition(condition)
    }

    /// Check all conditions — all must be true.
    pub fn check_all(&self, conditions: &[crate::scene::types::Condition]) -> bool {
        self.state.check_all(conditions)
    }
}

impl GameState {
    /// Human-readable age phase label for save descriptions.
    pub fn age_phase_label(&self) -> &'static str {
        match self.age_phase {
            crate::types::AgePhase::Youth => "Age 19",
            crate::types::AgePhase::YoungMan => "Age 24",
            crate::types::AgePhase::Adult => "Age 34",
            crate::types::AgePhase::Older => "Age 50+",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::scene::types::*;
    use crate::state::types::*;
    use tempfile::TempDir;

    /// Round-trip: new game → save → load → verify identical.
    #[test]
    fn round_trip_new_game() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::new_game(dir.path());

        // Save
        let path = store.save("test_slot").unwrap();
        assert!(path.exists());

        // Load
        let loaded = StateStore::load(&path).unwrap();

        // Verify critical fields survive
        assert_eq!(loaded.state().chapter.0, "prologue");
        assert_eq!(loaded.state().beat.0, "p1");
        assert_eq!(loaded.state().age_phase, AgePhase::Adult);
        assert_eq!(loaded.state().party.members.len(), 2);
        assert!(loaded.state().party.has_member(&CharacterId::new("galen")));
        assert!(loaded.state().party.has_member(&CharacterId::new("eli")));
        assert_eq!(loaded.state().prologue_choice, None);
        assert_eq!(loaded.state().relay_branch, None);
    }

    /// Version mismatch produces a clear error.
    #[test]
    fn version_mismatch_fails_cleanly() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::new_game(dir.path());
        let path = store.save("test_slot").unwrap();

        // Tamper with version
        let mut contents = std::fs::read_to_string(&path).unwrap();
        contents = contents.replace(
            &format!("version: {}", SAVE_VERSION),
            "version: 999",
        );
        std::fs::write(&path, &contents).unwrap();

        let result = StateStore::load(&path);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("version mismatch"), "error was: {}", err);
    }

    /// Eli's Loyalty line: grayed out but known. Skills not present, line exists in schema.
    #[test]
    fn eli_loyalty_line_grayed_out() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::new_game(dir.path());

        // Eli has no Loyalty skills unlocked
        let eli = store.state().party.members.iter()
            .find(|m| m.id.0 == "eli").unwrap();
        assert!(eli.unlocked_skills.is_empty());

        // But the SkillLine::Loyalty variant exists in the type system
        // (this is a compile-time guarantee, not a runtime check)
        let _loyalty = crate::combat::types::SkillLine::Loyalty;
    }

    /// Dead Drop persists once unlocked.
    #[test]
    fn dead_drop_persists() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Unlock Dead Drop on Galen
        let effect = StateEffect::UnlockSkill {
            character: CharacterId::new("galen"),
            skill: SkillId::new("dead_drop"),
        };
        store.apply_effect(&effect);

        // Verify it's there
        assert!(store.state().party.has_skill(
            &CharacterId::new("galen"),
            &SkillId::new("dead_drop"),
        ));

        // Save and reload
        let path = store.save("dead_drop_test").unwrap();
        let loaded = StateStore::load(&path).unwrap();
        assert!(loaded.state().party.has_skill(
            &CharacterId::new("galen"),
            &SkillId::new("dead_drop"),
        ));
    }

    /// Evidence integrity survives round-trip.
    #[test]
    fn evidence_integrity_survives() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Add evidence with partial integrity (scorched relay papers)
        store.state_mut().evidence.push(EvidenceItem {
            id: EvidenceId::new("relay_manifest"),
            evidence_type: EvidenceType::Documentary,
            source_chapter: ChapterId::new("ch2"),
            integrity: 45, // scorched, partial
            verified_against: vec![EvidenceId::new("archive_original")],
        });

        let path = store.save("evidence_test").unwrap();
        let loaded = StateStore::load(&path).unwrap();

        let evidence = loaded.state().evidence.iter()
            .find(|e| e.id.0 == "relay_manifest").unwrap();
        assert_eq!(evidence.integrity, 45);
        assert_eq!(evidence.verified_against.len(), 1);
        assert_eq!(evidence.verified_against[0].0, "archive_original");
    }

    /// Witness state survives branching outcomes.
    #[test]
    fn witness_state_branching() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Set witness states from the relay
        store.apply_effect(&StateEffect::SetWitnessState {
            id: WitnessId::new("tom_reed"),
            alive: true,
            integrity: 80,
        });
        store.apply_effect(&StateEffect::SetWitnessState {
            id: WitnessId::new("nella_creed"),
            alive: false,
            integrity: 0,
        });

        let path = store.save("witness_test").unwrap();
        let loaded = StateStore::load(&path).unwrap();

        let tom = loaded.state().witness_states.get("tom_reed").unwrap();
        assert!(tom.alive);
        assert_eq!(tom.integrity, 80);

        let nella = loaded.state().witness_states.get("nella_creed").unwrap();
        assert!(!nella.alive);
        assert_eq!(nella.integrity, 0);
    }

    /// Hand injury field persists.
    #[test]
    fn hand_injury_persists() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Damage Galen's hand
        if let Some(galen) = store.state_mut().party.members.iter_mut()
            .find(|m| m.id.0 == "galen")
        {
            galen.hand_state = HandState::Damaged;
        }

        let path = store.save("hand_test").unwrap();
        let loaded = StateStore::load(&path).unwrap();

        let galen = loaded.state().party.members.iter()
            .find(|m| m.id.0 == "galen").unwrap();
        assert_eq!(galen.hand_state, HandState::Damaged);
    }

    /// Memory objects survive and can echo.
    #[test]
    fn memory_objects_persist() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Add memory objects
        store.apply_effect(&StateEffect::AddMemoryObject(
            MemoryObjectId::new("wanted_poster"),
        ));
        store.apply_effect(&StateEffect::AddMemoryObject(
            MemoryObjectId::new("biscuit_cloth"),
        ));

        // Transform one
        store.apply_effect(&StateEffect::TransformMemoryObject {
            id: MemoryObjectId::new("biscuit_cloth"),
            new_state: "bloodstained".to_string(),
        });

        let path = store.save("memory_test").unwrap();
        let loaded = StateStore::load(&path).unwrap();

        assert_eq!(loaded.state().memory_objects.len(), 2);

        let poster = loaded.state().memory_objects.iter()
            .find(|o| o.id.0 == "wanted_poster").unwrap();
        assert_eq!(poster.state, "active");

        let cloth = loaded.state().memory_objects.iter()
            .find(|o| o.id.0 == "biscuit_cloth").unwrap();
        assert_eq!(cloth.state, "bloodstained");
    }

    /// Condition checking works correctly.
    #[test]
    fn condition_checking() {
        let mut state = GameState::new_game();

        // Set prologue choice
        state.prologue_choice = Some(PrologueChoice::HomesteadFirst);

        // Check prologue condition
        assert!(state.check_condition(&Condition::PrologueChoice(PrologueChoice::HomesteadFirst)));
        assert!(!state.check_condition(&Condition::PrologueChoice(PrologueChoice::TownDirect)));

        // Set a flag
        state.apply_effect(&StateEffect::SetFlag {
            id: FlagId::new("bitter_cut_pulled_punches"),
            value: FlagValue::Bool(true),
        });

        assert!(state.check_condition(&Condition::Flag {
            id: FlagId::new("bitter_cut_pulled_punches"),
            value: FlagValue::Bool(true),
        }));

        // Reputation check
        state.apply_effect(&StateEffect::AdjustReputation {
            axis: ReputationAxis::Rancher,
            delta: 15,
        });

        assert!(state.check_condition(&Condition::Reputation {
            axis: ReputationAxis::Rancher,
            op: CompareOp::Gte,
            threshold: 10,
        }));
        assert!(!state.check_condition(&Condition::Reputation {
            axis: ReputationAxis::Rancher,
            op: CompareOp::Gte,
            threshold: 20,
        }));
    }

    /// Relay branch as first-class state axis.
    #[test]
    fn relay_branch_first_class() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Set relay branch
        store.state_mut().relay_branch = Some(RelayBranch::Nella);
        store.state_mut().nella_alive = Some(true);
        store.state_mut().tom_alive = Some(false);

        // Check condition
        assert!(store.check(&Condition::RelayBranch(RelayBranch::Nella)));
        assert!(!store.check(&Condition::RelayBranch(RelayBranch::Tom)));

        // Persists
        let path = store.save("relay_test").unwrap();
        let loaded = StateStore::load(&path).unwrap();
        assert_eq!(loaded.state().relay_branch, Some(RelayBranch::Nella));
        assert_eq!(loaded.state().nella_alive, Some(true));
        assert_eq!(loaded.state().tom_alive, Some(false));
    }

    /// Effect application: complex multi-effect sequence.
    #[test]
    fn complex_effect_sequence() {
        let mut state = GameState::new_game();

        // Simulate the prologue's Beat 5 choice + consequences
        let effects = vec![
            StateEffect::SetFlag {
                id: FlagId::new("beat5_choice"),
                value: FlagValue::Text("homestead_first".to_string()),
            },
            StateEffect::AdjustReputation { axis: ReputationAxis::Rancher, delta: 10 },
            StateEffect::AdjustReputation { axis: ReputationAxis::TownLaw, delta: -5 },
            StateEffect::AddMemoryObject(MemoryObjectId::new("wanted_poster")),
            StateEffect::AdjustResource { resource: ResourceKind::Water, delta: -30 },
            StateEffect::AdjustResource { resource: ResourceKind::HorseStamina, delta: -40 },
            StateEffect::SetRelationship {
                a: CharacterId::new("galen"),
                b: CharacterId::new("eli"),
                value: 15,
            },
        ];

        state.apply_all(&effects);

        assert_eq!(state.reputation.get(ReputationAxis::Rancher), 10);
        assert_eq!(state.reputation.get(ReputationAxis::TownLaw), -5);
        assert_eq!(state.memory_objects.len(), 1);
        assert_eq!(state.resources.water, 70);
        assert_eq!(state.resources.horse_stamina, 60);
        assert_eq!(*state.party.relationships.get("galen:eli").unwrap(), 15);
    }
}
