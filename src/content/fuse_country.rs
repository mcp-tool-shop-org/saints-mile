//! Chapter 6 — Fuse Country.
//!
//! Emotional law: Contamination. Explosives make damage travel.
//! Lucien arrives as damage before face, consequence before charisma.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Into the corridor — evidence of Lucien's work before meeting him.
pub fn corridor_entry() -> Scene {
    scene(
        "fc_corridor_entry", "eastern_corridor", "6_1",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The eastern corridor of the Cinder Basin. Rail cuts, dry washes, \
                 and the scars: blast craters filled with rainwater, blackened timbers, \
                 a bridge rebuilt twice on the same pylons.",
                EmotionTag::Tense,
            ),
            narrate(
                "A road detour around a blown culvert. Water pooled where it \
                 shouldn't be. Trail Eye reads the blast pattern.",
            ),
        ],
        vec![
            choice("Investigate the burned freight depot", vec![], to_scene("fc_burned_depot")),
        ],
        vec![
            set_flag("ch6_started", true),
        ],
    )
}

/// The burned freight depot — Lucien's cost made visible.
pub fn burned_depot() -> Scene {
    scene(
        "fc_burned_depot", "burned_depot", "6_2",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "A supply depot that burned three weeks ago. Rail says lightning. \
                 The blast pattern says otherwise.",
                EmotionTag::Tense,
            ),
            say_with("ada",
                "Deliberate ignition. Not lightning. The fire started at the \
                 medicine crates. This is where Black Willow's missing supplies \
                 were supposed to be.",
                EmotionTag::Bitter,
            ),
            narrate(
                "Scorched medical supply manifests connect directly to the fever \
                 medicine Ada couldn't find in Chapter 3. The chain is physical now.",
            ),
            say_with("maeve",
                "Maeve Strand, stock widow. Big hands. Loud mouth. Eats like \
                 he's celebrating. That's the man who does this work. He blew \
                 the bridge my cattle trail crossed.",
                EmotionTag::Bitter,
            ),
        ],
        vec![
            choice("Follow the trail to the trestle", vec![
                set_flag("depot_investigated", true),
            ], to_scene("fc_corridor_locals")),
        ],
        vec![],
    )
}

/// Corridor locals — displaced family, rail section boss.
pub fn corridor_locals() -> Scene {
    scene(
        "fc_corridor_locals", "colter_station", "6_3",
        PacingTag::Pressure,
        vec![
            narrate(
                "Colter Station: a rail siding that was decommissioned after the \
                 previous trestle damage. A family displaced by the depot fire \
                 sleeps in a wagon, waiting for compensation that will arrive \
                 as a land buyout offer.",
            ),
            narrate(
                "The sheriff's trail leads here. Elias Mercer was asking about \
                 demolition contracts before he disappeared.",
            ),
        ],
        vec![
            choice("Continue to the trestle", vec![], to_scene("fc_meet_lucien")),
        ],
        vec![],
    )
}

/// Meet Lucien — damage before face, consequence before charisma.
pub fn meet_lucien() -> Scene {
    scene(
        "fc_meet_lucien", "millburn_trestle", "6_4",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The Millburn Trestle spans a dry canyon. Charges are visible on \
                 the support pylons. A man stands near the work — broad-shouldered, \
                 scarred hands, eating from a tin like the canyon owes him lunch.",
                EmotionTag::Tense,
            ),
            say_with("lucien",
                "Every road costs somebody. I'm just the cost you can see.",
                EmotionTag::Neutral,
            ),
            say("galen", "The depot fire. The bridge. The culvert. That's your work."),
            say_with("lucien",
                "I don't choose the target. I choose the charge weight. \
                 That's professionalism.",
                EmotionTag::Neutral,
            ),
            say_with("rosa",
                "You blew the bridge my cattle trail crossed. You burned the \
                 depot where our medicine was.",
                EmotionTag::Bitter,
            ),
            say_with("lucien",
                "You want to blame me for the bridge? Blame the man who built \
                 it where people were already living.",
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("The trestle is rigged — stop the detonation", vec![],
                to_scene("fc_trestle_approach")),
        ],
        vec![],
    )
}

/// Trestle approach — the party arguments about Lucien.
pub fn trestle_approach() -> Scene {
    scene(
        "fc_trestle_approach", "millburn_trestle", "6_5",
        PacingTag::Crisis,
        vec![
            narrate("The trestle has three charge sets on the pylons. Fuses are lit."),
            // The argument map
            say_with("rosa", "Kill him now.", EmotionTag::Bitter),
            say_with("ada",
                "Stop the damage first. The trestle is about to go. Settle him after.",
                EmotionTag::Tense,
            ),
            say_with("miriam",
                "Do not let vengeance turn the room into him.",
                EmotionTag::Neutral,
            ),
            say_with("eli",
                "He's a contractor. He has a client. The client matters more \
                 than the tool. Make him useful or let him lead you to someone \
                 who matters.",
                EmotionTag::Dry,
            ),
        ],
        vec![
            choice("Stop the trestle — deal with Lucien after", vec![
                set_text("ch6_stance", "stop_first"),
            ], to_combat("millburn_trestle")),
        ],
        vec![],
    )
}

/// Post-trestle: what to do with Lucien.
pub fn lucien_decision() -> Scene {
    scene(
        "fc_lucien_decision", "millburn_trestle", "6_6",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The trestle holds. Or most of it does. The canyon smells like \
                 sulfur and hot wood.",
                EmotionTag::Grief,
            ),
            narrate("Lucien is cornered. His crew is scattered. The question is \
                     what happens next."),
        ],
        vec![
            choice("Hold him as prisoner — Rosa's way", vec![
                set_text("lucien_status", "prisoner"),
                relate("galen", "rosa", 3),
            ], to_scene("fc_chapter_close")),
            choice("Use him as forced guide — Eli's way", vec![
                set_text("lucien_status", "forced_guide"),
                relate("galen", "eli", 3),
                relate("galen", "rosa", -3),
            ], to_scene("fc_chapter_close")),
            choice("Let Miriam hold him to account", vec![
                set_text("lucien_status", "judged"),
                relate("galen", "miriam", 3),
            ], to_scene("fc_chapter_close")),
        ],
        vec![],
    )
}

/// Chapter close — Lucien is captured but NOT recruited.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "fc_chapter_close", "eastern_corridor", "6_6",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Lucien reveals under pressure: he was contracted through \
                 intermediaries connected to the Briar Line regional office. \
                 The demolition pattern matches a land-acquisition strategy.",
                EmotionTag::Tense,
            ),
            say_with("lucien",
                "You think this canyon was peaceful before I got here?",
                EmotionTag::Neutral,
            ),
            narrate(
                "He knows where the next set of instructions was supposed to \
                 come from: a rail city called Iron Ledger.",
            ),
            // Status-specific lines
            say_if_with("rosa",
                "He stays restrained. If he runs, I rope him.",
                vec![flag_eq("lucien_status", "prisoner")],
                EmotionTag::Bitter,
            ),
            say_if_with("eli",
                "He knows the network's demolition infrastructure. Make him \
                 walk us through it.",
                vec![flag_eq("lucien_status", "forced_guide")],
                EmotionTag::Dry,
            ),
            say_if_with("miriam",
                "He will face what he's done. Not with convenient violence. \
                 With the weight of it.",
                vec![flag_eq("lucien_status", "judged")],
                EmotionTag::Quiet,
            ),
        ],
        vec![],
        vec![
            set_flag("ch6_complete", true),
            set_flag("lucien_captured", true),
            set_flag("iron_ledger_lead", true),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("trestle_blast_scar"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch8")),
            },
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// The Millburn Trestle — environmental combat signature set piece.
pub fn trestle_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("millburn_trestle"),
        phases: vec![CombatPhase {
            id: "trestle_fight".to_string(),
            description: "Fight Lucien's crew while disarming charges on the trestle.".to_string(),
            enemies: vec![
                enemy_full("demo_crew_a", "Demo Crewman", 22, 15, 7, 45, 7, 15, 5),
                enemy_full("demo_crew_b", "Demo Crewman", 20, 12, 6, 40, 8, 10, 4),
                enemy_full("nervous_rookie", "Nervous Rookie", 15, 8, 5, 35, 6, 30, 6),
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
            name: "Millburn Trestle".to_string(),
            cover: vec![
                CoverElement { name: "Pylon crossbeam".to_string(), durability: 40, destructible: true },
                CoverElement { name: "Supply crate".to_string(), durability: 20, destructible: true },
                CoverElement { name: "Rail car".to_string(), durability: 80, destructible: false },
            ],
            hazards: vec![
                EnvironmentalHazard::FuseCharge { turns_to_detonate: 3, blast_radius: 2 },
                EnvironmentalHazard::FuseCharge { turns_to_detonate: 4, blast_radius: 2 },
                EnvironmentalHazard::CollapseRisk { trigger_damage: 35 },
            ],
        },
        objectives: vec![
            Objective {
                id: "save_trestle".to_string(),
                label: "Save the trestle".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![set_flag("trestle_destroyed", true)],
                success_consequence: vec![set_flag("trestle_saved", true)],
            },
            Objective {
                id: "disarm_charges".to_string(),
                label: "Disarm the charges".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![set_flag("charges_detonated", true)],
                success_consequence: vec![set_flag("charges_disarmed", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "fc_corridor_entry" => Some(corridor_entry()),
        "fc_burned_depot" => Some(burned_depot()),
        "fc_corridor_locals" => Some(corridor_locals()),
        "fc_meet_lucien" => Some(meet_lucien()),
        "fc_trestle_approach" => Some(trestle_approach()),
        "fc_lucien_decision" => Some(lucien_decision()),
        "fc_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "millburn_trestle" => Some(trestle_encounter()),
        _ => None,
    }
}
