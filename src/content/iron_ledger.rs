//! Chapter 7 — Iron Ledger.
//!
//! Emotional law: Compromise. The machine indoors. Ledgers as weapons.
//! Lucien becomes necessary before forgivable.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Iron Ledger city entry — the machine indoors.
pub fn city_entry() -> Scene {
    scene(
        "il_city_entry", "iron_ledger", "7_1",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Iron Ledger: a functional rail hub city where the Briar Line's \
                 eastern corridor is administered, funded, contracted, and legally \
                 defended. Stone buildings, a telegraph exchange, two hotels, a \
                 printshop, and the Briar Line regional office where the ledgers live.",
                EmotionTag::Tense,
            ),
            narrate(
                "Your wanted poster is on the wall of the printshop on the main \
                 street. The printshop that printed it.",
            ),
            say_with("eli",
                "They wrote it all down. They always do. The trick is getting \
                 to read it before they decide what version you get to see.",
                EmotionTag::Dry,
            ),
        ],
        vec![
            choice("Read the city", vec![], to_scene("il_read_city")),
        ],
        vec![
            set_flag("ch7_started", true),
        ],
    )
}

/// Reading the city — party splits attention across targets.
pub fn read_city() -> Scene {
    scene(
        "il_read_city", "iron_ledger", "7_2",
        PacingTag::Pressure,
        vec![
            say_with("eli",
                "The Rail Hotel bar is where business happens after the office \
                 closes. Names, boasts, careless talk.",
                EmotionTag::Dry,
            ),
            say_with("ada",
                "The freight yard. Medical supply chain. Where did Black Willow's \
                 missing medicine actually go?",
                EmotionTag::Tense,
            ),
            say_with("rosa",
                "Security patrols. Formation, timing, coverage gaps.",
                EmotionTag::Neutral,
            ),
            say_with("miriam",
                "The territorial claims office. My composure and presence may \
                 gain access where Galen's face would not.",
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Approach the archive problem", vec![], to_scene("il_archive_problem")),
        ],
        vec![],
    )
}

/// The archive problem — how to get inside.
pub fn archive_problem() -> Scene {
    scene(
        "il_archive_problem", "iron_ledger_office", "7_3",
        PacingTag::Crisis,
        vec![
            narrate(
                "The critical records — demolition contracts, land-acquisition \
                 authorization, payroll connections to Saint's Mile — are in the \
                 locked section of the Briar Line archive.",
            ),
            // Party argues about approach
            say_with("eli",
                "Con the archive clerk. Social engineering. Fastest, least violent. \
                 But morally ugly — the clerk is scared, not guilty.",
                EmotionTag::Dry,
            ),
            say_with("rosa",
                "Walk in. Demand access. Hold the room if necessary. Loud. \
                 Fast. Honest.",
                EmotionTag::Bitter,
            ),
            say_with("ada",
                "Leverage the medical supply chain evidence. If we can prove \
                 the freight manifests don't match distribution records, we \
                 have legal standing for an audit.",
                EmotionTag::Neutral,
            ),
            // Lucien's contribution depends on custody status
            say_if_with("narrator",
                "Lucien, restrained and watched, says he knows the building's \
                 ventilation, wall thickness, and lock mechanisms from demolition \
                 briefings. He could get you in. If you untie his hands.",
                vec![flag_eq("lucien_status", "prisoner")],
                EmotionTag::Tense,
            ),
            say_if_with("lucien",
                "I've received briefings from this building. I know the filing \
                 logic because they generate my contracts. You need me for this \
                 and you know it.",
                vec![flag_eq("lucien_status", "forced_guide")],
                EmotionTag::Neutral,
            ),
            say_if_with("lucien",
                "The archive's filing system is organized to obscure. I can \
                 read it because I've been reading the output my whole career. \
                 Judge me after we find what's in there.",
                vec![flag_eq("lucien_status", "judged")],
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Use Eli's con — fastest way in", vec![
                set_text("archive_approach", "con"),
                relate("galen", "eli", 3),
                relate("galen", "ada", -2),
            ], to_scene("il_archive_entry")),
            choice("Use Lucien's knowledge — he knows the building", vec![
                set_text("archive_approach", "lucien"),
                relate("galen", "rosa", -5),
                relate("galen", "eli", 2),
                set_flag("used_lucien_in_archive", true),
            ], to_scene("il_archive_entry")),
            choice("Ada's legal leverage — slowest but cleanest", vec![
                set_text("archive_approach", "legal"),
                relate("galen", "ada", 5),
            ], to_scene("il_archive_entry")),
        ],
        vec![],
    )
}

/// Inside the archive — the convergence moment.
pub fn archive_entry() -> Scene {
    scene(
        "il_archive_entry", "iron_ledger_archive", "7_4",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The archive. Cedar smell, ink, paper that holds more violence \
                 per page than any canyon in the Basin.",
                EmotionTag::Tense,
            ),
            // Relay branch convergence — what the player finds
            say_if_with("narrator",
                "Route records confirm the convoy was set up along a corridor \
                 designed to fail at the relay. Tom's account was right — the \
                 road was wrong by design, not by accident.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "Payroll names include people Nella knew — convoy staff, camp \
                 workers, kitchen hands. Names that were people on the road are \
                 now line items in a budget. The lie has an accounting department.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Grief,
            ),
            say_if_with("narrator",
                "Signature comparison confirms the scorched relay documents were \
                 authentic. The archive holds the originals. The paper trail \
                 connects directly to its source.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Tense,
            ),
            // All branches see this
            narrate_with(
                "They wrote it all down. Demolition contracts, land acquisitions, \
                 payroll fraud, medical supply diversion. They wrote it all down \
                 because writing it down IS the crime.",
                EmotionTag::Grief,
            ),
            // Lucien sees himself
            say_if_with("narrator",
                "Lucien sees his own name in the files. His contracts. His blast \
                 sites. Reduced to budget lines and filing codes. The man who \
                 thought he was a professional now sees that he was an expense.",
                vec![flag_is("used_lucien_in_archive", true)],
                EmotionTag::Quiet,
            ),
            say_if_with("lucien",
                "They filed me under infrastructure maintenance.",
                vec![flag_is("used_lucien_in_archive", true)],
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Take what we can carry", vec![], to_combat("archive_break")),
        ],
        vec![
            set_flag("archive_accessed", true),
            memory("archive_convergence"),
        ],
    )
}

/// Post-archive fight: escaping the building.
pub fn archive_escape() -> Scene {
    scene(
        "il_archive_escape", "iron_ledger", "7_5",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "Security discovers the party in the archive. The fight happens \
                 in corridors, offices, a stairwell. Cover is desks, filing \
                 cabinets, and overturned chairs.",
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "Lucien collapses a doorframe to block pursuit. His demolition \
                 expertise, used defensively for the first time.",
                vec![flag_is("used_lucien_in_archive", true)],
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Get clear", vec![], to_scene("il_chapter_close")),
        ],
        vec![],
    )
}

/// Chapter close — the machine is visible.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "il_chapter_close", "iron_ledger_outskirts", "7_6",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The party escapes Iron Ledger with evidence. Not all of it. Enough. \
                 The conspiracy is institutional, not coincidental. Demolition, land \
                 acquisition, payroll fraud, and medical diversion are all connected \
                 through one authorization chain.",
                EmotionTag::Tense,
            ),
            narrate(
                "The authorization chain reaches backward past the rail, past the \
                 territorial office — toward an older set of claims. The Burned Mission.",
            ),
            // Lucien's crack deepens
            say_if_with("lucien",
                "They filed me under infrastructure maintenance.",
                vec![flag_is("used_lucien_in_archive", true)],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "He says it flat. Not guilt — disturbance. The crack from the \
                 archive is different from the crack at the depot. This one \
                 goes deeper, because it came from paper, not blast.",
                vec![flag_is("used_lucien_in_archive", true)],
                EmotionTag::Quiet,
            ),
        ],
        vec![],
        vec![
            set_flag("ch7_complete", true),
            set_flag("conspiracy_documented", true),
            set_flag("burned_mission_lead", true),
            memory("institutional_proof"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("archive_convergence"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch10")),
            },
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// The archive break — institutional fight.
pub fn archive_break_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("archive_break"),
        phases: vec![CombatPhase {
            id: "corridors".to_string(),
            description: "Fight through the Briar Line building. Cover is desks \
                          and filing cabinets.".to_string(),
            enemies: vec![
                enemy_full("security_a", "Building Security", 28, 22, 9, 55, 7, 10, 6),
                enemy_full("security_b", "Building Security", 26, 20, 8, 52, 8, 10, 6),
                enemy("security_c", "Guard", 22, 18, 7, 48, 6),
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
            name: "Briar Line Archive — Corridors".to_string(),
            cover: vec![
                CoverElement { name: "Filing cabinet".to_string(), durability: 25, destructible: true },
                CoverElement { name: "Desk".to_string(), durability: 30, destructible: true },
                CoverElement { name: "Stone doorframe".to_string(), durability: 100, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "escape_archive".to_string(),
                label: "Escape with the evidence".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![set_flag("evidence_lost", true)],
                success_consequence: vec![set_flag("evidence_secured", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "il_city_entry" => Some(city_entry()),
        "il_read_city" => Some(read_city()),
        "il_archive_problem" => Some(archive_problem()),
        "il_archive_entry" => Some(archive_entry()),
        "il_archive_escape" => Some(archive_escape()),
        "il_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "archive_break" => Some(archive_break_encounter()),
        _ => None,
    }
}
