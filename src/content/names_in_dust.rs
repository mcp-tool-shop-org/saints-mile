//! Chapter 12 — Names in the Dust.
//!
//! Emotional law: Dispersal. Truth survives. The group doesn't.
//! The adult road ends. Not in defeat, not in triumph. In residue.

use crate::types::*;
use crate::scene::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Post-Breakwater aftermath — the party at rest, spent.
pub fn aftermath() -> Scene {
    scene(
        "nd_aftermath", "breakwater_outskirts", "12_1",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The days after Breakwater. Treatment, repair, inventory, exhaustion. \
                 The party is still together but the purpose that held them is \
                 changing shape.",
                EmotionTag::Quiet,
            ),
            narrate(
                "The truth is out. The enemy is wounded. The road ahead is no \
                 longer clear.",
            ),
        ],
        vec![
            choice("Face what comes next", vec![], to_scene("nd_separations")),
        ],
        vec![
            set_flag("ch12_started", true),
        ],
    )
}

/// The separations — one by one, the party finds reasons to go.
pub fn separations() -> Scene {
    scene(
        "nd_separations", "breakwater_outskirts", "12_2",
        PacingTag::Intimate,
        vec![
            // Ada
            say_with("ada",
                "I have patients who aren't you. That's not a rejection. It's \
                 what winning looks like when nobody's shooting.",
                EmotionTag::Quiet,
            ),
            // Rosa
            say_with("rosa",
                "Fence doesn't hold itself. And Ma won't say she's tired, which \
                 means she's tired.",
                EmotionTag::Quiet,
            ),
            // Miriam
            say_with("miriam",
                "The room doesn't stay open by itself. Somebody has to be there \
                 when the talking matters. That's what I'm for.",
                EmotionTag::Neutral,
            ),
            // Lucien — the most ambiguous exit
            say_if_with("narrator",
                "Lucien is gone one morning. No speech. A fuse lighter left \
                 where Galen will find it.",
                vec![flag_is("lucien_captured", true)],
                EmotionTag::Quiet,
            ),
            // Eli — stays longest
            say_with("eli",
                "I'll be around. Not far, not close. Just somewhere the mail \
                 can find me if you need a worse idea than your own.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Accept the distance", vec![], to_scene("nd_campfire")),
        ],
        vec![
            set_flag("ada_departed", true),
            set_flag("rosa_departed", true),
            set_flag("miriam_departed", true),
            set_flag("eli_nearest", true),
        ],
    )
}

/// The last campfire — the most important one in the game.
pub fn last_campfire() -> Scene {
    scene(
        "nd_campfire", "campfire", "12_3",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "A ridge, or a creek bed, or a stretch of road. The fire is small. \
                 The sky is large. The sounds are the ones the game has been building \
                 for twelve chapters: wind, horse, creak, flame, breath.",
                EmotionTag::Quiet,
            ),
            narrate(
                "Nobody says 'this is the end.' Because nobody at a real campfire does.",
            ),
            narrate(
                "Ada checks Galen's hand without being asked. Routine now. Rosa \
                 looks at the sky. Miriam hums something. Eli produces a flask.",
            ),
            say_with("eli",
                "Funny thing about roads. They all go somewhere until you stop. \
                 Then they just go.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Let the fire burn down", vec![], to_scene("nd_last_road")),
        ],
        vec![
            memory("last_campfire"),
        ],
    )
}

/// The last road — the adult arc ends.
pub fn last_road() -> Scene {
    scene_with_memory(
        "nd_last_road", "open_road", "12_4",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "After the separations, after the campfire, Galen rides. Not toward \
                 something. Not away from something. Just riding.",
                EmotionTag::Quiet,
            ),
            narrate(
                "His hand rests differently on the reins. The player should notice.",
            ),
            narrate_with(
                "The road. The light. The distance.",
                EmotionTag::Quiet,
            ),
            narrate("Fifteen years later."),
        ],
        vec![],
        vec![
            set_flag("ch12_complete", true),
            set_flag("adult_arc_complete", true),
            // Truth stratification flags
            set_flag("public_truth_partial", true),
            set_flag("poster_in_limbo", true),
            set_flag("voss_still_free", true),
            // Party dispersal state
            set_flag("party_dispersed", true),
            // Hand state transition
            StateEffect::SetFlag {
                id: FlagId::new("hand_state_transition"),
                value: FlagValue::Text("adult_to_older".to_string()),
            },
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("last_campfire"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch14")),
            },
        ],
    )
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "nd_aftermath" => Some(aftermath()),
        "nd_separations" => Some(separations()),
        "nd_campfire" => Some(last_campfire()),
        "nd_last_road" => Some(last_road()),
        _ => None,
    }
}
