//! Chapter 14 — Old Friends, Bad Ground.
//!
//! Emotional law: Reassembly. Enough of each other survived
//! to stand near one another one last time.

use crate::types::*;
use crate::scene::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Eli's return — the one who stayed nearest.
pub fn eli_return() -> Scene {
    scene(
        "of_eli_return", "basin_road", "14_1",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Eli is older. The charm is quieter now — not gone, just deeper. \
                 He moves like a man who decided to stop running and has been \
                 learning what that costs every day since.",
                EmotionTag::Quiet,
            ),
            say_with("eli",
                "You look like you've been deciding things. That's worse than \
                 shooting, usually.",
                EmotionTag::Dry,
            ),
            say("galen", "You came."),
            say_with("eli",
                "I was always coming. Just took the scenic route through fifteen \
                 years of trying not to.",
                EmotionTag::Quiet,
            ),
            narrate(
                "He still has the ledger. After everything. He still has it.",
            ),
        ],
        vec![
            choice("The road ahead", vec![
                StateEffect::AddPartyMember(CharacterId::new("eli")),
                set_flag("eli_returned_body", true),
            ], to_scene("of_ada_return")),
        ],
        vec![],
    )
}

/// Ada's return — through necessity and history.
pub fn ada_return() -> Scene {
    scene(
        "of_ada_return", "settlement_clinic", "14_2",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Ada's clinic is clean, small, and full. She has patients. Real \
                 patients, not the kind you pick up on a frontier campaign.",
                EmotionTag::Neutral,
            ),
            narrate(
                "She sees Galen's hand before she comments on anything else. \
                 That is Ada.",
            ),
            say_with("ada",
                "The ridge healed clean. The tendon adapted. You compensated \
                 well.",
                EmotionTag::Neutral,
            ),
            narrate("Clinical first. Then not clinical."),
            say_with("ada",
                "You came back. I was hoping you wouldn't have to.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("We need what you know", vec![
                StateEffect::AddPartyMember(CharacterId::new("ada")),
                set_flag("ada_returned_body", true),
            ], to_scene("of_rosa_return")),
        ],
        vec![],
    )
}

/// Rosa's return — conditional, through territory and duty.
pub fn rosa_return() -> Scene {
    scene(
        "of_rosa_return", "varela_country", "14_3",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Varela country. The fence held. Rosa held the fence. Fifteen \
                 years of holding, and the ground shows it.",
                EmotionTag::Neutral,
            ),
            say_with("rosa",
                "My mother died three years ago. She was still fighting the \
                 filings when she went.",
                EmotionTag::Quiet,
            ),
            say_with("rosa",
                "If you're going back to Saint's Mile, you're going through \
                 my country. Same as before. Same conditions.",
                EmotionTag::Bitter,
            ),
            say("galen", "Same conditions."),
            say_with("rosa",
                "You move when I say.",
                EmotionTag::Neutral,
            ),
            narrate_with(
                "Fifteen years, and the first thing she does is set terms. \
                 That is Rosa.",
                EmotionTag::Warm,
            ),
        ],
        vec![
            choice("Accept her terms", vec![
                StateEffect::AddPartyMember(CharacterId::new("rosa")),
                set_flag("rosa_returned_conditional", true),
            ], to_scene("of_miriam_return")),
        ],
        vec![],
    )
}

/// Miriam's return — through public memory and room-holding.
pub fn miriam_return() -> Scene {
    scene(
        "of_miriam_return", "basin_assembly", "14_4",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Miriam holds a room in a town hall that didn't exist fifteen \
                 years ago. She built it. Not with her hands — with presence, \
                 patience, and the willingness to stand where nobody else would.",
                EmotionTag::Warm,
            ),
            say_with("miriam",
                "The rooms I opened after Deadwater — some of them stayed open. \
                 Some of them are being closed again now. That's why you're here.",
                EmotionTag::Neutral,
            ),
            say_with("miriam",
                "I know what communities did with the truth we gave them. Some \
                 of them carried it. Some of them set it down. Some of them were \
                 never given the choice.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Come with us", vec![
                StateEffect::AddPartyMember(CharacterId::new("miriam")),
                set_flag("miriam_returned_body", true),
            ], to_scene("of_assembly_scene")),
        ],
        vec![],
    )
}

/// Assembly — the old party together again, changed.
pub fn assembly_scene() -> Scene {
    scene(
        "of_assembly_scene", "gathering_point", "14_5",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "They are not who they were. But they are the only ones left \
                 who can carry this correctly.",
                EmotionTag::Quiet,
            ),
            narrate(
                "Some rhythms come back instantly. Some never do. Ada still \
                 checks the hand. Rosa still sets terms. Eli still deflects \
                 with humor. Miriam still listens longer than she speaks.",
            ),
            narrate_with(
                "But the weight is different. The jokes land in different \
                 places. The silences last longer. Nobody pretends this is \
                 what it was.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("The road to Saint's Mile", vec![], to_scene("of_chapter_close")),
        ],
        vec![],
    )
}

/// Chapter close — the final approach is assembled.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "of_chapter_close", "gathering_point", "14_6",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "We are not who we were, but we are the only ones left who \
                 can carry this correctly.",
                EmotionTag::Quiet,
            ),
            narrate(
                "The final road to Saint's Mile. Same ground, older bones. \
                 One more story to tell — or to be told about.",
            ),
        ],
        vec![],
        vec![
            set_flag("ch14_complete", true),
            set_flag("party_reassembled", true),
            set_flag("saints_mile_approach", true),
            memory("reassembly_complete"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("reassembly_complete"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch15")),
            },
        ],
    )
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "of_eli_return" => Some(eli_return()),
        "of_ada_return" => Some(ada_return()),
        "of_rosa_return" => Some(rosa_return()),
        "of_miriam_return" => Some(miriam_return()),
        "of_assembly_scene" => Some(assembly_scene()),
        "of_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}
