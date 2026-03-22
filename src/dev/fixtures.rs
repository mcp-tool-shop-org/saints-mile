//! Save fixtures — pre-built save files for each major branch.
//!
//! Generates golden saves at key decision points so testers can
//! load directly into any branch without replaying.

use std::path::{Path, PathBuf};
use anyhow::Result;

use crate::types::*;
use crate::scene::types::StateEffect;
use crate::state::store::StateStore;
use crate::state::types::GameState;
use super::quickstart::JumpPoint;

/// Generate all fixture saves in a directory.
pub fn generate_fixtures(dir: &Path) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let mut paths = Vec::new();

    // Jump point saves
    for jp in JumpPoint::all() {
        let state = jp.create_state();
        let store = StateStore::from_state(state, dir);
        let slug = format!("fixture_{:?}", jp).to_lowercase().replace(' ', "_");
        let path = store.save(&slug)?;
        paths.push(path);
    }

    // Branch-specific saves at relay triage completion
    for (branch_name, branch_choice) in &[
        ("relay_tom", "tom"),
        ("relay_nella", "nella"),
        ("relay_papers", "papers"),
    ] {
        let mut state = JumpPoint::RelayTriage.create_state();
        state.flags.insert("relay_branch".to_string(), FlagValue::Text(branch_choice.to_string()));
        state.flags.insert("poster_born".to_string(), FlagValue::Bool(true));
        state.flags.insert("chapter2_complete".to_string(), FlagValue::Bool(true));

        // Branch-specific survival
        match *branch_choice {
            "tom" => {
                state.flags.insert("nella_died".to_string(), FlagValue::Bool(true));
            }
            "nella" => {
                state.flags.insert("tom_died".to_string(), FlagValue::Bool(true));
            }
            "papers" => {
                state.flags.insert("tom_died".to_string(), FlagValue::Bool(true));
                state.flags.insert("nella_died".to_string(), FlagValue::Bool(true));
            }
            _ => {}
        }

        // Dead Drop always unlocked at relay
        state.apply_effect(&StateEffect::UnlockSkill {
            character: CharacterId::new("galen"),
            skill: SkillId::new("dead_drop"),
        });

        let store = StateStore::from_state(state, dir);
        let path = store.save(branch_name)?;
        paths.push(path);
    }

    // Prologue branch saves (town direct vs homestead)
    for (name, choice) in &[
        ("prologue_town_direct", "town_direct"),
        ("prologue_homestead", "homestead_first"),
    ] {
        let mut state = JumpPoint::PrologueCampfire.create_state();
        state.flags.insert("beat5_choice".to_string(), FlagValue::Text(choice.to_string()));
        state.flags.insert("eli_confession".to_string(), FlagValue::Bool(true));

        match *choice {
            "town_direct" => {
                state.apply_effect(&StateEffect::AdjustReputation {
                    axis: ReputationAxis::TownLaw, delta: 5,
                });
                state.apply_effect(&StateEffect::AdjustReputation {
                    axis: ReputationAxis::Rancher, delta: -10,
                });
            }
            "homestead_first" => {
                state.apply_effect(&StateEffect::AdjustReputation {
                    axis: ReputationAxis::Rancher, delta: 10,
                });
                state.apply_effect(&StateEffect::AdjustReputation {
                    axis: ReputationAxis::TownLaw, delta: -10,
                });
            }
            _ => {}
        }

        let store = StateStore::from_state(state, dir);
        let path = store.save(name)?;
        paths.push(path);
    }

    Ok(paths)
}

/// Print a summary of what fixtures were generated.
pub fn print_fixture_summary(paths: &[PathBuf]) {
    println!("=== Generated {} fixture saves ===", paths.len());
    for path in paths {
        if let Some(name) = path.file_stem() {
            println!("  {}", name.to_string_lossy());
        }
    }
}
