//! Scene runner — executes scenes against live game state.
//!
//! Lines are stateful events, not just strings. Every line can check conditions,
//! fire effects, register memory callbacks, and tag pacing/emotion.
//! Choice visibility and choice consequence are separate concerns.

use tracing::{debug, info};

use super::types::*;
use crate::types::*;
use crate::state::store::StateStore;

/// The result of running a scene — what happened and where to go next.
#[derive(Debug)]
pub struct SceneResult {
    /// Lines that were displayed (after condition filtering).
    pub displayed_lines: Vec<DisplayedLine>,
    /// The choice the player made, if any.
    pub chosen: Option<ChosenAction>,
    /// Where the scene leads next.
    pub transition: SceneTransition,
    /// Memory callbacks triggered during this scene.
    pub memory_callbacks: Vec<MemoryRef>,
}

/// A line as displayed to the player — includes metadata for presentation.
#[derive(Debug, Clone)]
pub struct DisplayedLine {
    pub speaker: String,
    pub text: String,
    pub emotion: Option<EmotionTag>,
    pub pacing: PacingTag,
}

/// A choice as presented to the player — visibility and lock state are separate.
#[derive(Debug, Clone)]
pub struct PresentedChoice {
    /// Index into the scene's choice array.
    pub index: usize,
    /// Display label.
    pub label: String,
    /// Whether this choice can be selected.
    pub available: bool,
    /// Why it's locked, if locked.
    pub lock_reason: Option<String>,
}

/// What the player chose.
#[derive(Debug)]
pub struct ChosenAction {
    pub choice_index: usize,
    pub label: String,
    pub effects: Vec<StateEffect>,
    pub transition: SceneTransition,
}

/// The scene runner — steps through a scene against live state.
pub struct SceneRunner;

impl SceneRunner {
    /// Filter scene lines by conditions, returning only those that pass.
    pub fn filter_lines(scene: &Scene, store: &StateStore) -> Vec<DisplayedLine> {
        let mut displayed = Vec::new();

        for (i, line) in scene.lines.iter().enumerate() {
            // Line-level condition gating
            if !line.conditions.is_empty() && !store.check_all(&line.conditions) {
                debug!(
                    scene = %scene.id,
                    line_index = i,
                    speaker = %line.speaker,
                    "line skipped — conditions not met"
                );
                continue;
            }

            debug!(
                scene = %scene.id,
                line_index = i,
                speaker = %line.speaker,
                "line displayed"
            );

            displayed.push(DisplayedLine {
                speaker: line.speaker.0.clone(),
                text: line.text.clone(),
                emotion: line.emotion,
                pacing: scene.pacing,
            });
        }

        displayed
    }

    /// Evaluate which choices are available, visible, or locked.
    /// Visibility and consequence are separate: a choice can be visible but locked.
    pub fn evaluate_choices(scene: &Scene, store: &StateStore) -> Vec<PresentedChoice> {
        let mut presented = Vec::new();

        for (i, choice) in scene.choices.iter().enumerate() {
            let conditions_met = choice.conditions.is_empty()
                || store.check_all(&choice.conditions);

            // All choices are visible — locked ones show why they're locked.
            // This is a design choice: the player sees what they COULD do,
            // which makes what they CAN'T do meaningful.
            let lock_reason = if conditions_met {
                None
            } else {
                Some(Self::describe_lock(&choice.conditions, store))
            };

            debug!(
                scene = %scene.id,
                choice_index = i,
                label = %choice.label,
                available = conditions_met,
                "choice evaluated"
            );

            presented.push(PresentedChoice {
                index: i,
                label: choice.label.clone(),
                available: conditions_met,
                lock_reason,
            });
        }

        presented
    }

    /// Execute a chosen action — apply effects to state, return transition.
    pub fn execute_choice(
        scene: &Scene,
        choice_index: usize,
        store: &mut StateStore,
    ) -> Option<ChosenAction> {
        let choice = scene.choices.get(choice_index)?;

        // Apply effects
        info!(
            scene = %scene.id,
            choice = %choice.label,
            effect_count = choice.effects.len(),
            "choice executed"
        );

        store.apply_effects(&choice.effects);

        Some(ChosenAction {
            choice_index,
            label: choice.label.clone(),
            effects: choice.effects.clone(),
            transition: choice.next.clone(),
        })
    }

    /// Apply scene-level state effects (fired when the scene plays, regardless of choice).
    pub fn apply_scene_effects(scene: &Scene, store: &mut StateStore) {
        if !scene.state_effects.is_empty() {
            info!(
                scene = %scene.id,
                effect_count = scene.state_effects.len(),
                "scene effects applied"
            );
            store.apply_effects(&scene.state_effects);
        }
    }

    /// Collect memory callbacks from this scene.
    pub fn collect_memory_callbacks(scene: &Scene) -> Vec<MemoryRef> {
        if !scene.memory_refs.is_empty() {
            debug!(
                scene = %scene.id,
                callback_count = scene.memory_refs.len(),
                "memory callbacks collected"
            );
        }
        scene.memory_refs.clone()
    }

    /// Run a complete scene: filter lines, evaluate choices, apply scene effects,
    /// collect memory callbacks. Returns everything the presentation layer needs.
    ///
    /// The caller (TUI layer) is responsible for:
    /// 1. Displaying lines
    /// 2. Presenting choices to the player
    /// 3. Calling execute_choice with the player's selection
    pub fn prepare_scene(scene: &Scene, store: &StateStore) -> PreparedScene {
        // Check scene-level conditions
        if !scene.conditions.is_empty() && !store.check_all(&scene.conditions) {
            info!(
                scene = %scene.id,
                "scene skipped — conditions not met"
            );
            return PreparedScene {
                id: scene.id.clone(),
                lines: Vec::new(),
                choices: Vec::new(),
                memory_callbacks: Vec::new(),
                pacing: scene.pacing,
                should_play: false,
            };
        }

        let lines = Self::filter_lines(scene, store);
        let choices = Self::evaluate_choices(scene, store);
        let memory_callbacks = Self::collect_memory_callbacks(scene);

        info!(
            scene = %scene.id,
            location = %scene.location,
            lines = lines.len(),
            choices = choices.len(),
            pacing = ?scene.pacing,
            "scene prepared"
        );

        PreparedScene {
            id: scene.id.clone(),
            lines,
            choices,
            memory_callbacks,
            pacing: scene.pacing,
            should_play: true,
        }
    }

    /// Describe why a choice is locked — for UI display.
    fn describe_lock(conditions: &[Condition], store: &StateStore) -> String {
        for condition in conditions {
            if !store.state().check_condition(condition) {
                return match condition {
                    Condition::Flag { id, .. } =>
                        format!("[Requires: {}]", id),
                    Condition::Reputation { axis, op, threshold } =>
                        format!("[Requires: {:?} {:?} {}]", axis, op, threshold),
                    Condition::PartyMember { character, present: true } =>
                        format!("[Requires: {} in party]", character),
                    Condition::PartyMember { character, present: false } =>
                        format!("[Requires: {} absent]", character),
                    Condition::HasSkill { character, skill } =>
                        format!("[Requires: {} has {}]", character, skill),
                    Condition::HasEvidence(id) =>
                        format!("[Requires: evidence {}]", id),
                    Condition::HasMemoryObject(id) =>
                        format!("[Requires: {}]", id),
                    Condition::Witness { id, alive: true } =>
                        format!("[Requires: {} alive]", id),
                    Condition::Witness { id, alive: false } =>
                        format!("[Requires: {} dead]", id),
                    Condition::RelayBranch(branch) =>
                        format!("[Requires: {:?} branch]", branch),
                    Condition::PrologueChoice(choice) =>
                        format!("[Requires: {:?}]", choice),
                };
            }
        }
        "[Locked]".to_string()
    }
}

/// A scene prepared for presentation — everything the TUI needs.
#[derive(Debug)]
pub struct PreparedScene {
    pub id: SceneId,
    pub lines: Vec<DisplayedLine>,
    pub choices: Vec<PresentedChoice>,
    pub memory_callbacks: Vec<MemoryRef>,
    pub pacing: PacingTag,
    /// False if scene-level conditions weren't met.
    pub should_play: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    #[allow(unused_imports)]
    use crate::state::types::*;
    use tempfile::TempDir;

    /// Helper: build a minimal test scene.
    fn test_scene() -> Scene {
        Scene {
            id: SceneId::new("test_scene"),
            location: LocationId::new("test_location"),
            beat: BeatId::new("test_beat"),
            lines: vec![
                SceneLine {
                    speaker: SpeakerId::new("narrator"),
                    text: "Dust hangs gold in the light.".to_string(),
                    conditions: vec![],
                    emotion: Some(EmotionTag::Warm),
                },
                SceneLine {
                    speaker: SpeakerId::new("galen"),
                    text: "Long day.".to_string(),
                    conditions: vec![],
                    emotion: Some(EmotionTag::Quiet),
                },
                // Conditional line — only shows if prologue choice was homestead
                SceneLine {
                    speaker: SpeakerId::new("eli"),
                    text: "You took the long way. People noticed.".to_string(),
                    conditions: vec![
                        Condition::PrologueChoice(PrologueChoice::HomesteadFirst),
                    ],
                    emotion: Some(EmotionTag::Dry),
                },
            ],
            choices: vec![
                Choice {
                    label: "Ask about the town".to_string(),
                    conditions: vec![],
                    effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("asked_about_town"),
                            value: FlagValue::Bool(true),
                        },
                    ],
                    next: SceneTransition::Scene(SceneId::new("town_info")),
                },
                Choice {
                    label: "Ride on in silence".to_string(),
                    conditions: vec![],
                    effects: vec![],
                    next: SceneTransition::Beat(BeatId::new("p5")),
                },
                // Locked choice — requires a skill
                Choice {
                    label: "Read the situation".to_string(),
                    conditions: vec![
                        Condition::HasSkill {
                            character: CharacterId::new("galen"),
                            skill: SkillId::new("cold_read"),
                        },
                    ],
                    effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("read_town_pressure"),
                            value: FlagValue::Bool(true),
                        },
                    ],
                    next: SceneTransition::Scene(SceneId::new("town_read")),
                },
            ],
            conditions: vec![],
            state_effects: vec![
                StateEffect::AddMemoryObject(MemoryObjectId::new("wanted_poster")),
            ],
            pacing: PacingTag::Pressure,
            memory_refs: vec![
                MemoryRef {
                    object: MemoryObjectId::new("wanted_poster"),
                    callback_type: MemoryCallbackType::Echo,
                    target_chapter: Some(ChapterId::new("ch2")),
                },
            ],
        }
    }

    #[test]
    fn filter_lines_basic() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::new_game(dir.path());
        let scene = test_scene();

        let lines = SceneRunner::filter_lines(&scene, &store);

        // Two unconditional lines show, the conditional Eli line does not
        // (prologue choice is None, not HomesteadFirst)
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].speaker, "narrator");
        assert_eq!(lines[1].speaker, "galen");
    }

    #[test]
    fn filter_lines_with_condition_met() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());
        store.state_mut().prologue_choice = Some(PrologueChoice::HomesteadFirst);

        let scene = test_scene();
        let lines = SceneRunner::filter_lines(&scene, &store);

        // All three lines show — Eli's conditional line passes
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[2].speaker, "eli");
        assert_eq!(lines[2].text, "You took the long way. People noticed.");
    }

    #[test]
    fn evaluate_choices_available_and_locked() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::new_game(dir.path());
        let scene = test_scene();

        let choices = SceneRunner::evaluate_choices(&scene, &store);

        assert_eq!(choices.len(), 3);

        // First two are available (no conditions)
        assert!(choices[0].available);
        assert!(choices[1].available);

        // Third is locked (requires cold_read skill)
        assert!(!choices[2].available);
        assert!(choices[2].lock_reason.is_some());
        let reason = choices[2].lock_reason.as_ref().unwrap();
        assert!(reason.contains("cold_read"), "reason was: {}", reason);
    }

    #[test]
    fn evaluate_choices_unlocked_by_skill() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Unlock cold_read on Galen
        store.apply_effect(&StateEffect::UnlockSkill {
            character: CharacterId::new("galen"),
            skill: SkillId::new("cold_read"),
        });

        let scene = test_scene();
        let choices = SceneRunner::evaluate_choices(&scene, &store);

        // Now all three are available
        assert!(choices[0].available);
        assert!(choices[1].available);
        assert!(choices[2].available);
        assert!(choices[2].lock_reason.is_none());
    }

    #[test]
    fn execute_choice_applies_effects() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());
        let scene = test_scene();

        // Choose "Ask about the town"
        let result = SceneRunner::execute_choice(&scene, 0, &mut store);

        assert!(result.is_some());
        let chosen = result.unwrap();
        assert_eq!(chosen.label, "Ask about the town");

        // Effect was applied
        assert!(store.state().flags.get("asked_about_town")
            == Some(&FlagValue::Bool(true)));
    }

    #[test]
    fn scene_effects_applied() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());
        let scene = test_scene();

        // Scene-level effects add the wanted poster
        assert!(store.state().memory_objects.is_empty());

        SceneRunner::apply_scene_effects(&scene, &mut store);

        assert_eq!(store.state().memory_objects.len(), 1);
        assert_eq!(store.state().memory_objects[0].id.0, "wanted_poster");
    }

    #[test]
    fn memory_callbacks_collected() {
        let scene = test_scene();
        let callbacks = SceneRunner::collect_memory_callbacks(&scene);

        assert_eq!(callbacks.len(), 1);
        assert_eq!(callbacks[0].object.0, "wanted_poster");
        assert_eq!(callbacks[0].target_chapter.as_ref().unwrap().0, "ch2");
    }

    #[test]
    fn prepare_scene_full_pipeline() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::new_game(dir.path());
        let scene = test_scene();

        let prepared = SceneRunner::prepare_scene(&scene, &store);

        assert!(prepared.should_play);
        assert_eq!(prepared.lines.len(), 2); // conditional Eli line filtered
        assert_eq!(prepared.choices.len(), 3); // all visible, one locked
        assert_eq!(prepared.memory_callbacks.len(), 1);
        assert_eq!(prepared.pacing, PacingTag::Pressure);
    }

    #[test]
    fn scene_gated_by_conditions() {
        let dir = TempDir::new().unwrap();
        let store = StateStore::new_game(dir.path());

        // Scene requires a flag that isn't set
        let scene = Scene {
            id: SceneId::new("gated_scene"),
            location: LocationId::new("test"),
            beat: BeatId::new("test"),
            lines: vec![SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "You should not see this.".to_string(),
                conditions: vec![],
                emotion: None,
            }],
            choices: vec![],
            conditions: vec![Condition::Flag {
                id: FlagId::new("chapter3_started"),
                value: FlagValue::Bool(true),
            }],
            state_effects: vec![],
            pacing: PacingTag::Exploration,
            memory_refs: vec![],
        };

        let prepared = SceneRunner::prepare_scene(&scene, &store);
        assert!(!prepared.should_play);
        assert!(prepared.lines.is_empty());
    }

    /// Test the Morrow Crossing poster scene shape:
    /// narration → Eli dialogue → consequential choice.
    #[test]
    fn morrow_crossing_poster_shape() {
        let dir = TempDir::new().unwrap();
        let mut store = StateStore::new_game(dir.path());

        // Build the poster scene
        let poster_scene = Scene {
            id: SceneId::new("prologue_poster"),
            location: LocationId::new("saints_mile_trail"),
            beat: BeatId::new("p2"),
            lines: vec![
                SceneLine {
                    speaker: SpeakerId::new("narrator"),
                    text: "A wanted poster with your name on it. Not as legend — as current business.".to_string(),
                    conditions: vec![],
                    emotion: Some(EmotionTag::Tense),
                },
                SceneLine {
                    speaker: SpeakerId::new("eli"),
                    text: "You riding toward Morrow Crossing? Because if you are, you're either about to get paid or shot. Probably both.".to_string(),
                    conditions: vec![],
                    emotion: Some(EmotionTag::Dry),
                },
            ],
            choices: vec![
                Choice {
                    label: "Tear down the poster".to_string(),
                    conditions: vec![],
                    effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("tore_poster"),
                            value: FlagValue::Bool(true),
                        },
                    ],
                    next: SceneTransition::Scene(SceneId::new("eli_intro")),
                },
                Choice {
                    label: "Leave it. Let them look.".to_string(),
                    conditions: vec![],
                    effects: vec![
                        StateEffect::SetFlag {
                            id: FlagId::new("left_poster"),
                            value: FlagValue::Bool(true),
                        },
                        StateEffect::AdjustReputation {
                            axis: ReputationAxis::TownLaw,
                            delta: -2,
                        },
                    ],
                    next: SceneTransition::Scene(SceneId::new("eli_intro")),
                },
            ],
            conditions: vec![],
            state_effects: vec![
                StateEffect::AddMemoryObject(MemoryObjectId::new("wanted_poster")),
                StateEffect::AddPartyMember(CharacterId::new("eli")),
            ],
            pacing: PacingTag::Pressure,
            memory_refs: vec![
                MemoryRef {
                    object: MemoryObjectId::new("wanted_poster"),
                    callback_type: MemoryCallbackType::Echo,
                    target_chapter: Some(ChapterId::new("ch2")),
                },
            ],
        };

        // Prepare
        let prepared = SceneRunner::prepare_scene(&poster_scene, &store);
        assert!(prepared.should_play);
        assert_eq!(prepared.lines.len(), 2);
        assert_eq!(prepared.choices.len(), 2);
        assert!(prepared.choices[0].available);
        assert!(prepared.choices[1].available);

        // Apply scene effects (poster added, Eli joins)
        SceneRunner::apply_scene_effects(&poster_scene, &mut store);
        assert!(store.state().party.has_member(&CharacterId::new("eli")));
        assert!(store.state().memory_objects.iter().any(|o| o.id.0 == "wanted_poster"));

        // Player chooses "Leave it. Let them look."
        let chosen = SceneRunner::execute_choice(&poster_scene, 1, &mut store).unwrap();
        assert_eq!(chosen.label, "Leave it. Let them look.");

        // Effects applied
        assert_eq!(
            store.state().flags.get("left_poster"),
            Some(&FlagValue::Bool(true))
        );
        // Reputation shifted
        assert_eq!(store.state().reputation.get(ReputationAxis::TownLaw), -2);

        // Transition leads to Eli intro
        match &chosen.transition {
            SceneTransition::Scene(id) => assert_eq!(id.0, "eli_intro"),
            _ => panic!("expected scene transition"),
        }
    }
}
