//! Event log — records all player-facing decisions and state changes for analysis.
//!
//! Exports to RON or plain text for post-session review.
//! Captures: scene entries, choices made, skills used, combat outcomes,
//! relay branch, key flags, reputation changes.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::types::*;

/// A single logged event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Sequential event number.
    pub seq: u32,
    /// Chapter/beat context.
    pub chapter: String,
    pub beat: String,
    /// What happened.
    pub event: LogEvent,
}

/// The kinds of events we track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEvent {
    /// Player entered a scene.
    SceneEntered { scene_id: String, location: String },
    /// Player made a choice.
    ChoiceMade { scene_id: String, choice_label: String, choice_index: usize },
    /// A state flag was set.
    FlagSet { id: String, value: FlagValue },
    /// Reputation changed.
    ReputationChanged { axis: String, delta: i32, new_value: i32 },
    /// A skill was unlocked.
    SkillUnlocked { character: String, skill: String },
    /// Combat started.
    CombatStarted { encounter_id: String },
    /// Standoff posture chosen.
    StandoffChosen { posture: String, focus_target: Option<String> },
    /// Combat ended.
    CombatEnded { result: String, rounds: u32 },
    /// Relay branch chosen.
    RelayBranchChosen { branch: String },
    /// Memory object added or transformed.
    MemoryEvent { object_id: String, action: String },
    /// Game saved.
    GameSaved { slot: String, path: String },
    /// Debug note.
    Note(String),
}

/// The event log — accumulates entries during a session.
#[derive(Debug, Default)]
pub struct EventLog {
    entries: Vec<LogEntry>,
    next_seq: u32,
    current_chapter: String,
    current_beat: String,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current chapter/beat context.
    pub fn set_context(&mut self, chapter: &str, beat: &str) {
        self.current_chapter = chapter.to_string();
        self.current_beat = beat.to_string();
    }

    /// Log an event.
    pub fn log(&mut self, event: LogEvent) {
        self.entries.push(LogEntry {
            seq: self.next_seq,
            chapter: self.current_chapter.clone(),
            beat: self.current_beat.clone(),
            event,
        });
        self.next_seq += 1;
    }

    /// Convenience: log a scene entry.
    pub fn scene_entered(&mut self, scene_id: &str, location: &str) {
        self.log(LogEvent::SceneEntered {
            scene_id: scene_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Convenience: log a choice.
    pub fn choice_made(&mut self, scene_id: &str, label: &str, index: usize) {
        self.log(LogEvent::ChoiceMade {
            scene_id: scene_id.to_string(),
            choice_label: label.to_string(),
            choice_index: index,
        });
    }

    /// Convenience: log a flag set.
    pub fn flag_set(&mut self, id: &str, value: &FlagValue) {
        self.log(LogEvent::FlagSet {
            id: id.to_string(),
            value: value.clone(),
        });
    }

    /// Convenience: log a relay branch choice.
    pub fn relay_branch(&mut self, branch: &str) {
        self.log(LogEvent::RelayBranchChosen { branch: branch.to_string() });
    }

    /// Get all entries.
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Export to a plain-text summary.
    pub fn export_text(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Saint's Mile — Event Log ===\n\n");

        for entry in &self.entries {
            let line = match &entry.event {
                LogEvent::SceneEntered { scene_id, location } =>
                    format!("[{}] Scene: {} @ {}", entry.seq, scene_id, location),
                LogEvent::ChoiceMade { scene_id, choice_label, .. } =>
                    format!("[{}] Choice: \"{}\" in {}", entry.seq, choice_label, scene_id),
                LogEvent::FlagSet { id, value } =>
                    format!("[{}] Flag: {} = {:?}", entry.seq, id, value),
                LogEvent::ReputationChanged { axis, delta, new_value } =>
                    format!("[{}] Rep: {} {:+} → {}", entry.seq, axis, delta, new_value),
                LogEvent::SkillUnlocked { character, skill } =>
                    format!("[{}] Skill: {} unlocked {} ", entry.seq, character, skill),
                LogEvent::CombatStarted { encounter_id } =>
                    format!("[{}] Combat: {} started", entry.seq, encounter_id),
                LogEvent::StandoffChosen { posture, focus_target } =>
                    format!("[{}] Standoff: {} (focus: {:?})", entry.seq, posture, focus_target),
                LogEvent::CombatEnded { result, rounds } =>
                    format!("[{}] Combat: {} in {} rounds", entry.seq, result, rounds),
                LogEvent::RelayBranchChosen { branch } =>
                    format!("[{}] RELAY BRANCH: {}", entry.seq, branch),
                LogEvent::MemoryEvent { object_id, action } =>
                    format!("[{}] Memory: {} — {}", entry.seq, object_id, action),
                LogEvent::GameSaved { slot, .. } =>
                    format!("[{}] Saved: {}", entry.seq, slot),
                LogEvent::Note(text) =>
                    format!("[{}] Note: {}", entry.seq, text),
            };
            output.push_str(&format!("  {}/{} {}\n", entry.chapter, entry.beat, line));
        }

        // Summary section
        output.push_str("\n=== Summary ===\n");

        let choices: Vec<_> = self.entries.iter()
            .filter_map(|e| match &e.event {
                LogEvent::ChoiceMade { choice_label, .. } => Some(choice_label.as_str()),
                _ => None,
            })
            .collect();
        output.push_str(&format!("Choices made: {}\n", choices.len()));
        for c in &choices {
            output.push_str(&format!("  - {}\n", c));
        }

        let relay = self.entries.iter()
            .find_map(|e| match &e.event {
                LogEvent::RelayBranchChosen { branch } => Some(branch.as_str()),
                _ => None,
            });
        if let Some(branch) = relay {
            output.push_str(&format!("Relay branch: {}\n", branch));
        }

        output
    }

    /// Export to RON file.
    pub fn export_ron(&self, path: &Path) -> anyhow::Result<()> {
        let serialized = ron::ser::to_string_pretty(&self.entries, ron::ser::PrettyConfig::default())?;
        std::fs::write(path, serialized)?;
        Ok(())
    }
}
