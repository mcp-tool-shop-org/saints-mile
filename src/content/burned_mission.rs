//! Chapter 8 — The Burned Mission.
//!
//! Emotional law: Revelation. History beneath record.
//! The party is the investigative instrument.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Into the mission valley.
pub fn valley_entry() -> Scene {
    scene(
        "bm_valley_entry", "mission_valley", "8_1",
        PacingTag::Exploration,
        vec![
            narrate_with(
                "The road to the mission is older than the rail, older than the \
                 territory, older than the law that claims to own it. Stone-lined \
                 drainage, prayer markers turned into fence posts, a path designed \
                 for carts, not wagons.",
                EmotionTag::Quiet,
            ),
            narrate(
                "You are moving backward in time, not just space. The frontier \
                 has layers. This is the oldest one.",
            ),
        ],
        vec![
            choice("Approach the ruins", vec![], to_scene("bm_ruins")),
        ],
        vec![
            set_flag("ch8_started", true),
        ],
    )
}

/// The mission ruins — contested ground.
pub fn ruins_scene() -> Scene {
    scene(
        "bm_ruins", "burned_mission", "8_2",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Adobe walls reduced to shoulder height. A collapsed bell tower. \
                 A well with a newer wooden frame — the one reliable water source \
                 in this part of the basin. A small cemetery with markers worn \
                 to near-illegibility.",
                EmotionTag::Quiet,
            ),
            narrate(
                "A territorial surveyor crew camps nearby, mapping the water \
                 source. An older woman lives in a small house built from \
                 mission stone. She maintains the cemetery.",
            ),
            say_with("cordelia",
                "Cordelia Vane. I've been waiting for someone to come asking \
                 the right questions. I've decided in advance to be disappointed \
                 by whoever does.",
                EmotionTag::Dry,
            ),
        ],
        vec![
            choice("Enter the basement", vec![], to_scene("bm_basement")),
        ],
        vec![],
    )
}

/// The basement — the descent into truth.
pub fn basement_scene() -> Scene {
    scene(
        "bm_basement", "mission_basement", "8_3",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Below grade. Stone walls, partial ceiling, water damage. The air \
                 is cold in a way that doesn't match the surface temperature. \
                 Characters who spend time here become quieter afterward.",
                EmotionTag::Quiet,
            ),
            narrate(
                "What remains of the mission's paper is here: land grants, water \
                 certificates, transfer deeds, baptismal records, and a death \
                 register.",
            ),
        ],
        vec![
            choice("Read the records — each with different eyes", vec![],
                to_scene("bm_party_reads")),
        ],
        vec![],
    )
}

/// Party reads — the collective investigation.
pub fn party_reads_scene() -> Scene {
    scene(
        "bm_party_reads", "mission_basement", "8_3",
        PacingTag::Intimate,
        vec![
            // Ada reads medical
            say_with("ada",
                "Treatment patterns that predate the current fever by decades. \
                 The same water source, the same symptoms, the same communities. \
                 This place has been making people sick — or healing them — \
                 for longer than anyone alive remembers.",
                EmotionTag::Neutral,
            ),
            // Eli reads financial
            say_with("eli",
                "Financial chain shows the mission's land grant was transferred, \
                 amended, and 'lost' through territorial re-filings. The money \
                 trail predates the railroad by forty years. This isn't one \
                 crime. It's a tradition.",
                EmotionTag::Dry,
            ),
            // Galen reads land
            narrate_with(
                "The land grants prove the original claim covered everything the \
                 rail is now taking. These grants were never legally voided. \
                 They were burned.",
                EmotionTag::Tense,
            ),
            // Miriam reads death
            say_with("miriam",
                "More names in the death register than markers in the ground. \
                 Where are they? The people who are missing from the cemetery \
                 are missing from the official record too.",
                EmotionTag::Grief,
            ),
            // Cordelia fills the gaps
            say_with("cordelia",
                "They counted the dead for the books. They didn't count the \
                 dead for the ground.",
                EmotionTag::Quiet,
            ),
            say_with("cordelia",
                "I was a child when it burned. It was not a cook fire. The men \
                 who came — they knew which rooms had the paper. They burned \
                 those rooms first.",
                EmotionTag::Grief,
            ),
            // Lucien reads the fire
            say_if_with("lucien",
                "This was a job. Better than mine, but the same language. \
                 Directed ignition. Controlled enough to destroy the record \
                 rooms while leaving walls standing.",
                vec![flag_is("lucien_captured", true)],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "He says it flat. The man who demolishes things recognized \
                 the work of another man who demolished things. He is \
                 confronting a lineage he did not ask to join.",
                vec![flag_is("lucien_captured", true)],
                EmotionTag::Quiet,
            ),
            // Rosa reads the land
            say_with("rosa",
                "The well sits on the valley's anchor. My family has drawn \
                 from water like this for decades. If this grant is real — \
                 and it is — then we were never trespassing. We were inheriting.",
                EmotionTag::Warm,
            ),
        ],
        vec![
            choice("Absorb what this means", vec![], to_scene("bm_bell_moment")),
        ],
        vec![
            set_flag("mission_records_read", true),
            set_flag("historical_fraud_discovered", true),
            set_flag("lucien_reads_fire_pattern", true),
        ],
    )
}

/// The bell moment — the uncanny, unresolved.
pub fn bell_moment() -> Scene {
    scene(
        "bm_bell_moment", "burned_mission", "8_4",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The bell in the collapsed tower should not ring. It is fallen, \
                 half-buried, cracked.",
                EmotionTag::Quiet,
            ),
            narrate("It rings."),
            narrate(
                "Not loud. Not dramatic. A single tone that carries through \
                 the valley like it was always there and you only just stopped \
                 talking long enough to hear it.",
            ),
            // Each character reacts differently
            say_with("ada",
                "Acoustic resonance from the crack. The valley's shape amplifies \
                 certain frequencies.",
                EmotionTag::Neutral,
            ),
            say_with("eli", "Irrelevant. Focus on the records.", EmotionTag::Dry),
            say_with("rosa",
                "Don't stand where old things make noise.",
                EmotionTag::Tense,
            ),
            say_with("miriam",
                "...",
                EmotionTag::Quiet,
            ),
            narrate_with(
                "Miriam does not explain. She does not dismiss. She listens. \
                 The bell is speaking. What it says depends on what you bring \
                 to it.",
                EmotionTag::Quiet,
            ),
            say_if_with("lucien",
                "I've heard metal talk before. Usually means the structure's \
                 about to go.",
                vec![flag_is("lucien_captured", true)],
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Continue", vec![
                set_flag("bell_heard", true),
            ], to_combat("mission_defense")),
        ],
        vec![],
    )
}

/// Chapter close — the ground has memory.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "bm_chapter_close", "mission_valley", "8_6",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The party leaves the mission with the evidence. Behind them, \
                 the ruins remain. The bell may or may not ring as they go.",
                EmotionTag::Quiet,
            ),
            narrate(
                "The machine did not create the lie. The machine inherited it. \
                 And the people who built the machine knew exactly what they \
                 were building on top of.",
            ),
            narrate_with(
                "The lie didn't start with the railroad. The railroad just \
                 learned the language of something older.",
                EmotionTag::Grief,
            ),
        ],
        vec![],
        vec![
            set_flag("ch8_complete", true),
            set_flag("mission_truth_recovered", true),
            set_flag("fire_was_deliberate", true),
            set_flag("regrant_was_fraud", true),
            memory("mission_land_grants"),
            memory("death_register_discrepancy"),
            memory("bell_phenomenon"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("bell_phenomenon"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch15")),
            },
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// Mission defense — enforcement team arrives.
pub fn mission_defense_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("mission_defense"),
        phases: vec![CombatPhase {
            id: "ruins_fight".to_string(),
            description: "Fight in and around the mission ruins. Partial walls, \
                          cemetery, well area.".to_string(),
            enemies: vec![
                enemy_full("enforcer_a", "Enforcement Agent", 30, 24, 10, 58, 7, 12, 7),
                enemy_full("enforcer_b", "Enforcement Agent", 28, 22, 9, 55, 8, 12, 6),
                enemy("enforcer_c", "Enforcement Agent", 26, 20, 8, 52, 7),
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
            name: "Burned Mission Ruins".to_string(),
            cover: vec![
                CoverElement { name: "Adobe wall section".to_string(), durability: 45, destructible: true },
                CoverElement { name: "Bell tower base".to_string(), durability: 70, destructible: false },
                CoverElement { name: "Well housing".to_string(), durability: 60, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "defend_records".to_string(),
                label: "Defend the mission records".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![set_flag("mission_records_destroyed", true)],
                success_consequence: vec![set_flag("mission_records_defended", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "bm_valley_entry" => Some(valley_entry()),
        "bm_ruins" => Some(ruins_scene()),
        "bm_basement" => Some(basement_scene()),
        "bm_party_reads" => Some(party_reads_scene()),
        "bm_bell_moment" => Some(bell_moment()),
        "bm_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "mission_defense" => Some(mission_defense_encounter()),
        _ => None,
    }
}
