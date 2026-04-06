//! Chapter 3 — Black Willow Fever.
//!
//! Saint's Mile → refusal. The first adult chapter. Ada joins. Wounds matter.
//! The game proves it can sustain a campaign, not just a great opening.
//!
//! Emotional law: Refusal. Galen refuses to let paper outrank bodies.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Entry Scenes (vary by relay branch) ───────────────────────────

/// Morrow Crossing aftermath — cashes out prologue consequence.
pub fn morrow_aftermath() -> Scene {
    scene(
        "bw_morrow_aftermath", "morrow_crossing", "3_1",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Morrow Crossing the morning after. The town's changed eyes are \
                 still there. You see them in every face that looks at you too long \
                 or not long enough.",
                EmotionTag::Tense,
            ),
            // Branch-specific entry texture
            say_if_with("narrator",
                "Tom Reed's account gives you a freight lead: medical supply \
                 routing around Black Willow was wrong before the relay failed. \
                 Wagon patterns. Load timing. The road was physically set up.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "Nella overheard names, kitchens, handoff habits. Her memory \
                 points toward Black Willow as a place where fever relief and \
                 rail paper crossed too often.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "The scorched routing fragment shows a medical consignment \
                 discrepancy tied to Black Willow. The paper trail is the \
                 strongest clue, but it costs you — people see the paper \
                 in your hands and remember the relay.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Tense,
            ),
            say_with("ada",
                "My brother was last tracked toward Black Willow. He was \
                 chasing records tied to the same chain you keep pulling at.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Ride to Black Willow", vec![], to_scene("bw_road")),
        ],
        vec![
            set_flag("ch3_started", true),
        ],
    )
}

/// Road to Black Willow — trio dynamic establishes.
pub fn road_scene() -> Scene {
    scene(
        "bw_road", "trail_to_black_willow", "3_2",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The road south turns swampy within two hours. The air thickens. \
                 Willow shade traps heat like a fist.",
                EmotionTag::Tense,
            ),
            narrate(
                "A house by the road has black cloth on the door. Nobody comes out.",
            ),
            say_with("ada",
                "Fever marker. Someone in that house is either sick or was.",
                EmotionTag::Neutral,
            ),
            say_with("eli",
                "Lot of those markers between here and Black Willow. More \
                 than last month.",
                EmotionTag::Dry,
            ),
            say_with("ada",
                "You counting them for sentiment, or because you're working \
                 out where the medicine isn't going?",
                EmotionTag::Bitter,
            ),
            say_with("eli",
                "Both. Does that bother you?",
                EmotionTag::Dry,
            ),
            say_with("ada",
                "Everything about you bothers me. That doesn't mean you're wrong.",
                EmotionTag::Bitter,
            ),
        ],
        vec![
            choice("Press on to Black Willow", vec![], to_scene("bw_district")),
        ],
        vec![],
    )
}

/// Black Willow triage district — the playable zone.
pub fn district_scene() -> Scene {
    scene(
        "bw_district", "black_willow", "3_3",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "Black Willow is not a town. It is a fever district: half-settlement, \
                 half-overflow zone built around an old willow-lined drainage ditch, \
                 a quarantine shed, a rail medicine spur, and a few stubborn houses.",
                EmotionTag::Grief,
            ),
            narrate(
                "Cloth strips on doors. Rail crates with official stamps and \
                 missing contents. Families sleeping near wagons because the \
                 shed is full.",
            ),
            say_with("ada",
                "The fever's outrun the supplies. This isn't mismanagement. \
                 Somebody redirected medicine that was supposed to be here.",
                EmotionTag::Bitter,
            ),
        ],
        vec![
            choice(
                "Help treat the sick first — bodies before clues",
                vec![
                    set_flag("treated_first", true),
                    set_flag("care_before_pursuit", true),
                ],
                to_scene("bw_triage"),
            ),
            choice(
                "Follow the sheriff's trail — the clue can't wait",
                vec![
                    set_flag("pursued_first", true),
                ],
                to_scene("bw_sheriff_trail"),
            ),
        ],
        vec![],
    )
}

/// Triage scene — treating patients costs time but earns trust and Ada's respect.
pub fn triage_scene() -> Scene {
    scene(
        "bw_triage", "black_willow_shed", "3_3",
        PacingTag::Intimate,
        vec![
            narrate(
                "The quarantine shed smells like sweat, vinegar, and the sweet \
                 rot of cloth left too long on wounds.",
            ),
            say_with("inez",
                "Sister Inez Corrow. If you're here to pray, do it quietly. \
                 If you're here to help, wash your hands first.",
                EmotionTag::Bitter,
            ),
            narrate(
                "Ada is already moving. Not posing. Her hands find the worst \
                 cases. She stabilizes a child, redresses a burn, reads a \
                 fever chart like it's evidence.",
            ),
            say_with("ada",
                "This fever pattern is not natural spread. It clusters around \
                 the rail spur. Where the crates were supposed to arrive.",
                EmotionTag::Tense,
            ),
            say_if_with("eli",
                "So somebody diverted the medicine, and the fever filled \
                 the gap. Paper before blood.",
                vec![],
                EmotionTag::Dry,
            ),
        ],
        vec![
            choice("Help Ada with the patients", vec![
                set_flag("helped_patients", true),
                relate("galen", "ada", 5),
            ], to_scene("bw_sheriff_trail")),
        ],
        vec![],
    )
}

/// Sheriff trail — following Elias Mercer's last movements.
pub fn sheriff_trail_scene() -> Scene {
    scene(
        "bw_sheriff_trail", "black_willow", "3_4",
        PacingTag::Pressure,
        vec![
            narrate(
                "The sheriff's trail is half blood, half ink. Notes in a supply \
                 ledger. Markings on a quarantine door. A route toward the pump \
                 house.",
            ),
            say_if_with("narrator",
                "The freight logic Tom described matches the supply gaps here. \
                 Wagon routes that should feed Black Willow were diverted three \
                 weeks before the fever spiked.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "Nella's overheard names appear on the quarantine shed roster. \
                 The same people who handled convoy supplies handled fever relief. \
                 The chain is human, not just paper.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "The scorched routing fragment matches a medical consignment \
                 filed through this spur. The paper trail points directly at \
                 the pump house records.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Neutral,
            ),
            say_with("junie",
                "Junie Pell, teenager, feverish herself, remembers the sheriff. \
                 He asked about the pump house. About who signs the crate manifests. \
                 He went down to the spur shed two days ago. Didn't come back \
                 the same way.",
                EmotionTag::Tense,
            ),
            // If player treated first, they have more local trust
            say_if_with("inez",
                "You helped before you asked. That counts here. The pump house \
                 — Marl Hobb keeps the supply keys. He's too tidy for a man \
                 who lost half his stock to fever.",
                vec![flag_is("treated_first", true)],
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Head to the pump house", vec![], to_combat("pump_house_hold")),
        ],
        vec![],
    )
}

/// Post-combat: Ada's join and chapter close.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "bw_chapter_close", "black_willow", "3_6",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The pump house yields what the sheriff was chasing: rail \
                 consignment records showing medicine diverted from Black Willow \
                 to a secondary depot serving the rail extension. Not lost. \
                 Redirected.",
                EmotionTag::Tense,
            ),
            narrate(
                "Sheriff Mercer's name appears in a security file. 'Inquiries \
                 — referred to regional security.' He was being tracked by the \
                 same system he was investigating.",
            ),
            say_with("ada",
                "I'm not following you because I trust you. I'm following you \
                 because the men I do trust are either missing, dead, or still \
                 filing this under procedure.",
                EmotionTag::Bitter,
            ),
            narrate_with(
                "Ada joins the party. Not as sentiment. As disgusted clarity.",
                EmotionTag::Neutral,
            ),
            // If the player treated first
            say_if_with("ada",
                "You stopped for the patients before the clue. I noticed that.",
                vec![flag_is("care_before_pursuit", true)],
                EmotionTag::Quiet,
            ),
            // If the player pursued first
            say_if_with("ada",
                "You went for the records first. I understand why. I don't \
                 have to like it.",
                vec![flag_is("pursued_first", true)],
                EmotionTag::Bitter,
            ),
        ],
        vec![],
        vec![
            StateEffect::AddPartyMember(CharacterId::new("ada")),
            set_flag("ada_joined", true),
            set_flag("ch3_complete", true),
            set_flag("sheriff_trail_found", true),
            memory("sheriff_security_file"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("sheriff_security_file"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch7")),
            },
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// Pump house hold — triage under fire. The chapter's signature set piece.
///
/// The fight starts as a containment problem: sick people panicking,
/// rail men trying to burn records, Ada refusing to leave untreated bodies.
pub fn pump_house_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("pump_house_hold"),
        phases: vec![CombatPhase {
            id: "pump_house".to_string(),
            description: "Rail-aligned men at the spur shed. Fever victims as \
                          shield and confusion. The sheriff's trail converges here.".to_string(),
            enemies: vec![
                enemy_full("rail_enforcer_a", "Rail Enforcer", 28, 22, 9, 58, 7, 15, 6),
                enemy_full("rail_enforcer_b", "Rail Enforcer", 26, 20, 8, 55, 8, 15, 6),
                // Not a real threat — a sick person panicking in the crossfire
                enemy_full("panicked_civilian", "Panicked Civilian", 8, 3, 2, 15, 4, 0, 2),
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
            name: "Black Willow Pump House".to_string(),
            cover: vec![
                CoverElement { name: "Supply crates".to_string(), durability: 30, destructible: true },
                CoverElement { name: "Pump housing".to_string(), durability: 80, destructible: false },
                CoverElement { name: "Quarantine wall".to_string(), durability: 50, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "secure_records".to_string(),
                label: "Secure the supply records".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![
                    set_flag("records_burned", true),
                ],
                success_consequence: vec![
                    set_flag("records_secured", true),
                ],
            },
            Objective {
                id: "protect_civilians".to_string(),
                label: "Protect fever patients".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![
                    set_flag("bw_civilians_harmed", true),
                ],
                success_consequence: vec![
                    set_flag("bw_civilians_safe", true),
                ],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "bw_morrow_aftermath" => Some(morrow_aftermath()),
        "bw_road" => Some(road_scene()),
        "bw_district" => Some(district_scene()),
        "bw_triage" => Some(triage_scene()),
        "bw_sheriff_trail" => Some(sheriff_trail_scene()),
        "bw_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "pump_house_hold" => Some(pump_house_encounter()),
        _ => None,
    }
}
