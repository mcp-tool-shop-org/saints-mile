//! Content authoring helpers — just enough ergonomics to keep chapters clean.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;

// ─── Line Builders ─────────────────────────────────────────────────

/// Narration line — no speaker name shown.
pub fn narrate(text: &str) -> SceneLine {
    SceneLine {
        speaker: SpeakerId::new("narrator"),
        text: text.to_string(),
        conditions: vec![],
        emotion: None,
    }
}

/// Narration with emotion tag.
pub fn narrate_with(text: &str, emotion: EmotionTag) -> SceneLine {
    SceneLine {
        speaker: SpeakerId::new("narrator"),
        text: text.to_string(),
        conditions: vec![],
        emotion: Some(emotion),
    }
}

/// Character dialogue.
pub fn say(speaker: &str, text: &str) -> SceneLine {
    SceneLine {
        speaker: SpeakerId::new(speaker),
        text: text.to_string(),
        conditions: vec![],
        emotion: None,
    }
}

/// Character dialogue with emotion.
pub fn say_with(speaker: &str, text: &str, emotion: EmotionTag) -> SceneLine {
    SceneLine {
        speaker: SpeakerId::new(speaker),
        text: text.to_string(),
        conditions: vec![],
        emotion: Some(emotion),
    }
}

/// Conditional line — only shows if conditions pass.
pub fn say_if(speaker: &str, text: &str, conditions: Vec<Condition>) -> SceneLine {
    SceneLine {
        speaker: SpeakerId::new(speaker),
        text: text.to_string(),
        conditions,
        emotion: None,
    }
}

/// Conditional line with emotion.
pub fn say_if_with(
    speaker: &str,
    text: &str,
    conditions: Vec<Condition>,
    emotion: EmotionTag,
) -> SceneLine {
    SceneLine {
        speaker: SpeakerId::new(speaker),
        text: text.to_string(),
        conditions,
        emotion: Some(emotion),
    }
}

// ─── Condition Helpers ─────────────────────────────────────────────

/// Flag must equal a bool value.
pub fn flag_is(id: &str, value: bool) -> Condition {
    Condition::Flag {
        id: FlagId::new(id),
        value: FlagValue::Bool(value),
    }
}

/// Flag must equal a text value.
pub fn flag_eq(id: &str, value: &str) -> Condition {
    Condition::Flag {
        id: FlagId::new(id),
        value: FlagValue::Text(value.to_string()),
    }
}

/// Character must have a skill.
pub fn has_skill(character: &str, skill: &str) -> Condition {
    Condition::HasSkill {
        character: CharacterId::new(character),
        skill: SkillId::new(skill),
    }
}

// ─── Effect Helpers ────────────────────────────────────────────────

/// Set a boolean flag.
pub fn set_flag(id: &str, value: bool) -> StateEffect {
    StateEffect::SetFlag {
        id: FlagId::new(id),
        value: FlagValue::Bool(value),
    }
}

/// Set a text flag.
pub fn set_text(id: &str, value: &str) -> StateEffect {
    StateEffect::SetFlag {
        id: FlagId::new(id),
        value: FlagValue::Text(value.to_string()),
    }
}

/// Adjust a reputation axis.
pub fn rep(axis: ReputationAxis, delta: i32) -> StateEffect {
    StateEffect::AdjustReputation { axis, delta }
}

/// Unlock a skill on a character.
pub fn unlock(character: &str, skill: &str) -> StateEffect {
    StateEffect::UnlockSkill {
        character: CharacterId::new(character),
        skill: SkillId::new(skill),
    }
}

/// Set relationship between two characters.
pub fn relate(a: &str, b: &str, value: i32) -> StateEffect {
    StateEffect::SetRelationship {
        a: CharacterId::new(a),
        b: CharacterId::new(b),
        value,
    }
}

/// Add a memory object.
pub fn memory(id: &str) -> StateEffect {
    StateEffect::AddMemoryObject(MemoryObjectId::new(id))
}

// ─── Choice Builder ────────────────────────────────────────────────

/// Build a choice with label, effects, and transition.
pub fn choice(label: &str, effects: Vec<StateEffect>, next: SceneTransition) -> Choice {
    Choice {
        label: label.to_string(),
        conditions: vec![],
        effects,
        next,
    }
}

/// Build a conditional choice.
pub fn choice_if(
    label: &str,
    conditions: Vec<Condition>,
    effects: Vec<StateEffect>,
    next: SceneTransition,
) -> Choice {
    Choice {
        label: label.to_string(),
        conditions,
        effects,
        next,
    }
}

/// Shorthand for SceneTransition::Scene.
pub fn to_scene(id: &str) -> SceneTransition {
    SceneTransition::Scene(SceneId::new(id))
}

/// Shorthand for SceneTransition::Combat.
pub fn to_combat(id: &str) -> SceneTransition {
    SceneTransition::Combat(EncounterId::new(id))
}

/// Shorthand for SceneTransition::End.
pub fn end() -> SceneTransition {
    SceneTransition::End
}

// ─── Scene Builder ─────────────────────────────────────────────────

/// Build a scene with minimal boilerplate.
pub fn scene(
    id: &str,
    location: &str,
    beat: &str,
    pacing: PacingTag,
    lines: Vec<SceneLine>,
    choices: Vec<Choice>,
    effects: Vec<StateEffect>,
) -> Scene {
    Scene {
        id: SceneId::new(id),
        location: LocationId::new(location),
        beat: BeatId::new(beat),
        lines,
        choices,
        conditions: vec![],
        state_effects: effects,
        pacing,
        memory_refs: vec![],
    }
}

/// Build a scene with memory refs.
pub fn scene_with_memory(
    id: &str,
    location: &str,
    beat: &str,
    pacing: PacingTag,
    lines: Vec<SceneLine>,
    choices: Vec<Choice>,
    effects: Vec<StateEffect>,
    memory_refs: Vec<MemoryRef>,
) -> Scene {
    Scene {
        id: SceneId::new(id),
        location: LocationId::new(location),
        beat: BeatId::new(beat),
        lines,
        choices,
        conditions: vec![],
        state_effects: effects,
        pacing,
        memory_refs,
    }
}

// ─── Enemy Builder ─────────────────────────────────────────────────

/// Build an enemy template.
pub fn enemy(
    id: &str,
    name: &str,
    hp: i32,
    nerve: i32,
    damage: i32,
    accuracy: i32,
    speed: i32,
) -> EnemyTemplate {
    EnemyTemplate {
        id: id.to_string(),
        name: name.to_string(),
        hp, nerve, damage, accuracy, speed,
        bluff: 20,
        nerve_threshold: 5,
    }
}

/// Build an enemy with full stats.
pub fn enemy_full(
    id: &str,
    name: &str,
    hp: i32,
    nerve: i32,
    damage: i32,
    accuracy: i32,
    speed: i32,
    bluff: i32,
    nerve_threshold: i32,
) -> EnemyTemplate {
    EnemyTemplate {
        id: id.to_string(),
        name: name.to_string(),
        hp, nerve, damage, accuracy, speed,
        bluff, nerve_threshold,
    }
}
