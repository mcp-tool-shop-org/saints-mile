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
            // Hand injury — fifteen years of adaptation
            say_if_with("narrator",
                "The hand has adapted. He holds the reins differently — ring \
                 and little finger do the grip work, index and middle guide. \
                 Fifteen years of compensation have made the workaround invisible \
                 to anyone who didn't know him before.",
                vec![flag_is("hand_wounded", true)],
                EmotionTag::Quiet,
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
/// The witness who surfaces depends on the relay branch — who survived
/// fifteen years ago determines who is still around to remember.
pub fn old_witness() -> Scene {
    scene(
        "fg_old_witness", "basin_town", "13_3",
        PacingTag::Intimate,
        vec![
            // Tom branch: the old road worker who learned freight from Tom
            say_if_with("narrator",
                "An old road worker in a boarding house that used to be a relay \
                 station. His hands still move like a man counting cargo — \
                 thumb to fingers, tallying loads that no longer exist. He \
                 learned the freight trade from Tom Reed, and he has been \
                 waiting fifteen years for someone to ask about the routes \
                 that didn't add up.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Quiet,
            ),
            say_if_with("old_worker",
                "Tom Reed. He made the road make sense. The wagon weights, \
                 the timing, the way a full load handles differently from \
                 an empty one painted to look full. Nobody remembers him \
                 in the version they put on the wall. But every driver who \
                 worked this spur for the next ten years knew his name.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Grief,
            ),
            say_if_with("old_worker",
                "I kept his route maps. Figured somebody would come asking \
                 eventually. The roads changed, but the routes he marked — \
                 the ones where cargo disappeared — those are still the \
                 same roads. You can still drive them and count the gaps.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Quiet,
            ),
            // Nella branch: a woman from Nella's kitchen network
            say_if_with("narrator",
                "A woman running a boarding house kitchen — the kind of place \
                 where travelers talk and the cook remembers. She knew Nella. \
                 She was part of the network of kitchens and back rooms where \
                 names were whispered and connections were made without anyone \
                 writing anything down.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Quiet,
            ),
            say_if_with("old_worker",
                "Nella. She made bad coffee and kept people alive through it. \
                 Her name's not on anything official. But every kitchen between \
                 here and Morrow Crossing passed messages through her hands \
                 for three years after the relay. When they finally stopped \
                 asking, she stopped answering. But she never stopped listening.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Grief,
            ),
            say_if_with("old_worker",
                "I still hear things in this kitchen. Names that connect to \
                 names. The men who did the diverting — some of them still \
                 eat here. They don't know I know. Nella taught me that: \
                 the best witness is the one nobody notices pouring coffee.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Quiet,
            ),
            // Papers branch: a former filing clerk haunted by what he processed
            say_if_with("narrator",
                "A retired filing clerk in a room above the old territorial \
                 office. His desk is covered in newspapers, each one folded \
                 to a story about the Basin. He has been cross-referencing \
                 the official record against his own memory for fifteen years, \
                 marking every lie in the margin with a pencil.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Quiet,
            ),
            say_if_with("old_worker",
                "The papers. Somebody had the papers. The version on the wall \
                 says they were stolen. The version I remember says they were \
                 the only thing true about any of it. I filed the replacement \
                 documents myself. I know what was changed because I held \
                 both versions in my hands on the same afternoon.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Grief,
            ),
            say_if_with("old_worker",
                "I kept copies. Not the originals — those are long gone or \
                 locked up. But I copied the filing numbers, the dates, the \
                 signatures. Fifteen years of margins full of pencil marks. \
                 Nobody asked. I stopped expecting them to. But I never \
                 stopped marking.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Quiet,
            ),
            // Common closing
            say_with("old_worker",
                "I remember the convoy. I remember the relay. I remember the \
                 sound of it. People don't ask anymore. They read the plaque \
                 and think they know.",
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
            ], to_scene("fg_eli_first")),
            choice("Find the others first", vec![
                set_text("first_contact", "others"),
            ], to_scene("fg_others_first")),
        ],
        vec![],
    )
}

/// Eli-first branch — an intermediate scene showing what fifteen years did to him.
pub fn eli_first() -> Scene {
    scene(
        "fg_eli_first", "basin_town", "13_4a",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The letter takes three weeks. The reply comes in Eli's \
                 handwriting — older, smaller, the letters pressed harder \
                 into the paper as if he's fighting the pen.",
                EmotionTag::Quiet,
            ),
            narrate(
                "They meet at a rail depot cafe two towns over. Eli chose \
                 the location — public enough to be safe, quiet enough to \
                 talk. He sits in the corner facing the door. Some habits \
                 never change.",
            ),
            narrate_with(
                "Fifteen years have carved Eli down. He is thinner, quieter, \
                 more careful with his words. The easy charm is gone — replaced \
                 by something watchful. He still has the ledger. It sits on \
                 the table between them like a third person at the meal.",
                EmotionTag::Grief,
            ),
            say_with("eli",
                "I kept moving for the first five years. Then I stopped. \
                 Settled into a place where nobody knew the name Winter \
                 and nobody asked why I slept with a book under my coat.",
                EmotionTag::Quiet,
            ),
            say_with("eli",
                "The ledger. I read it once a year. On the anniversary. \
                 Then I put it back and try to live like a man who doesn't \
                 carry a dead town's receipts in his pocket.",
                EmotionTag::Grief,
            ),
            say_with("eli",
                "You came back because the lie is hardening. I know. I \
                 watched it set from the outside. The plaque. The school \
                 dedication. Voss on the committee. Every year the wrong \
                 version gets heavier.",
                EmotionTag::Bitter,
            ),
            narrate_with(
                "Eli's hands are steady. His eyes are not. Fifteen years of \
                 carrying truth alone have made him precise and brittle. He \
                 will break clean if he breaks at all.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("We finish this together", vec![
                relate("galen", "eli", 5),
            ], to_scene("fg_chapter_close")),
        ],
        vec![],
    )
}

/// Others-first branch — Rosa or Ada's perspective before reaching Eli.
pub fn others_first() -> Scene {
    scene(
        "fg_others_first", "basin_town", "13_4b",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Galen finds Rosa first. She is still on her land — what's \
                 left of it. The fence line has been redrawn twice by \
                 territorial surveyors. She let them. She kept farming \
                 inside whatever line they gave her.",
                EmotionTag::Quiet,
            ),
            say_with("rosa",
                "I stopped fighting the fence fifteen years ago. I started \
                 farming inside it. The land doesn't care whose name is \
                 on the grant. The water still runs the same direction.",
                EmotionTag::Bitter,
            ),
            say_with("rosa",
                "Ada writes me every spring. She's still practicing — not \
                 licensed, never will be, but the families out here don't \
                 care about territorial stamps. They care about who shows \
                 up when the fever comes.",
                EmotionTag::Quiet,
            ),
            narrate(
                "Ada arrives two days later. She is older but not diminished. \
                 Her medical bag is newer. Her diagnostic eye is sharper. \
                 She has spent fifteen years treating people the territory \
                 forgot, and it has made her harder and more necessary.",
            ),
            say_with("ada",
                "Eli. He's still out there. I hear from him once a year — \
                 a letter with no return address and too much detail about \
                 nothing. He's still carrying it. We all are. But he's \
                 carrying it alone, and that's the problem.",
                EmotionTag::Grief,
            ),
            narrate_with(
                "Through Rosa and Ada, Galen sees what fifteen years of \
                 private truth looks like. It is not dramatic. It is \
                 quiet, persistent, and exhausting. The official lie takes \
                 no effort to maintain. The truth takes everything.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Send word to Eli together", vec![
                relate("galen", "rosa", 3),
                relate("galen", "ada", 3),
            ], to_scene("fg_chapter_close")),
        ],
        vec![],
    )
}

/// Chapter close — the return is now committed.
/// Emotional tenor varies based on who Galen reached first.
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
            // Eli-first close: the weight is shared between two men who
            // understand each other's damage
            say_if_with("narrator",
                "Eli's presence changes the arithmetic. Two men carrying the \
                 same weight is not half as heavy — it is twice as dangerous. \
                 Eli knows the ledger. Galen knows the ground. Between them, \
                 they hold enough truth to rewrite the plaque. Whether the \
                 territory will let them is another question entirely.",
                vec![flag_eq("first_contact", "eli")],
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "The return feels like reconnaissance. Galen and Eli move \
                 through the Basin like men casing a building they intend \
                 to enter by force. The tenderness is underneath — in the \
                 pauses, in the things they do not say about the years \
                 between. The mission comes first. The grief waits.",
                vec![flag_eq("first_contact", "eli")],
                EmotionTag::Quiet,
            ),
            // Others-first close: the return is grounded in community,
            // not just purpose
            say_if_with("narrator",
                "Rosa and Ada have already begun the work. Not the dramatic \
                 kind — the slow, daily kind. Patients treated, fence lines \
                 maintained, letters written to a man who never writes back \
                 his address. Galen's return does not start the fight. It \
                 gives the fight a name and a direction.",
                vec![flag_eq("first_contact", "others")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "The return feels like a homecoming that nobody planned. \
                 Three people who carried the truth separately discover \
                 they were carrying the same shape. The grief is older \
                 and more familiar. The purpose is steadier. Eli is still \
                 out there, and now there are enough voices to make the \
                 asking worth his answer.",
                vec![flag_eq("first_contact", "others")],
                EmotionTag::Warm,
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
        "fg_eli_first" => Some(eli_first()),
        "fg_others_first" => Some(others_first()),
        "fg_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}
