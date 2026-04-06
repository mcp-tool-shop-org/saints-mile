//! Chapter 5 — Dust Revival.
//!
//! Emotional law: Containment. The party tries to stop a community
//! from becoming a weapon. Miriam joins.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Arrival at Silt Crossing revival ground.
pub fn revival_arrival() -> Scene {
    scene(
        "dr_arrival", "silt_crossing", "5_1",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Silt Crossing: a town deciding whether to die or not. The well \
                 is marginal. The rail spur was promised and never built. A revival \
                 has filled the empty spaces with something between hope and hysteria.",
                EmotionTag::Tense,
            ),
            narrate(
                "More tents than expected. More people. More heat. More need. \
                 A family at the revival's edge, isolated — the Dunnicks. Nobody \
                 will trade with them. Their fire is too small.",
            ),
        ],
        vec![
            choice("Read the revival", vec![], to_scene("dr_read_revival")),
        ],
        vec![
            set_flag("ch5_started", true),
        ],
    )
}

/// Reading the revival — party splits attention.
pub fn read_revival() -> Scene {
    scene(
        "dr_read_revival", "silt_crossing", "5_2",
        PacingTag::Pressure,
        vec![
            say_with("ada",
                "These people aren't cursed. They're dehydrated, dust-exposed, \
                 and drinking from a contaminated source. I can see the fever \
                 pattern from here.",
                EmotionTag::Bitter,
            ),
            say_with("eli",
                "Two men in the edge camp. Arrived a week before the revival. \
                 Buying land options from families too scared to hold.",
                EmotionTag::Dry,
            ),
            say_with("rosa",
                "Stock families from three counties. Some I know. Some know \
                 my mother. The land-grab fear is making people sell.",
                EmotionTag::Bitter,
            ),
        ],
        vec![
            choice("Visit the Dunnick camp", vec![], to_scene("dr_dunnick_camp")),
            choice("Attend the evening sermon", vec![], to_scene("dr_sermon")),
        ],
        vec![],
    )
}

/// The Dunnick family — the accused.
pub fn dunnick_camp() -> Scene {
    scene(
        "dr_dunnick_camp", "silt_crossing_edge", "5_3",
        PacingTag::Intimate,
        vec![
            narrate(
                "Wes Dunnick, father. Stubborn, ashamed, holding ground because \
                 leaving means admitting guilt he doesn't carry.",
            ),
            say_with("etta",
                "Etta Dunnick, his daughter. Not eating so her brother can. \
                 Sharp enough to know what's happening.",
                EmotionTag::Bitter,
            ),
            say("etta", "You here to help, or to watch?"),
            say("galen", "Depends what's actually happening."),
            say_with("etta",
                "We drilled a well. The water came up bad. Now we're the reason \
                 the drought has a face.",
                EmotionTag::Bitter,
            ),
        ],
        vec![
            choice("Attend the sermon", vec![
                set_flag("visited_dunnicks", true),
            ], to_scene("dr_sermon")),
        ],
        vec![],
    )
}

/// Brother Whittle's sermon — rhetoric as frontier weapon.
pub fn sermon_scene() -> Scene {
    scene(
        "dr_sermon", "revival_stage", "5_4",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "Brother Amos Whittle is powerful, sincere, and increasingly \
                 specific about the Dunnick well. He does not say 'kill them.' \
                 He says 'the water is marked' and 'the faithful must cleanse \
                 what poisons the ground.'",
                EmotionTag::Tense,
            ),
            narrate(
                "The crowd hears what it needs to hear.",
            ),
            narrate_with(
                "A woman stands at the back of the gathering. Poised, watchful. \
                 She recognizes the shape of what is about to happen because \
                 she used to be the person shaping it.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Wait for morning", vec![], to_scene("dr_crowd_breaks")),
        ],
        vec![],
    )
}

/// The crowd breaks toward the Dunnick camp.
pub fn crowd_breaks() -> Scene {
    scene(
        "dr_crowd_breaks", "silt_crossing", "5_5",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "Morning. The crowd moves toward the Dunnick camp. Whittle may \
                 or may not be leading it — the crowd has its own momentum now.",
                EmotionTag::Tense,
            ),
            narrate(
                "The Dunnicks are about to be dragged out.",
            ),
            narrate_with(
                "The woman from the sermon steps into the crowd's path. She \
                 speaks. Not shouting. Not pleading. Speaking with the cadence \
                 of someone who knows exactly how a room turns and has decided \
                 this one will not.",
                EmotionTag::Tense,
            ),
            say_with("miriam",
                "I've spent ten years learning how a crowd turns. I won't \
                 watch it happen here and tell myself I didn't know better.",
                EmotionTag::Neutral,
            ),
            narrate(
                "She holds the crowd for a moment. Then the crowd turns on \
                 her instead — she's an outsider, she's defending the cursed \
                 family, she's suspicious.",
            ),
            narrate_with(
                "The party has to get her out. That's how she joins.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            // The five-way argument becomes the combat approach
            choice("Let Miriam speak — trust her to hold the room", vec![
                set_flag("ch5_approach", true),
                set_text("ch5_stance", "miriam_speaks"),
                relate("galen", "miriam", 5),
            ], to_scene("dr_crowd_fight_setup")),
            choice("Handle it by force — Rosa holds the line physically", vec![
                set_text("ch5_stance", "force_hold"),
                relate("galen", "rosa", 3),
                relate("galen", "miriam", -3),
            ], to_scene("dr_crowd_fight_setup")),
        ],
        vec![],
    )
}

/// Setup for the crowd fight.
pub fn crowd_fight_setup() -> Scene {
    scene(
        "dr_crowd_fight_setup", "silt_crossing", "5_5",
        PacingTag::Crisis,
        vec![
            narrate("The crowd surges. This is not a conventional fight."),
            narrate_with(
                "Objective: break the mob's nerve before the Dunnick family is killed.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Engage the crowd", vec![], to_combat("crowd_containment")),
        ],
        vec![
            // Miriam joins as temporary NPC for the crowd fight
            StateEffect::AddPartyMember(CharacterId::new("miriam")),
        ],
    )
}

/// Post-crowd fight: the aftermath hired guns.
pub fn crowd_aftermath_intro() -> Scene {
    scene(
        "dr_aftermath_intro", "silt_crossing", "5_5",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The crowd scatters. But armed men who were using the revival \
                 as cover for a land grab now move in. Real enemies — hired guns \
                 with nerve, cover, and intent.",
                EmotionTag::Tense,
            ),
            narrate(
                "The party is battered from the crowd encounter. Wounds carry \
                 over. Ammo is partially spent.",
            ),
        ],
        vec![
            choice("Fight the land-grab men", vec![], to_combat("aftermath_guns")),
        ],
        vec![],
    )
}

/// Chapter close — Miriam joins permanently.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "dr_chapter_close", "silt_crossing", "5_6",
        PacingTag::Intimate,
        vec![
            say_if_with("miriam",
                "You trusted me to speak. The room held. That's not nothing.",
                vec![flag_eq("ch5_stance", "miriam_speaks")],
                EmotionTag::Quiet,
            ),
            say_if_with("miriam",
                "You overrode the voice for the fist. The room held anyway. \
                 But that's not how I'd have done it.",
                vec![flag_eq("ch5_stance", "force_hold")],
                EmotionTag::Bitter,
            ),
            say_with("miriam",
                "The room doesn't stay open by itself. Somebody has to be \
                 there when the talking matters. That's what I'm for.",
                EmotionTag::Neutral,
            ),
            narrate_with(
                "Miriam joins the party. Not through recruitment. Through \
                 measured intervention.",
                EmotionTag::Neutral,
            ),
        ],
        vec![],
        vec![
            set_flag("miriam_joined", true),
            set_flag("ch5_complete", true),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("revival_memory"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch10")),
            },
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// Crowd containment — non-lethal, nerve-based.
/// This is NOT a standard combat encounter. It uses the crowd pressure system.
/// But we provide a combat encounter shell for the aftermath.
pub fn crowd_containment_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("crowd_containment"),
        phases: vec![CombatPhase {
            id: "crowd".to_string(),
            description: "Break the mob's nerve before the Dunnick family is killed.".to_string(),
            // "Enemies" are ringleaders — the crowd itself is the CrowdState
            enemies: vec![
                enemy_full("ringleader_a", "Loud Accuser", 15, 12, 3, 25, 5, 30, 4),
                enemy_full("ringleader_b", "Angry Farmer", 18, 15, 4, 30, 4, 20, 5),
            ],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None, // No standoff with a crowd
        party_slots: 4,
        terrain: Terrain {
            name: "Silt Crossing Revival Ground".to_string(),
            cover: vec![
                CoverElement { name: "Revival stage".to_string(), durability: 60, destructible: false },
                CoverElement { name: "Tent posts".to_string(), durability: 15, destructible: true },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "contain_crowd".to_string(),
                label: "Contain the crowd".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![set_flag("crowd_broke", true)],
                success_consequence: vec![set_flag("crowd_contained", true)],
            },
            Objective {
                id: "protect_dunnicks".to_string(),
                label: "Protect the Dunnick family".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![set_flag("dunnicks_harmed", true)],
                success_consequence: vec![set_flag("dunnicks_safe", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Aftermath hired guns — conventional combat after crowd disperses.
pub fn aftermath_guns_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("aftermath_guns"),
        phases: vec![CombatPhase {
            id: "land_grab".to_string(),
            description: "Rusk-connected hired guns move in when the crowd scatters.".to_string(),
            enemies: vec![
                enemy_full("hired_gun_a", "Land-Grab Enforcer", 30, 24, 10, 58, 7, 15, 7),
                enemy_full("hired_gun_b", "Land-Grab Enforcer", 28, 22, 9, 55, 8, 15, 6),
                enemy("hired_gun_c", "Hired Gun", 25, 18, 8, 50, 6),
            ],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: Some(Standoff {
            postures: vec![StandoffPosture::EarlyDraw, StandoffPosture::SteadyHand, StandoffPosture::Bait],
            allow_focus: true,
            eli_influence: true,
        }),
        party_slots: 4,
        terrain: Terrain {
            name: "Silt Crossing — Post-Revival".to_string(),
            cover: vec![
                CoverElement { name: "Overturned cart".to_string(), durability: 40, destructible: true },
                CoverElement { name: "Stone well".to_string(), durability: 100, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![Objective {
            id: "survive_aftermath".to_string(),
            label: "Survive the land-grab enforcers".to_string(),
            objective_type: ObjectiveType::Primary,
            fail_consequence: vec![],
            success_consequence: vec![set_flag("aftermath_survived", true)],
        }],
        outcome_effects: vec![],
        escapable: true,
    }
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "dr_arrival" => Some(revival_arrival()),
        "dr_read_revival" => Some(read_revival()),
        "dr_dunnick_camp" => Some(dunnick_camp()),
        "dr_sermon" => Some(sermon_scene()),
        "dr_crowd_breaks" => Some(crowd_breaks()),
        "dr_crowd_fight_setup" => Some(crowd_fight_setup()),
        "dr_aftermath_intro" => Some(crowd_aftermath_intro()),
        "dr_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "crowd_containment" => Some(crowd_containment_encounter()),
        "aftermath_guns" => Some(aftermath_guns_encounter()),
        _ => None,
    }
}
