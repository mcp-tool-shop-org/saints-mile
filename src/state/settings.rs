//! Game settings — player-configurable preferences persisted to settings.ron.
//!
//! Separate from save data: settings apply across all save slots.

use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Settings file name.
const SETTINGS_FILE: &str = "settings.ron";

/// Minimum text speed multiplier.
pub const TEXT_SPEED_MIN: f32 = 0.5;
/// Maximum text speed multiplier.
pub const TEXT_SPEED_MAX: f32 = 2.0;
/// Default text speed multiplier.
pub const TEXT_SPEED_DEFAULT: f32 = 1.0;

/// Player-configurable game settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Text reveal speed multiplier. Higher = faster text.
    /// Range: 0.5 (slow) to 2.0 (fast). Default: 1.0.
    pub text_speed_multiplier: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            text_speed_multiplier: TEXT_SPEED_DEFAULT,
        }
    }
}

impl GameSettings {
    /// Clamp text_speed_multiplier to valid range.
    pub fn validate(&mut self) {
        self.text_speed_multiplier = self.text_speed_multiplier.clamp(TEXT_SPEED_MIN, TEXT_SPEED_MAX);
    }

    /// Load settings from the save directory. Returns defaults if file missing or corrupt.
    pub fn load(save_dir: &Path) -> Self {
        let path = save_dir.join(SETTINGS_FILE);
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                match ron::from_str::<GameSettings>(&contents) {
                    Ok(mut settings) => {
                        settings.validate();
                        settings
                    }
                    Err(_) => Self::default(),
                }
            }
            Err(_) => Self::default(),
        }
    }

    /// Save settings to the save directory.
    pub fn save(&self, save_dir: &Path) -> Result<PathBuf> {
        std::fs::create_dir_all(save_dir)
            .with_context(|| format!("failed to create save directory: {}", save_dir.display()))?;

        let path = save_dir.join(SETTINGS_FILE);
        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .context("failed to serialize settings")?;

        std::fs::write(&path, &serialized)
            .with_context(|| format!("failed to write settings: {}", path.display()))?;

        Ok(path)
    }

    /// Get the effective text reveal rate in milliseconds, applying the multiplier.
    /// A higher multiplier means faster text (lower ms per character).
    pub fn apply_text_speed(&self, base_ms: u64) -> u64 {
        if base_ms == 0 {
            return 0; // Crisis pacing stays instant
        }
        let adjusted = base_ms as f64 / self.text_speed_multiplier as f64;
        (adjusted.round() as u64).max(1) // At least 1ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_settings() {
        let s = GameSettings::default();
        assert!((s.text_speed_multiplier - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn validate_clamps_low() {
        let mut s = GameSettings { text_speed_multiplier: 0.1 };
        s.validate();
        assert!((s.text_speed_multiplier - TEXT_SPEED_MIN).abs() < f32::EPSILON);
    }

    #[test]
    fn validate_clamps_high() {
        let mut s = GameSettings { text_speed_multiplier: 10.0 };
        s.validate();
        assert!((s.text_speed_multiplier - TEXT_SPEED_MAX).abs() < f32::EPSILON);
    }

    #[test]
    fn round_trip() {
        let dir = TempDir::new().unwrap();
        let settings = GameSettings { text_speed_multiplier: 1.5 };
        settings.save(dir.path()).unwrap();

        let loaded = GameSettings::load(dir.path());
        assert!((loaded.text_speed_multiplier - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn load_missing_returns_default() {
        let dir = TempDir::new().unwrap();
        let loaded = GameSettings::load(dir.path());
        assert!((loaded.text_speed_multiplier - TEXT_SPEED_DEFAULT).abs() < f32::EPSILON);
    }

    #[test]
    fn load_corrupt_returns_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        std::fs::write(&path, "this is not valid RON").unwrap();

        let loaded = GameSettings::load(dir.path());
        assert!((loaded.text_speed_multiplier - TEXT_SPEED_DEFAULT).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_text_speed_multiplier() {
        let s = GameSettings { text_speed_multiplier: 2.0 };
        assert_eq!(s.apply_text_speed(30), 15);

        let slow = GameSettings { text_speed_multiplier: 0.5 };
        assert_eq!(slow.apply_text_speed(30), 60);
    }

    #[test]
    fn apply_text_speed_crisis_stays_zero() {
        let s = GameSettings { text_speed_multiplier: 2.0 };
        assert_eq!(s.apply_text_speed(0), 0);
    }

    #[test]
    fn apply_text_speed_minimum_one() {
        let fast = GameSettings { text_speed_multiplier: 2.0 };
        // Even with very small base, result is at least 1ms
        assert!(fast.apply_text_speed(1) >= 1);
    }
}
