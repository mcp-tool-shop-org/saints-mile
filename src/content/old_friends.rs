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
            // Relationship-aware reunion: warm if Galen reached for Eli in ch13
            say_if_with("eli",
                "I kept the flask. Figured you'd want it back. Or maybe I kept \
                 it because it was the last honest thing between us.",
                vec![flag_is("reaching_for_eli", true)],
                EmotionTag::Warm,
            ),
            // Cooler if the ledger was used against Rosa's interests (iron_ledger choice)
            say_if_with("eli",
                "Fifteen years is a long time to wonder if someone's coming \
                 back to settle a debt or share a road. I wasn't sure which \
                 one you'd be.",
                vec![flag_is("eli_pre_echo", true)],
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
            // Hand injury — Ada's clinical assessment after fifteen years
            say_if_with("ada",
                "The nerve path rerouted through the ulnar side. You retrained \
                 without knowing it. That's the hand talking back to the brain \
                 — it found its own road.",
                vec![flag_is("hand_wounded", true)],
                EmotionTag::Quiet,
            ),
            narrate("Clinical first. Then not clinical."),
            say_with("ada",
                "You came back. I was hoping you wouldn't have to.",
                EmotionTag::Quiet,
            ),
            // Warm reunion if Ada was prioritized during the black_willow investigation
            say_if_with("ada",
                "I kept your chart. Fifteen years of updates from a patient \
                 I never expected to see again. Professional habit. Nothing more.",
                vec![flag_is("ada_joined", true)],
                EmotionTag::Warm,
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
            // Warmer if Rosa's land was prioritized during ch4 ropehouse
            say_if_with("rosa",
                "You held the line for us once. I remember that. The fence \
                 remembers that.",
                vec![flag_is("rosa_joined", true)],
                EmotionTag::Warm,
            ),
            // Tension if the force approach was taken at the revival (ch5)
            say_if_with("rosa",
                "Last time we shared a road, you chose the fist over the \
                 voice. I haven't forgotten. But the country needs what \
                 it needs.",
                vec![flag_eq("ch5_stance", "force_hold")],
                EmotionTag::Bitter,
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
            // Warm if Miriam was trusted to speak at the revival (ch5)
            say_if_with("miriam",
                "You let me speak when the room was turning. I've carried \
                 that trust into every room since. It mattered.",
                vec![flag_eq("ch5_stance", "miriam_speaks")],
                EmotionTag::Warm,
            ),
            // Cooler if Miriam's voice was overridden
            say_if_with("miriam",
                "We've disagreed about method before. I suspect we will \
                 again. But the rooms are closing, and I'd rather disagree \
                 inside than agree from the outside.",
                vec![flag_eq("ch5_stance", "force_hold")],
                EmotionTag::Neutral,
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
            // Memory ref: last_campfire echo from ch12
            say_if_with("narrator",
                "The last time they sat together at a fire, nobody said \
                 'this is the end.' Nobody says 'this is the beginning' \
                 now, either. Some rhythms survive fifteen years. Some \
                 just echo.",
                vec![Condition::HasMemoryObject(MemoryObjectId::new("last_campfire"))],
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
