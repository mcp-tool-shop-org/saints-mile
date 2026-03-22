//! Chapter 13 — Fifteen Years Gone.
//!
//! Emotional law: Return. Older bones on familiar ground. The lie is
//! stronger than before. The need to step back in anyway.

use crate::types::*;
use crate::scene::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// The return — older Galen enters a Basin that narrates around him.
pub fn return_entry() -> Scene {
    scene(
        "fg_return", "cinder_basin", "13_1",
        PacingTag::Exploration,
        vec![
            narrate_with(
                "Fifteen years. The hand rests differently on the reins. The \
                 horse is different. The road is the same, but cleaner — and \
                 that's the problem.",
                EmotionTag::Quiet,
            ),
            narrate(
                "The Basin has learned to narrate around him. Renamed depots. \
                 Commemorative plaques that lie by omission. Younger officials \
                 who inherited the paperwork but not the cost.",
            ),
            narrate_with(
                "He returns because history is closing over the wound. Not to \
                 finish a revenge plot. Not to relive glory. To stop permanence \
                 from settling in the wrong shape.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("See what they made of it", vec![], to_scene("fg_official_lie")),
        ],
        vec![
            set_flag("ch13_started", true),
            StateEffect::SetFlag {
                id: FlagId::new("age_phase"),
                value: FlagValue::Text("older".to_string()),
            },
        ],
    )
}

/// The official lie — what the world says happened.
pub fn official_lie() -> Scene {
    scene(
        "fg_official_lie", "basin_town", "13_2",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "A plaque outside the rebuilt relay station: 'Saint's Mile — \
                 Site of the 18__ frontier incident. Order was restored through \
                 the diligence of territorial authorities.'",
                EmotionTag::Bitter,
            ),
            narrate(
                "Deadwater is now cited, not felt. Breakwater is remembered \
                 as disorder, not retaliation. The men who lied are becoming \
                 history's heroes.",
            ),
            say_with("young_clerk",
                "A young clerk in the territorial office knows the official \
                 story only. She has never heard Galen's name without the \
                 word 'fugitive' attached.",
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Find someone who remembers", vec![], to_scene("fg_old_witness")),
        ],
        vec![
            set_flag("official_lie_seen", true),
        ],
    )
}

/// An old witness — tired of remembering alone.
pub fn old_witness() -> Scene {
    scene(
        "fg_old_witness", "basin_town", "13_3",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "An old road worker in a boarding house that used to be a relay \
                 station. He's the kind of man who has been waiting to tell \
                 someone what he saw and has stopped believing anyone will ask.",
                EmotionTag::Quiet,
            ),
            say_with("old_worker",
                "I remember the convoy. I remember the relay. I remember the \
                 sound of it. People don't ask anymore. They read the plaque \
                 and think they know.",
                EmotionTag::Grief,
            ),
            say_if_with("old_worker",
                "Tom Reed. He made the road make sense. Nobody remembers him \
                 in the version they put on the wall.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Grief,
            ),
            say_if_with("old_worker",
                "There was a woman on that convoy. Made bad coffee and kept \
                 people alive through it. Her name's not on anything official.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Grief,
            ),
            say_if_with("old_worker",
                "The papers. Somebody had the papers. The version on the wall \
                 says they were stolen. The version I remember says they were \
                 the only thing true about any of it.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Grief,
            ),
        ],
        vec![
            choice("Ask about the people who were there", vec![], to_scene("fg_first_contact")),
        ],
        vec![
            set_flag("old_witness_found", true),
        ],
    )
}

/// First contact toward reassembly — selective, not automatic.
pub fn first_contact() -> Scene {
    scene(
        "fg_first_contact", "basin_town", "13_4",
        PacingTag::Pressure,
        vec![
            narrate(
                "The old worker mentions names. Some are dead. Some left the \
                 Basin. Some are still here, quieter, older, carrying the same \
                 weight Galen carries.",
            ),
            say_with("old_worker",
                "Eli Winter. He was around for a while after. Stayed closer \
                 than most. I heard he's still somewhere the mail can reach.",
                EmotionTag::Quiet,
            ),
            narrate(
                "Voss is still here. Not in office — but his name is on the \
                 plaque's dedication committee. His version is the one being \
                 taught.",
            ),
            narrate_with(
                "Saint's Mile itself must be revisited. The ground that started \
                 this has one more story to tell — or to be told about it.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Reach out to Eli", vec![
                set_text("first_contact", "eli"),
                set_flag("reaching_for_eli", true),
            ], to_scene("fg_chapter_close")),
            choice("Find the others first", vec![
                set_text("first_contact", "others"),
            ], to_scene("fg_chapter_close")),
        ],
        vec![],
    )
}

/// Chapter close — the return is now committed.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "fg_chapter_close", "cinder_basin", "13_5",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Galen came back too late to stop the lie from spreading. But \
                 not too late to fight what it settles into.",
                EmotionTag::Tense,
            ),
            narrate(
                "The wrong version is becoming permanent now. The men who lied \
                 are about to become history's heroes. Someone has to go back.",
            ),
        ],
        vec![],
        vec![
            set_flag("ch13_complete", true),
            set_flag("return_committed", true),
            set_flag("voss_on_plaque_committee", true),
            memory("return_necessity"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("return_necessity"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch15")),
            },
        ],
    )
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "fg_return" => Some(return_entry()),
        "fg_official_lie" => Some(official_lie()),
        "fg_old_witness" => Some(old_witness()),
        "fg_first_contact" => Some(first_contact()),
        "fg_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}
