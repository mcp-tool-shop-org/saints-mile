//! TUI presentation layer — spare, typographic, atmospheric.
//!
//! The command menu carries biography. Pacing drives rhythm.
//! Trust the player, stay legible.

pub mod theme;
pub mod text_reveal;
pub mod input;
pub mod widgets;
pub mod screens;

/// Re-export the App types for input.rs to use.
pub mod mod_types {
    pub use super::{App, AppScreen, InputResult};
}

use crate::combat::engine::{EncounterState, EncounterPhase, CombatSide, ActionResult};
use crate::combat::types::{Encounter, StandoffPosture, SkillLine};
use crate::scene::runner::{PreparedScene, SceneRunner};
use crate::scene::types::{Scene, SceneTransition};
use crate::state::store::StateStore;
use crate::state::types::MemoryObject;
use crate::types::AgePhase;

use self::text_reveal::TextReveal;
use self::screens::save_load::SaveLoadMode;
use self::screens::standoff::StandoffUi;
use self::screens::combat::{CombatUi, CombatMenuItem};

// ─── App State ────────────────────────────────────────────────────

/// The top-level application state.
pub struct App {
    pub screen: AppScreen,
    pub store: StateStore,
    pub should_quit: bool,

    // Scene UI state
    pub reveal: TextReveal,
    pub choice_cursor: usize,
    pub save_cursor: usize,
    pub current_prepared: Option<PreparedScene>,
    pub current_scene: Option<Scene>,

    // Combat UI state
    pub encounter_state: Option<EncounterState>,
    pub encounter_def: Option<Encounter>,
    pub standoff_ui: Option<StandoffUi>,
    pub combat_ui: CombatUi,
    pub combat_actions: Vec<CombatMenuItem>,
    /// Scene to return to after combat resolves.
    pub post_combat_scene: Option<String>,
}

/// Which screen the player is on.
pub enum AppScreen {
    Title,
    Scene {
        chapter_label: String,
        location_label: String,
    },
    Standoff,
    StandoffResult,
    Combat,
    CombatOutcome,
    SaveLoad {
        mode: SaveLoadMode,
    },
}

/// What an input event caused.
#[derive(Debug)]
pub enum InputResult {
    None,
    Redraw,
    Quit,
    NewGame,
    LoadScreen,
    BackToTitle,
    QuickSave,
    AdvanceScene,
    ConfirmChoice(usize),
    ConfirmSaveLoad(usize),
    // Standoff
    StandoffConfirm,
    StandoffCyclePosture(i32),
    StandoffCycleFocus(i32),
    // Combat
    CombatConfirmAction,
    CombatCycleAction(i32),
    CombatCycleTarget(i32),
    // Post-standoff / post-combat advance
    AdvanceCombat,
}

impl App {
    /// Create a new app at the title screen.
    pub fn new(save_dir: std::path::PathBuf) -> Self {
        Self {
            screen: AppScreen::Title,
            store: StateStore::new_game(&save_dir),
            should_quit: false,
            reveal: TextReveal::new(&[], crate::scene::types::PacingTag::Exploration),
            choice_cursor: 0,
            save_cursor: 0,
            current_prepared: None,
            current_scene: None,
            encounter_state: None,
            encounter_def: None,
            standoff_ui: None,
            combat_ui: CombatUi::new(),
            combat_actions: Vec::new(),
            post_combat_scene: None,
        }
    }

    /// Start a new game — load the first scene.
    pub fn new_game(&mut self) {
        self.store = StateStore::new_game(self.save_dir());
        self.load_scene("prologue_poster");
    }

    /// Load a scene by ID and transition to the scene screen.
    pub fn load_scene(&mut self, scene_id: &str) {
        if let Some(scene) = lookup_scene(scene_id) {
            let prepared = SceneRunner::prepare_scene(&scene, &self.store);
            SceneRunner::apply_scene_effects(&scene, &mut self.store);

            let line_lengths: Vec<usize> = prepared.lines.iter()
                .map(|l| l.text.len())
                .collect();

            self.reveal = TextReveal::new(&line_lengths, prepared.pacing);
            self.choice_cursor = 0;

            let chapter_label = format_chapter(&self.store.state().chapter.0);
            let location_label = format_location(&scene.location.0);

            self.current_prepared = Some(prepared);
            self.current_scene = Some(scene);
            self.screen = AppScreen::Scene {
                chapter_label,
                location_label,
            };
        }
    }

    /// Execute the selected choice and transition.
    pub fn execute_choice(&mut self, choice_index: usize) {
        let scene = self.current_scene.take();
        if let Some(scene) = scene {
            if let Some(prepared) = &self.current_prepared {
                if let Some(choice) = prepared.choices.get(choice_index) {
                    if !choice.available {
                        self.current_scene = Some(scene);
                        return;
                    }
                }
            }

            if let Some(chosen) = SceneRunner::execute_choice(&scene, choice_index, &mut self.store) {
                match chosen.transition {
                    SceneTransition::Scene(ref id) => {
                        self.load_scene(&id.0);
                    }
                    SceneTransition::Beat(ref id) => {
                        self.load_scene(&id.0);
                    }
                    SceneTransition::Combat(ref id) => {
                        self.enter_encounter(&id.0);
                    }
                    SceneTransition::End => {
                        self.screen = AppScreen::Title;
                    }
                }
            } else {
                self.current_scene = Some(scene);
            }
        }
    }

    // ─── Encounter Lifecycle ──────────────────────────────────────

    /// Enter an encounter by ID — set up standoff or combat.
    pub fn enter_encounter(&mut self, encounter_id: &str) {
        let encounter = lookup_encounter(encounter_id);
        if encounter.is_none() {
            // Fallback: skip to next scene if encounter not found
            eprintln!("[warn] encounter '{}' not found, falling back to post-combat scene", encounter_id);
            if let Some(next) = post_combat_scene(encounter_id) {
                self.load_scene(next);
            } else {
                eprintln!("[warn] no post-combat scene for '{}' either, returning to title", encounter_id);
                self.screen = AppScreen::Title;
            }
            return;
        }
        let encounter = encounter.unwrap();

        // Set the post-combat return scene
        self.post_combat_scene = post_combat_scene(encounter_id).map(|s| s.to_string());

        // Build party from current state
        let party_data = build_party_data(&self.store);
        let mut state = EncounterState::new(&encounter, party_data);

        // Assign state early so build_combat_actions_from_current can access it
        self.encounter_def = Some(encounter.clone());
        self.combat_ui = CombatUi::new();

        if encounter.standoff.is_some() {
            // Enter standoff phase
            let postures = encounter.standoff.as_ref().unwrap().postures.clone();
            let enemy_count = state.enemies.len();
            self.standoff_ui = Some(StandoffUi::new(postures, enemy_count));
            self.encounter_state = Some(state);
            self.screen = AppScreen::Standoff;
        } else {
            // Skip to combat
            state.build_turn_queue();
            self.encounter_state = Some(state);
            self.build_combat_actions_from_current();
            self.screen = AppScreen::Combat;
        }
    }

    /// Resolve the standoff with the player's chosen posture.
    pub fn resolve_standoff(&mut self) {
        let sui = match self.standoff_ui.as_ref() {
            Some(s) => s,
            None => return,
        };
        let posture = sui.selected_posture();
        let focus = if posture == StandoffPosture::Bait {
            let enemies = match self.encounter_state.as_ref() {
                Some(s) => &s.enemies,
                None => return,
            };
            let active: Vec<&str> = enemies.iter()
                .filter(|e| !e.down)
                .map(|e| e.id.as_str())
                .collect();
            active.get(sui.focus_cursor).map(|s| s.to_string())
        } else {
            None
        };

        let state = match self.encounter_state.as_mut() {
            Some(s) => s,
            None => return,
        };
        let _result = state.resolve_standoff(posture, focus.as_deref());

        // Store posture for result display
        self.combat_ui.standoff_posture = Some(posture);
        self.combat_ui.showing_standoff_result = true;
        self.screen = AppScreen::StandoffResult;
    }

    /// Transition from standoff result to combat.
    pub fn begin_combat(&mut self) {
        let state = match self.encounter_state.as_mut() {
            Some(s) => s,
            None => return,
        };
        state.build_turn_queue();
        self.combat_ui.showing_standoff_result = false;
        self.build_combat_actions_from_current();
        self.screen = AppScreen::Combat;
    }

    /// Execute the selected combat action.
    pub fn execute_combat_action(&mut self) {
        // Build the action from menu before borrowing encounter_state mutably
        let action = match self.build_action_from_menu() {
            Some(a) => a,
            None => return,
        };

        let state = match self.encounter_state.as_mut() {
            Some(s) => s,
            None => return,
        };
        let entry = state.current_turn_entry().cloned();

        if let Some(entry) = entry {
            if entry.side == CombatSide::Party {
                let result = state.execute_action(&action);
                self.combat_ui.push_result(&result);

                let Some(state) = self.encounter_state.as_mut() else { return; };
                state.evaluate_objectives();

                if let Some(outcome) = state.check_resolution() {
                    let effects = outcome.effects.clone();
                    self.store.apply_effects(&effects);
                    self.screen = AppScreen::CombatOutcome;
                    return;
                }

                if !state.advance_turn() {
                    state.build_turn_queue();
                }

                self.auto_execute_enemy_turns();
                self.build_combat_actions_from_current();
            }
        }
    }

    /// Auto-execute enemy and NPC turns until it's the player's turn again.
    fn auto_execute_enemy_turns(&mut self) {
        let state = match self.encounter_state.as_mut() {
            Some(s) => s,
            None => return,
        };

        loop {
            let entry = state.current_turn_entry().cloned();
            match entry {
                Some(entry) if entry.side != CombatSide::Party => {
                    // Enemy/NPC action — simple AI
                    let action = build_enemy_action(state, &entry);
                    let result = state.execute_action(&action);
                    self.combat_ui.push_result(&result);
                    state.evaluate_objectives();

                    if state.check_resolution().is_some() {
                        let outcome = state.outcome.as_ref().unwrap();
                        self.store.apply_effects(&outcome.effects);
                        self.screen = AppScreen::CombatOutcome;
                        return;
                    }

                    if !state.advance_turn() {
                        state.build_turn_queue();
                        // Check if the new round starts with an enemy too
                        continue;
                    }
                }
                _ => break, // Player's turn or no more entries
            }
        }
    }

    /// Build the action menu items for the current turn's party member.
    fn build_combat_actions_from_current(&mut self) {
        // Extract what we need without holding a borrow on self
        let data = self.encounter_state.as_ref().and_then(|state| {
            let entry = state.current_turn_entry()?;
            if entry.side != CombatSide::Party { return None; }
            let actor_id = entry.combatant_id.clone();
            let member = state.party.iter().flatten().find(|m| m.id == actor_id)?;
            let skills = member.skills.clone();
            let duo_techs = member.duo_techs.clone();
            let ammo = member.ammo;
            // Check duo partner availability
            let duo_available: Vec<bool> = duo_techs.iter()
                .map(|d| is_duo_partner_active(state, &actor_id, &d.0))
                .collect();
            Some((skills, duo_techs, ammo, duo_available))
        });

        self.combat_actions.clear();
        self.combat_ui.action_cursor = 0;
        self.combat_ui.target_cursor = 0;

        if let Some((skills, duo_techs, ammo, duo_available)) = data {
            for skill_id in &skills {
                let name = humanize_skill(&skill_id.0);
                let cost = skill_cost_text(&skill_id.0);
                let line = skill_line_label(&skill_id.0);
                let can_afford = ammo > 0 || !cost.contains("ammo");

                self.combat_actions.push(CombatMenuItem {
                    label: name,
                    cost_text: cost,
                    line_label: line,
                    available: can_afford,
                    lock_reason: if can_afford { None } else { Some("[No ammo]".to_string()) },
                });
            }

            for (i, duo_id) in duo_techs.iter().enumerate() {
                let name = humanize_skill(&duo_id.0);
                let partner_ok = duo_available.get(i).copied().unwrap_or(false);
                self.combat_actions.push(CombatMenuItem {
                    label: name,
                    cost_text: "duo".to_string(),
                    line_label: "Duo Tech".to_string(),
                    available: partner_ok,
                    lock_reason: if partner_ok { None } else { Some("[Partner absent]".to_string()) },
                });
            }

            self.combat_actions.push(CombatMenuItem {
                label: "Take Cover".to_string(),
                cost_text: "\u{2014}".to_string(),
                line_label: String::new(),
                available: true,
                lock_reason: None,
            });
            self.combat_actions.push(CombatMenuItem {
                label: "Defend".to_string(),
                cost_text: "\u{2014}".to_string(),
                line_label: String::new(),
                available: true,
                lock_reason: None,
            });
        }
    }

    /// Build a CombatAction from the current menu selection.
    fn build_action_from_menu(&self) -> Option<crate::combat::engine::CombatAction> {
        use crate::combat::engine::{CombatAction, TargetSelection};

        let action = self.combat_actions.get(self.combat_ui.action_cursor)?;
        if !action.available {
            return None;
        }

        let state = self.encounter_state.as_ref()?;

        // Find living enemy for targeting — bounds-safe via nth() returning None
        let living: Vec<_> = state.enemies.iter().filter(|e| !e.down).collect();
        let target_id = living.get(self.combat_ui.target_cursor)
            .or_else(|| living.first())
            .map(|e| e.id.clone());

        if action.label == "Take Cover" {
            return Some(CombatAction::TakeCover);
        }
        if action.label == "Defend" {
            return Some(CombatAction::Defend);
        }
        if action.line_label == "Duo Tech" {
            let duo_id = crate::types::DuoTechId::new(to_snake_case(&action.label));
            return Some(CombatAction::UseDuoTech {
                duo_tech: duo_id,
                target: TargetSelection::Single(target_id.unwrap_or_default()),
            });
        }

        // Skill action
        let skill_id = crate::types::SkillId::new(to_snake_case(&action.label));
        Some(CombatAction::UseSkill {
            skill: skill_id,
            target: TargetSelection::Single(target_id.unwrap_or_default()),
        })
    }

    /// After combat ends, return to scene flow.
    pub fn exit_combat(&mut self) {
        // Apply pending encounter effects to state
        if let Some(state) = &self.encounter_state {
            self.store.apply_effects(&state.pending_effects);
        }

        // Find the post-combat scene
        // Default: campfire_choice for prologue, or next scene in the chapter
        let next_scene = self.post_combat_scene.take()
            .unwrap_or_else(|| "campfire_choice".to_string());

        self.encounter_state = None;
        self.encounter_def = None;
        self.standoff_ui = None;

        self.load_scene(&next_scene);
    }

    /// Advance past a scene with no choices — chain to next chapter or end.
    pub fn advance_no_choice_scene(&mut self) {
        let scene = self.current_scene.take();
        if let Some(scene) = scene {
            let scene_id = scene.id.0.as_str();
            // Check if this is a chapter-end scene that chains to the next chapter
            if let Some(next) = next_chapter_scene(scene_id) {
                // Update age phase at chapter boundaries
                update_age_phase_for_chapter(next, &mut self.store);
                self.load_scene(next);
            } else {
                // Game complete or unlinked end scene
                self.screen = AppScreen::Title;
            }
        } else {
            self.screen = AppScreen::Title;
        }
    }

    /// Tick the text reveal forward.
    pub fn tick(&mut self) {
        if let Some(prepared) = &self.current_prepared {
            let line_lengths: Vec<usize> = prepared.lines.iter()
                .map(|l| l.text.len())
                .collect();
            self.reveal.tick(&line_lengths);
        }
    }

    pub fn choice_count(&self) -> usize {
        self.current_prepared.as_ref().map_or(0, |p| p.choices.len())
    }

    pub fn current_line_lengths(&self) -> Vec<usize> {
        self.current_prepared.as_ref().map_or(Vec::new(), |p| {
            p.lines.iter().map(|l| l.text.len()).collect()
        })
    }

    pub fn age_phase(&self) -> AgePhase {
        self.store.state().age_phase
    }

    pub fn memory_objects(&self) -> &[MemoryObject] {
        &self.store.state().memory_objects
    }

    /// Computed save directory — single source of truth for where saves live.
    pub fn save_dir(&self) -> std::path::PathBuf {
        dirs_next_or_default()
    }

    pub fn quick_save(&self) {
        let _ = self.store.save("quicksave");
    }

    /// Get the number of living enemies for target cycling.
    pub fn living_enemy_count(&self) -> usize {
        self.encounter_state.as_ref()
            .map(|s| s.enemies.iter().filter(|e| !e.down).count())
            .unwrap_or(0)
    }
}

// ─── Content Lookup ───────────────────────────────────────────────

fn lookup_scene(id: &str) -> Option<Scene> {
    use crate::content::*;
    prologue::get_scene(id)
        .or_else(|| cedar_wake::get_scene(id))
        .or_else(|| saints_mile_convoy::get_scene(id))
        .or_else(|| black_willow::get_scene(id))
        .or_else(|| ropehouse_blood::get_scene(id))
        .or_else(|| dust_revival::get_scene(id))
        .or_else(|| fuse_country::get_scene(id))
        .or_else(|| iron_ledger::get_scene(id))
        .or_else(|| burned_mission::get_scene(id))
        .or_else(|| long_wire::get_scene(id))
        .or_else(|| deadwater_trial::get_scene(id))
        .or_else(|| breakwater_junction::get_scene(id))
        .or_else(|| names_in_dust::get_scene(id))
        .or_else(|| fifteen_years_gone::get_scene(id))
        .or_else(|| old_friends::get_scene(id))
        .or_else(|| saints_mile_again::get_scene(id))
}

fn lookup_encounter(id: &str) -> Option<Encounter> {
    use crate::content::*;
    prologue::get_encounter(id)
        .or_else(|| cedar_wake::get_encounter(id))
        .or_else(|| saints_mile_convoy::get_encounter(id))
        .or_else(|| black_willow::get_encounter(id))
        .or_else(|| ropehouse_blood::get_encounter(id))
        .or_else(|| dust_revival::get_encounter(id))
        .or_else(|| fuse_country::get_encounter(id))
        .or_else(|| iron_ledger::get_encounter(id))
        .or_else(|| burned_mission::get_encounter(id))
        .or_else(|| breakwater_junction::get_encounter(id))
}

/// Build party combat data from the current game state.
///
/// Derives combat tuples from the live PartyState, using PartyTemplate base stats
/// for the current age phase and overlaying state-tracked skills and injuries.
/// Falls back to prologue_party() only if the state party is empty (new game).
fn build_party_data(store: &StateStore) -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<crate::types::SkillId>, Vec<crate::types::DuoTechId>, Vec<crate::combat::types::Wound>)> {
    use crate::combat::party_defs;
    use crate::combat::wounds;

    let game = store.state();
    if game.party.members.is_empty() {
        return crate::content::prologue::prologue_party();
    }

    let phase = game.age_phase;

    game.party.members.iter().map(|member| {
        // Look up the base template for this character + age phase
        let template = match member.id.0.as_str() {
            "galen" => Some(party_defs::galen(phase)),
            "eli" => Some(party_defs::eli_adult()),
            "ada" => Some(party_defs::ada()),
            "rosa" => Some(party_defs::rosa()),
            "miriam" => Some(party_defs::miriam()),
            "lucien" => Some(party_defs::lucien()),
            _ => None,
        };

        // Convert state-tracked injuries to combat Wound structs
        let combat_wounds: Vec<crate::combat::types::Wound> = member.injuries.iter()
            .filter_map(|inj| match inj.0.as_str() {
                "gunshot" => Some(wounds::gunshot_wound()),
                "blunt_trauma" => Some(wounds::blunt_trauma()),
                "exhaustion" => Some(wounds::exhaustion()),
                "nerve_shock" => Some(wounds::nerve_shock()),
                _ => None,
            })
            .collect();

        if let Some(tmpl) = template {
            // Use state-tracked skills if any have been unlocked, otherwise template defaults
            let skills = if member.unlocked_skills.is_empty() {
                tmpl.skills.clone()
            } else {
                member.unlocked_skills.clone()
            };

            (
                tmpl.id.to_string(),
                tmpl.name.to_string(),
                tmpl.hp, tmpl.nerve, tmpl.ammo,
                tmpl.speed, tmpl.accuracy, tmpl.damage,
                skills,
                tmpl.duo_techs.clone(),
                combat_wounds,
            )
        } else {
            // Unknown character — use member state with minimal defaults
            (
                member.id.0.clone(),
                member.name.clone(),
                20, 15, 6,   // fallback stats
                8, 40, 5,
                member.unlocked_skills.clone(),
                vec![],
                combat_wounds,
            )
        }
    }).collect()
}

/// Simple enemy AI: attack the first living party member.
fn build_enemy_action(state: &EncounterState, entry: &crate::combat::engine::TurnEntry) -> crate::combat::engine::CombatAction {
    use crate::combat::engine::{CombatAction, TargetSelection};

    let target = state.party.iter().flatten()
        .find(|p| !p.down)
        .map(|p| p.id.clone())
        .unwrap_or_default();

    CombatAction::UseSkill {
        skill: crate::types::SkillId::new("attack"),
        target: TargetSelection::Single(target),
    }
}

/// Check if a duo tech partner is active in the encounter.
fn is_duo_partner_active(state: &EncounterState, actor_id: &str, duo_id: &str) -> bool {
    // Loaded Deck requires both galen and eli
    match duo_id {
        "loaded_deck" => {
            let galen_up = state.party.iter().flatten().any(|m| m.id == "galen" && !m.down);
            let eli_up = state.party.iter().flatten().any(|m| m.id == "eli" && !m.down);
            galen_up && eli_up
        }
        _ => false,
    }
}

fn humanize_skill(id: &str) -> String {
    id.replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn skill_cost_text(id: &str) -> String {
    match id {
        "quick_draw" => "1 ammo".to_string(),
        "called_shot" | "snap_shot" => "1 ammo".to_string(),
        "take_cover" | "duck" | "sprint" => "\u{2014}".to_string(),
        "rally" => "3 nerve".to_string(),
        "setup_shot" => "1 ammo".to_string(),
        "overwatch" => "1 ammo".to_string(),
        "dead_drop" => "2 ammo, 5 nerve".to_string(),
        "sidearm" => "1 ammo".to_string(),
        "fast_talk" => "2 nerve".to_string(),
        "bluff" => "3 nerve".to_string(),
        "dirty_trick" => "1 nerve".to_string(),
        "patch_up" => "2 nerve".to_string(),
        "loaded_deck" => "2 ammo, 4 nerve".to_string(),
        _ => "1 ammo".to_string(),
    }
}

fn skill_line_label(id: &str) -> String {
    match id {
        "quick_draw" | "called_shot" | "snap_shot" | "steady_aim"
        | "dead_drop" | "overwatch" => "Deadeye".to_string(),
        "take_cover" | "trail_eye" | "cold_read" | "duck" | "sprint"
        | "pistol_whip" => "Trailcraft".to_string(),
        "rally" | "setup_shot" | "grit" | "suppressing_fire" => "Command".to_string(),
        "sidearm" | "fast_talk" | "quick_hands" | "read_the_room" => "Hustle".to_string(),
        "bluff" | "dirty_trick" | "double_down" => "Deceit".to_string(),
        "patch_up" => "\u{2014}".to_string(),
        _ => String::new(),
    }
}

fn to_snake_case(s: &str) -> String {
    s.to_lowercase().replace(' ', "_")
}

fn format_chapter(id: &str) -> String {
    match id {
        "prologue" => "Prologue".to_string(),
        "ch1" | "cedar_wake" => "Ch.1 \u{2014} Cedar Wake".to_string(),
        "ch2" | "saints_mile_convoy" => "Ch.2 \u{2014} Saint's Mile".to_string(),
        "ch3" | "black_willow" => "Ch.3 \u{2014} Black Willow".to_string(),
        "ch4" | "ropehouse_blood" => "Ch.4 \u{2014} Ropehouse Blood".to_string(),
        "ch5" | "dust_revival" => "Ch.5 \u{2014} Dust Revival".to_string(),
        "ch6" | "fuse_country" => "Ch.6 \u{2014} Fuse Country".to_string(),
        "ch7" | "iron_ledger" => "Ch.7 \u{2014} Iron Ledger".to_string(),
        "ch8" | "burned_mission" => "Ch.8 \u{2014} Burned Mission".to_string(),
        "ch9" | "long_wire" => "Ch.9 \u{2014} Long Wire".to_string(),
        "ch10" | "deadwater_trial" => "Ch.10 \u{2014} Deadwater Trial".to_string(),
        "ch11" | "breakwater_junction" => "Ch.11 \u{2014} Breakwater Junction".to_string(),
        "ch12" | "names_in_dust" => "Ch.12 \u{2014} Names in the Dust".to_string(),
        "ch13" | "fifteen_years_gone" => "Ch.13 \u{2014} Fifteen Years Gone".to_string(),
        "ch14" | "old_friends" => "Ch.14 \u{2014} Old Friends, Bad Ground".to_string(),
        "ch15" | "saints_mile_again" => "Ch.15 \u{2014} Saint's Mile Again".to_string(),
        other => other.replace('_', " "),
    }
}

fn format_location(id: &str) -> String {
    id.replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn dirs_next_or_default() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("saves")
}

// ─── Campaign Flow Tables ─────────────────────────────────────────

/// What scene follows after a combat encounter resolves.
/// Derived from the scene chain analysis of all 16 chapters.
fn post_combat_scene(encounter_id: &str) -> Option<&'static str> {
    match encounter_id {
        // Prologue
        "glass_arroyo" => Some("campfire_choice"),
        // Ch1 — Cedar Wake
        "horse_thief" => Some("cw_horse_thief_return"),
        "bandit_camp" => Some("cw_bandit_camp_return"),
        "bitter_cut" => Some("cw_bitter_cut_aftermath"),
        // Ch2 — Saint's Mile Convoy
        "red_switch_wash" => Some("night1_camp"),
        "hollow_pump" => Some("night2_camp"),
        "saints_mile_relay" => Some("relay_triage"),
        // Ch3 — Black Willow
        "pump_house_hold" => Some("bw_chapter_close"),
        // Ch4 — Ropehouse Blood
        "ropehouse_fight" => Some("rh_aftermath"),
        // Ch5 — Dust Revival
        "crowd_containment" => Some("dr_aftermath_intro"),
        "aftermath_guns" => Some("dr_chapter_close"),
        // Ch6 — Fuse Country
        "millburn_trestle" => Some("fc_lucien_decision"),
        // Ch7 — Iron Ledger
        "archive_break" => Some("il_archive_escape"),
        // Ch8 — Burned Mission
        "mission_defense" => Some("bm_chapter_close"),
        // Ch11 — Breakwater Junction
        "breakwater_battle" => Some("bj_hand_injury"),
        _ => None,
    }
}

/// What scene opens the next chapter after a chapter-end scene.
/// This is the seam that makes Saint's Mile feel like one life.
fn next_chapter_scene(end_scene_id: &str) -> Option<&'static str> {
    match end_scene_id {
        // Prologue ends → Ch1 Cedar Wake begins
        "return_town_direct" | "return_homestead" => Some("cw_arrival"),
        // Ch1 ends → Ch2 Convoy begins
        "cw_bitter_cut_return" => Some("convoy_join"),
        // Ch2 ends → Ch3 Black Willow begins (via Morrow aftermath)
        "relay_aftermath" => Some("bw_morrow_aftermath"),
        // Ch3 ends → Ch4 Ropehouse Blood
        "bw_chapter_close" => Some("rh_varela_approach"),
        // Ch4 ends → Ch5 Dust Revival
        "rh_chapter_close" => Some("dr_arrival"),
        // Ch5 ends → Ch6 Fuse Country
        "dr_chapter_close" => Some("fc_corridor_entry"),
        // Ch6 ends → Ch7 Iron Ledger
        "fc_chapter_close" => Some("il_city_entry"),
        // Ch7 ends → Ch8 Burned Mission
        "il_chapter_close" => Some("bm_valley_entry"),
        // Ch8 ends → Ch9 Long Wire
        "bm_chapter_close" => Some("lw_junction_arrival"),
        // Ch9 ends → Ch10 Deadwater Trial
        "lw_chapter_close" => Some("dw_arrival"),
        // Ch10 ends → Ch11 Breakwater Junction
        "dw_verdict" => Some("bj_entry"),
        // Ch11 ends → Ch12 Names in the Dust
        "bj_victory" => Some("nd_aftermath"),
        // Ch12 ends → Ch13 Fifteen Years Gone (time skip)
        "nd_last_road" => Some("fg_return"),
        // Ch13 ends → Ch14 Old Friends
        "fg_chapter_close" => Some("of_eli_return"),
        // Ch14 ends → Ch15 Saint's Mile Again
        "of_chapter_close" => Some("sm_return"),
        // Ch15 testament → game complete, return to title
        "sm_testament" => None,
        _ => None,
    }
}

/// Update the game state's age phase at chapter boundaries.
fn update_age_phase_for_chapter(next_scene: &str, store: &mut StateStore) {
    let new_phase = match next_scene {
        // Ch1 = Youth (age 19)
        "cw_arrival" => Some(AgePhase::Youth),
        // Ch2 = YoungMan (age 24)
        "convoy_join" => Some(AgePhase::YoungMan),
        // Ch3+ = Adult (age 34) — return from prologue flashback
        "bw_morrow_aftermath" => Some(AgePhase::Adult),
        // Ch13+ = Older (age ~50) — 15-year time skip
        "fg_return" => Some(AgePhase::Older),
        _ => None,
    };

    if let Some(phase) = new_phase {
        store.state_mut().age_phase = phase;
        // Also update chapter tracking
        let chapter = match next_scene {
            "cw_arrival" => "ch1",
            "convoy_join" => "ch2",
            "bw_morrow_aftermath" => "ch3",
            "rh_varela_approach" => "ch4",
            "dr_arrival" => "ch5",
            "fc_corridor_entry" => "ch6",
            "il_city_entry" => "ch7",
            "bm_valley_entry" => "ch8",
            "lw_junction_arrival" => "ch9",
            "dw_arrival" => "ch10",
            "bj_entry" => "ch11",
            "nd_aftermath" => "ch12",
            "fg_return" => "ch13",
            "of_eli_return" => "ch14",
            "sm_return" => "ch15",
            _ => return,
        };
        store.state_mut().chapter = crate::types::ChapterId::new(chapter);
    } else {
        // Still update chapter for non-phase-change boundaries
        let chapter = match next_scene {
            "rh_varela_approach" => Some("ch4"),
            "dr_arrival" => Some("ch5"),
            "fc_corridor_entry" => Some("ch6"),
            "il_city_entry" => Some("ch7"),
            "bm_valley_entry" => Some("ch8"),
            "lw_junction_arrival" => Some("ch9"),
            "dw_arrival" => Some("ch10"),
            "bj_entry" => Some("ch11"),
            "nd_aftermath" => Some("ch12"),
            "of_eli_return" => Some("ch14"),
            "sm_return" => Some("ch15"),
            _ => None,
        };
        if let Some(ch) = chapter {
            store.state_mut().chapter = crate::types::ChapterId::new(ch);
        }
    }
}
