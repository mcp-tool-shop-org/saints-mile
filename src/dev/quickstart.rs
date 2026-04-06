//! Quickstart system — jump to key scenes/encounters for testing.
//!
//! Provides pre-configured game states at critical points so testers
//! don't have to replay from the beginning to reach Bitter Cut or the relay.

use crate::types::*;
use crate::scene::types::StateEffect;
use crate::state::store::StateStore;
use crate::state::types::GameState;

/// Named jump points in the opening arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpPoint {
    /// Prologue start — fresh game.
    PrologueStart,
    /// Prologue Glass Arroyo — just before the fight.
    PrologueArroyo,
    /// Prologue campfire — just before Beat 5 choice.
    PrologueCampfire,
    /// Chapter 1 start — Cedar Wake arrival.
    CedarWakeStart,
    /// Chapter 1 shooting post — just before Steady Aim unlock.
    CedarWakeShootingPost,
    /// Chapter 1 bandit camp — just before the "clean work" fight.
    CedarWakeBanditCamp,
    /// Chapter 1 Bitter Cut — just before the dispatch.
    BitterCutDispatch,
    /// Chapter 1 Bitter Cut — just before the fight.
    BitterCutFight,
    /// Chapter 2 convoy join.
    ConvoyStart,
    /// Chapter 2 Red Switch Wash — just before the fight.
    RedSwitchWash,
    /// Chapter 2 Hollow Pump — just before the encounter.
    HollowPump,
    /// Chapter 2 Night 2 — Eli perimeter walk.
    ConvoyNight2,
    /// Chapter 2 relay arrival — just before the break.
    RelayArrival,
    /// Chapter 2 triage — just before the branch choice.
    RelayTriage,
}

impl JumpPoint {
    /// Create a GameState configured for this jump point.
    ///
    /// Each jump point builds on the previous one's state via cascading calls
    /// (e.g. `BitterCutFight` calls `BitterCutDispatch.create_state()`). This is
    /// intentional — it ensures state consistency across the narrative timeline
    /// so later jump points carry all the flags, skills, and effects of earlier ones.
    pub fn create_state(&self) -> GameState {
        match self {
            JumpPoint::PrologueStart => GameState::new_game(),

            JumpPoint::PrologueArroyo => {
                let mut state = GameState::new_game();
                state.beat = BeatId::new("p7");
                state.apply_effect(&StateEffect::AddMemoryObject(MemoryObjectId::new("wanted_poster")));
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("tore_poster"), value: FlagValue::Bool(true),
                });
                state.apply_effect(&StateEffect::AdjustReputation {
                    axis: ReputationAxis::TownLaw, delta: 5,
                });
                state
            }

            JumpPoint::PrologueCampfire => {
                let mut state = Self::PrologueArroyo.create_state();
                state.beat = BeatId::new("p8");
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("arroyo_survived"), value: FlagValue::Bool(true),
                });
                state
            }

            JumpPoint::CedarWakeStart => {
                let mut state = GameState::new_game();
                state.chapter = ChapterId::new("ch1");
                state.beat = BeatId::new("1a1");
                state.age_phase = AgePhase::Youth;
                // Strip party to solo Galen
                state.party.members.retain(|m| m.id.0 == "galen");
                // Reset skills to youth kit
                if let Some(galen) = state.party.members.iter_mut().find(|m| m.id.0 == "galen") {
                    galen.unlocked_skills = vec![
                        SkillId::new("quick_draw"),
                        SkillId::new("snap_shot"),
                        SkillId::new("duck"),
                        SkillId::new("sprint"),
                    ];
                }
                state
            }

            JumpPoint::CedarWakeShootingPost => {
                let mut state = Self::CedarWakeStart.create_state();
                state.beat = BeatId::new("1b1");
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("met_molly"), value: FlagValue::Bool(true),
                });
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("met_declan"), value: FlagValue::Bool(true),
                });
                state.apply_effect(&StateEffect::UnlockSkill {
                    character: CharacterId::new("galen"),
                    skill: SkillId::new("trail_eye"),
                });
                state
            }

            JumpPoint::CedarWakeBanditCamp => {
                let mut state = Self::CedarWakeShootingPost.create_state();
                state.beat = BeatId::new("1b7");
                state.apply_effect(&StateEffect::UnlockSkill {
                    character: CharacterId::new("galen"),
                    skill: SkillId::new("steady_aim"),
                });
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("voss_taught_steady_aim"), value: FlagValue::Bool(true),
                });
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("horse_thief_done"), value: FlagValue::Bool(true),
                });
                state
            }

            JumpPoint::BitterCutDispatch => {
                let mut state = Self::CedarWakeBanditCamp.create_state();
                state.beat = BeatId::new("1c1");
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("bandit_camp_done"), value: FlagValue::Bool(true),
                });
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("clean_work"), value: FlagValue::Bool(true),
                });
                state
            }

            JumpPoint::BitterCutFight => {
                let mut state = Self::BitterCutDispatch.create_state();
                state.beat = BeatId::new("1c4");
                state.apply_effect(&StateEffect::SetFlag {
                    id: FlagId::new("carrying_dispatch"), value: FlagValue::Bool(true),
                });
                state
            }

            JumpPoint::ConvoyStart => {
                // Cascade from BitterCutFight — carries all Ch1 flags, skills, and effects
                let mut state = Self::BitterCutFight.create_state();
                state.chapter = ChapterId::new("ch2");
                state.beat = BeatId::new("2d1");
                state.age_phase = AgePhase::YoungMan;
                // Solo Galen for convoy — Eli is separated between chapters
                state.party.members.retain(|m| m.id.0 == "galen");
                // Upgrade to young-man skill kit (Ch1 skills + new unlocks)
                if let Some(galen) = state.party.members.iter_mut().find(|m| m.id.0 == "galen") {
                    galen.unlocked_skills = vec![
                        SkillId::new("quick_draw"),
                        SkillId::new("snap_shot"),
                        SkillId::new("duck"),
                        SkillId::new("steady_aim"),
                        SkillId::new("trail_eye"),
                        SkillId::new("called_shot_basic"),
                        SkillId::new("cold_read"),
                        SkillId::new("grit"),
                    ];
                }
                // Mark chapter transition
                state.flags.insert("chapter1_complete".to_string(), FlagValue::Bool(true));
                state.flags.insert("bitter_cut_done".to_string(), FlagValue::Bool(true));
                state
            }

            JumpPoint::RedSwitchWash => {
                let mut state = Self::ConvoyStart.create_state();
                state.beat = BeatId::new("2d1_wash");
                state.flags.insert("formation".to_string(), FlagValue::Text("scout".to_string()));
                state
            }

            JumpPoint::HollowPump => {
                let mut state = Self::RedSwitchWash.create_state();
                state.beat = BeatId::new("2d2");
                state.flags.insert("wash_survived".to_string(), FlagValue::Bool(true));
                state
            }

            JumpPoint::ConvoyNight2 => {
                let mut state = Self::HollowPump.create_state();
                state.beat = BeatId::new("2n2");
                state.flags.insert("pump_resolved".to_string(), FlagValue::Bool(true));
                state.flags.insert("took_flask".to_string(), FlagValue::Bool(true));
                state.memory_objects.push(crate::state::types::MemoryObject {
                    id: MemoryObjectId::new("nella_coffee"),
                    state: "active".to_string(),
                });
                state.memory_objects.push(crate::state::types::MemoryObject {
                    id: MemoryObjectId::new("eli_flask"),
                    state: "active".to_string(),
                });
                state
            }

            JumpPoint::RelayArrival => {
                let mut state = Self::ConvoyNight2.create_state();
                state.beat = BeatId::new("2d3");
                state.memory_objects.push(crate::state::types::MemoryObject {
                    id: MemoryObjectId::new("nella_bath_bread_roof"),
                    state: "active".to_string(),
                });
                state.memory_objects.push(crate::state::types::MemoryObject {
                    id: MemoryObjectId::new("nella_biscuit_cloth"),
                    state: "active".to_string(),
                });
                state
            }

            JumpPoint::RelayTriage => {
                let mut state = Self::RelayArrival.create_state();
                state.flags.insert("relay_survived".to_string(), FlagValue::Bool(true));
                state.flags.insert("bale_dead".to_string(), FlagValue::Bool(true));
                state
            }
        }
    }

    /// All jump points, in order.
    pub fn all() -> &'static [JumpPoint] {
        &[
            JumpPoint::PrologueStart,
            JumpPoint::PrologueArroyo,
            JumpPoint::PrologueCampfire,
            JumpPoint::CedarWakeStart,
            JumpPoint::CedarWakeShootingPost,
            JumpPoint::CedarWakeBanditCamp,
            JumpPoint::BitterCutDispatch,
            JumpPoint::BitterCutFight,
            JumpPoint::ConvoyStart,
            JumpPoint::RedSwitchWash,
            JumpPoint::HollowPump,
            JumpPoint::ConvoyNight2,
            JumpPoint::RelayArrival,
            JumpPoint::RelayTriage,
        ]
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::PrologueStart => "Prologue — Start",
            Self::PrologueArroyo => "Prologue — Glass Arroyo",
            Self::PrologueCampfire => "Prologue — Campfire Choice",
            Self::CedarWakeStart => "Ch1 — Cedar Wake Arrival",
            Self::CedarWakeShootingPost => "Ch1 — Shooting Post",
            Self::CedarWakeBanditCamp => "Ch1 — Bandit Camp",
            Self::BitterCutDispatch => "Ch1 — Bitter Cut Dispatch",
            Self::BitterCutFight => "Ch1 — Bitter Cut Fight",
            Self::ConvoyStart => "Ch2 — Convoy Join",
            Self::RedSwitchWash => "Ch2 — Red Switch Wash",
            Self::HollowPump => "Ch2 — Hollow Pump",
            Self::ConvoyNight2 => "Ch2 — Night 2 Camp",
            Self::RelayArrival => "Ch2 — Relay Arrival",
            Self::RelayTriage => "Ch2 — Relay Triage",
        }
    }
}
