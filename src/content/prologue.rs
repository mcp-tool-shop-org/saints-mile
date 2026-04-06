//! Prologue — Morrow Crossing content.
//!
//! The full slice path: poster → Eli intro → town pressure → Glass Arroyo
//! standoff/fight → campfire choice → return with changed eyes.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// P2 — The wanted poster at the relay post.
pub fn poster_scene() -> Scene {
    Scene {
        id: SceneId::new("prologue_poster"),
        location: LocationId::new("saints_mile_trail"),
        beat: BeatId::new("p2"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Dusk on a dead stretch of trail called Saint's Mile. Your horse drinks \
                       from a nearly dry trough behind an abandoned relay post.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Quiet),
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Nailed to the post is a fresh wanted poster with your name on it. \
                       Not as legend — as current business. Somebody nearby cared enough \
                       to print and post it.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Tense),
            },
        ],
        choices: vec![
            Choice {
                label: "Tear it down".to_string(),
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
                ],
                next: SceneTransition::Scene(SceneId::new("eli_intro")),
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

/// P3 — Eli Winter appears.
pub fn eli_intro_scene() -> Scene {
    Scene {
        id: SceneId::new("eli_intro"),
        location: LocationId::new("saints_mile_trail"),
        beat: BeatId::new("p3"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Before you can decide about the poster, a voice comes from the shadow \
                       side of the relay post. A man steps out — aging grifter's face, bad \
                       shoulder, dry mouth, the kind of look that says he recognizes you \
                       before he says your name.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Tense),
            },
            SceneLine {
                speaker: SpeakerId::new("eli"),
                text: "You riding toward Morrow Crossing? Because if you are, you're either \
                       about to get paid or shot. Probably both.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Dry),
            },
            // Branch: if player tore poster, Eli comments
            SceneLine {
                speaker: SpeakerId::new("eli"),
                text: "You tore that down like it bothered you. Good. Means you're not \
                       used to it yet.".to_string(),
                conditions: vec![
                    Condition::Flag {
                        id: FlagId::new("tore_poster"),
                        value: FlagValue::Bool(true),
                    },
                ],
                emotion: Some(EmotionTag::Dry),
            },
            // Branch: if player left poster
            SceneLine {
                speaker: SpeakerId::new("eli"),
                text: "Left the poster up. Smart or tired, I can't tell. Either way, you're \
                       already known here.".to_string(),
                conditions: vec![
                    Condition::Flag {
                        id: FlagId::new("left_poster"),
                        value: FlagValue::Bool(true),
                    },
                ],
                emotion: Some(EmotionTag::Dry),
            },
        ],
        choices: vec![
            Choice {
                label: "Ride together".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::SetRelationship {
                        a: CharacterId::new("galen"),
                        b: CharacterId::new("eli"),
                        value: 5,
                    },
                ],
                next: SceneTransition::Scene(SceneId::new("morrow_square")),
            },
            Choice {
                label: "Keep your distance".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::SetRelationship {
                        a: CharacterId::new("galen"),
                        b: CharacterId::new("eli"),
                        value: -2,
                    },
                ],
                next: SceneTransition::Scene(SceneId::new("morrow_square")),
            },
        ],
        conditions: vec![],
        state_effects: vec![
            StateEffect::AddPartyMember(CharacterId::new("eli")),
        ],
        pacing: PacingTag::Pressure,
        memory_refs: vec![],
    }
}

/// P5 — Morrow Crossing square standoff.
pub fn morrow_square_scene() -> Scene {
    Scene {
        id: SceneId::new("morrow_square"),
        location: LocationId::new("morrow_crossing"),
        beat: BeatId::new("p5"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Morrow Crossing: a rail-and-ranch town built around a water tower, \
                       a telegraph office, a boarding house, and a church one hard summer \
                       from folding in on itself.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Neutral),
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "You ride into an armed standoff over a locked medicine crate and \
                       the water pump.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Tense),
            },
            SceneLine {
                speaker: SpeakerId::new("vale"),
                text: "Deputy Tomas Vale. I could use someone who knows which end of a \
                       gun works. If you're willing.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Tense),
            },
            SceneLine {
                speaker: SpeakerId::new("alma"),
                text: "Alma Varela stares from across the square, arms folded over a \
                       rifle. She doesn't ask for help. She asks whose side you're on.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Bitter),
            },
            SceneLine {
                speaker: SpeakerId::new("rusk"),
                text: "Gideon Rusk, railroad agent, stands by a locked crate of medicine. \
                       He says he knows exactly what kind of rider drifts into town when \
                       money goes missing.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Tense),
            },
        ],
        choices: vec![
            Choice {
                label: "Side with the deputy — keep order".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::AdjustReputation { axis: ReputationAxis::TownLaw, delta: 10 },
                    StateEffect::AdjustReputation { axis: ReputationAxis::Rancher, delta: -5 },
                    StateEffect::SetFlag {
                        id: FlagId::new("square_stance"),
                        value: FlagValue::Text("law".to_string()),
                    },
                ],
                next: SceneTransition::Scene(SceneId::new("ride_to_arroyo")),
            },
            Choice {
                label: "Side with the ranchers — they need the medicine".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::AdjustReputation { axis: ReputationAxis::Rancher, delta: 10 },
                    StateEffect::AdjustReputation { axis: ReputationAxis::TownLaw, delta: -5 },
                    StateEffect::AdjustReputation { axis: ReputationAxis::Railroad, delta: -5 },
                    StateEffect::SetFlag {
                        id: FlagId::new("square_stance"),
                        value: FlagValue::Text("rancher".to_string()),
                    },
                ],
                next: SceneTransition::Scene(SceneId::new("ride_to_arroyo")),
            },
            Choice {
                label: "Stay neutral — you're here for work, not sides".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("square_stance"),
                        value: FlagValue::Text("neutral".to_string()),
                    },
                ],
                next: SceneTransition::Scene(SceneId::new("ride_to_arroyo")),
            },
        ],
        conditions: vec![],
        state_effects: vec![],
        pacing: PacingTag::Crisis,
        memory_refs: vec![],
    }
}

/// P6 — Ride to Glass Arroyo (trail segment).
pub fn ride_to_arroyo_scene() -> Scene {
    Scene {
        id: SceneId::new("ride_to_arroyo"),
        location: LocationId::new("trail_to_arroyo"),
        beat: BeatId::new("p6"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Dr. Mercer forces the standoff to pause. A second supply satchel — \
                       lost with an overturned wagon in Glass Arroyo, half a day out. \
                       Bring it back before morning.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Tense),
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "The trail eats water and horse. Canteens low. The ride costs more \
                       than distance.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Neutral),
            },
        ],
        choices: vec![
            Choice {
                label: "Push hard — speed over safety".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::AdjustResource { resource: ResourceKind::HorseStamina, delta: -40 },
                    StateEffect::AdjustResource { resource: ResourceKind::Water, delta: -20 },
                ],
                next: SceneTransition::Combat(EncounterId::new("glass_arroyo")),
            },
            Choice {
                label: "Pace yourself — conserve what you have".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::AdjustResource { resource: ResourceKind::HorseStamina, delta: -20 },
                    StateEffect::AdjustResource { resource: ResourceKind::Water, delta: -30 },
                ],
                next: SceneTransition::Combat(EncounterId::new("glass_arroyo")),
            },
        ],
        conditions: vec![],
        state_effects: vec![],
        pacing: PacingTag::Pressure,
        memory_refs: vec![],
    }
}

/// P8 — Campfire on the ridge. The Beat 5 choice.
pub fn campfire_choice_scene() -> Scene {
    Scene {
        id: SceneId::new("campfire_choice"),
        location: LocationId::new("ridge_camp"),
        beat: BeatId::new("p8"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Night falls before town. Cold ridge. Morrow Crossing's lights \
                       barely visible in the distance.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Quiet),
            },
            SceneLine {
                speaker: SpeakerId::new("eli"),
                text: "I tipped the missing sheriff toward the wrong men. For easy cash. \
                       I think he might still be alive — near a burned mission beyond \
                       the Varela homestead.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Grief),
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "There's only enough medicine, water, and horse left to do one \
                       thing before dawn.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Tense),
            },
        ],
        choices: vec![
            Choice {
                label: "Ride straight to town — the square needs stabilizing".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("beat5_choice"),
                        value: FlagValue::Text("town_direct".to_string()),
                    },
                    StateEffect::SetFlag {
                        id: FlagId::new("eli_confession"),
                        value: FlagValue::Bool(true),
                    },
                ],
                next: SceneTransition::Scene(SceneId::new("return_town_direct")),
            },
            Choice {
                label: "Divert to the homestead — children and elderly are burning with fever".to_string(),
                conditions: vec![],
                effects: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("beat5_choice"),
                        value: FlagValue::Text("homestead_first".to_string()),
                    },
                    StateEffect::SetFlag {
                        id: FlagId::new("eli_confession"),
                        value: FlagValue::Bool(true),
                    },
                ],
                next: SceneTransition::Scene(SceneId::new("return_homestead")),
            },
        ],
        conditions: vec![],
        state_effects: vec![],
        pacing: PacingTag::Intimate,
        memory_refs: vec![],
    }
}

/// P9a — Return: went straight to town.
pub fn return_town_direct_scene() -> Scene {
    Scene {
        id: SceneId::new("return_town_direct"),
        location: LocationId::new("morrow_crossing"),
        beat: BeatId::new("p9"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "Dr. Mercer stabilizes the square. Deputy Vale keeps the peace by \
                       inches. The town remembers you prevented a massacre.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Warm),
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "But word spreads: the Varela homestead was left to fend for itself. \
                       Alma's gratitude curdles into something colder than hatred.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Bitter),
            },
            SceneLine {
                speaker: SpeakerId::new("vale"),
                text: "You held the line. That counts for something here.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Warm),
            },
            // Changes based on square stance
            SceneLine {
                speaker: SpeakerId::new("alma"),
                text: "Alma Varela passes you in the street. She does not look.".to_string(),
                conditions: vec![
                    Condition::Flag {
                        id: FlagId::new("square_stance"),
                        value: FlagValue::Text("law".to_string()),
                    },
                ],
                emotion: Some(EmotionTag::Bitter),
            },
            SceneLine {
                speaker: SpeakerId::new("alma"),
                text: "Alma Varela stops you with a look that could strip paint. \
                       'You said you were with us. Then you left us to burn.'".to_string(),
                conditions: vec![
                    Condition::Flag {
                        id: FlagId::new("square_stance"),
                        value: FlagValue::Text("rancher".to_string()),
                    },
                ],
                emotion: Some(EmotionTag::Bitter),
            },
        ],
        choices: vec![],
        conditions: vec![],
        state_effects: vec![
            StateEffect::AdjustReputation { axis: ReputationAxis::TownLaw, delta: 5 },
            StateEffect::AdjustReputation { axis: ReputationAxis::Rancher, delta: -10 },
            StateEffect::SetFlag {
                id: FlagId::new("prologue_complete"),
                value: FlagValue::Bool(true),
            },
        ],
        pacing: PacingTag::Pressure,
        memory_refs: vec![],
    }
}

/// P9b — Return: diverted to homestead first.
pub fn return_homestead_scene() -> Scene {
    Scene {
        id: SceneId::new("return_homestead"),
        location: LocationId::new("morrow_crossing"),
        beat: BeatId::new("p9"),
        lines: vec![
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "The homestead survives. The ranchers speak your name like it means \
                       something.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Warm),
            },
            SceneLine {
                speaker: SpeakerId::new("narrator"),
                text: "But the square went bad before you arrived. Somebody is dead. \
                       Somebody else is maimed. The town's version of events now includes \
                       you as the rider who came too late.".to_string(),
                conditions: vec![],
                emotion: Some(EmotionTag::Grief),
            },
            SceneLine {
                speaker: SpeakerId::new("vale"),
                text: "Where were you? We needed you here.".to_string(),
                conditions: vec![
                    Condition::Flag {
                        id: FlagId::new("square_stance"),
                        value: FlagValue::Text("law".to_string()),
                    },
                ],
                emotion: Some(EmotionTag::Bitter),
            },
            SceneLine {
                speaker: SpeakerId::new("alma"),
                text: "Alma Varela nods once as you pass. Heavy. Grateful. \
                       'My people are alive because of you.'".to_string(),
                conditions: vec![
                    Condition::Flag {
                        id: FlagId::new("square_stance"),
                        value: FlagValue::Text("rancher".to_string()),
                    },
                ],
                emotion: Some(EmotionTag::Warm),
            },
        ],
        choices: vec![],
        conditions: vec![],
        state_effects: vec![
            StateEffect::AdjustReputation { axis: ReputationAxis::Rancher, delta: 10 },
            StateEffect::AdjustReputation { axis: ReputationAxis::TownLaw, delta: -10 },
            StateEffect::SetFlag {
                id: FlagId::new("prologue_complete"),
                value: FlagValue::Bool(true),
            },
        ],
        pacing: PacingTag::Pressure,
        memory_refs: vec![],
    }
}

// ─── Encounters ────────────────────────────────────────────────────

/// Glass Arroyo standoff and shootout.
pub fn glass_arroyo_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("glass_arroyo"),
        phases: vec![CombatPhase {
            id: "ambush".to_string(),
            description: "Armed group on the far ridge at the wagon wreck.".to_string(),
            enemies: vec![
                EnemyTemplate {
                    id: "ridge_raider".to_string(),
                    name: "Ridge Raider".to_string(),
                    hp: 25, nerve: 20, damage: 8, accuracy: 55,
                    speed: 8, bluff: 30, nerve_threshold: 5,
                },
                EnemyTemplate {
                    id: "hired_gun".to_string(),
                    name: "Hired Gunman".to_string(),
                    hp: 30, nerve: 25, damage: 10, accuracy: 60,
                    speed: 6, bluff: 15, nerve_threshold: 8,
                },
                EnemyTemplate {
                    id: "lookout".to_string(),
                    name: "Nervous Lookout".to_string(),
                    hp: 15, nerve: 10, damage: 5, accuracy: 40,
                    speed: 10, bluff: 50, nerve_threshold: 8,
                },
            ],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: Some(Standoff {
            postures: vec![
                StandoffPosture::EarlyDraw,
                StandoffPosture::SteadyHand,
                StandoffPosture::Bait,
            ],
            allow_focus: true,
            eli_influence: true,
        }),
        party_slots: 4,
        terrain: Terrain {
            name: "Glass Arroyo".to_string(),
            cover: vec![
                CoverElement {
                    name: "Overturned wagon".to_string(),
                    durability: 50,
                    destructible: true,
                },
                CoverElement {
                    name: "Rock outcrop".to_string(),
                    durability: 100,
                    destructible: false,
                },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "survive".to_string(),
                label: "Survive the ambush".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![],
                success_consequence: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("arroyo_survived"),
                        value: FlagValue::Bool(true),
                    },
                ],
            },
            Objective {
                id: "satchel".to_string(),
                label: "Secure the medicine satchel".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("satchel_lost"),
                        value: FlagValue::Bool(true),
                    },
                ],
                success_consequence: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("satchel_secured"),
                        value: FlagValue::Bool(true),
                    },
                ],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Prologue party data for Galen + Eli.
pub fn prologue_party() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![
        (
            "galen".to_string(), "Galen Rook".to_string(),
            40, 30, 12,  // hp, nerve, ammo
            12, 70, 10,  // speed, accuracy, damage
            vec![
                SkillId::new("quick_draw"),
                SkillId::new("called_shot"),
                SkillId::new("take_cover"),
                SkillId::new("rally"),
                SkillId::new("setup_shot"),
                SkillId::new("overwatch"),
            ],
            vec![DuoTechId::new("loaded_deck")],
            vec![],
        ),
        (
            "eli".to_string(), "Eli Winter".to_string(),
            30, 25, 8,
            10, 50, 6,
            vec![
                SkillId::new("sidearm"),
                SkillId::new("fast_talk"),
                SkillId::new("bluff"),
                SkillId::new("dirty_trick"),
                SkillId::new("patch_up"),
            ],
            vec![DuoTechId::new("loaded_deck")],
            vec![],
        ),
    ]
}

// ─── Scene Registry ────────────────────────────────────────────────

/// Get a scene by ID. Returns None if not found.
pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "prologue_poster" => Some(poster_scene()),
        "eli_intro" => Some(eli_intro_scene()),
        "morrow_square" => Some(morrow_square_scene()),
        "ride_to_arroyo" => Some(ride_to_arroyo_scene()),
        "campfire_choice" => Some(campfire_choice_scene()),
        "return_town_direct" => Some(return_town_direct_scene()),
        "return_homestead" => Some(return_homestead_scene()),
        _ => None,
    }
}

/// Get an encounter by ID.
pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "glass_arroyo" => Some(glass_arroyo_encounter()),
        _ => None,
    }
}
