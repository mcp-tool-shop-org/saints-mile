//! Chapter 9 — The Long Wire.
//!
//! Emotional law: Fracture. The party splits. Trust tested by distance.
//! The assignment IS the argument.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::combat::split_party;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Breakwater Junction arrival — the wire goes hot.
pub fn junction_arrival() -> Scene {
    scene(
        "lw_junction_arrival", "breakwater_junction", "9_1",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "Breakwater Junction: a switch town built around a telegraph \
                 office, a relay shed, freight sidings, and a payroll annex. \
                 The wire hums in dry wind.",
                EmotionTag::Tense,
            ),
            narrate(
                "A sealed transmission packet is scheduled to leave before \
                 nightfall — the Briar Line's official version of events that \
                 frames the party as fugitive archive raiders.",
            ),
            say_with("eli",
                "Three targets. Not enough people. Not enough time.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Split the party", vec![], to_scene("lw_assignment")),
        ],
        vec![
            set_flag("ch9_started", true),
        ],
    )
}

/// The assignment scene — the argument expressed through deployment.
pub fn assignment_scene() -> Scene {
    scene(
        "lw_assignment", "breakwater_junction", "9_1",
        PacingTag::Crisis,
        vec![
            narrate(
                "Three objectives, six people, not enough of either."),
            narrate(
                "Wire office: hold the telegraph and transmit the party's version. \
                 Signal tower: secure the relay and delay the counter-narrative. \
                 Witness route: protect the surviving witness and their testimony."),
            // The party voices their positions
            say_with("rosa",
                "Hold the line physically. Stop the enemy from moving anything.",
                EmotionTag::Bitter,
            ),
            say_with("ada",
                "Protect the living witnesses and operators. A dead witness is \
                 a burned record with blood on it.",
                EmotionTag::Tense,
            ),
            say_with("miriam",
                "Stabilize the room and keep panic from doing the enemy's work.",
                EmotionTag::Neutral,
            ),
            say_with("eli",
                "Change the message or redirect the authority chain. The cleanest \
                 win is making the lie fail inside its own system.",
                EmotionTag::Dry,
            ),
            say_if_with("lucien",
                "Break the line. Collapse the route. Stop the packet by force.",
                vec![flag_is("lucien_captured", true)],
                EmotionTag::Neutral,
            ),
        ],
        vec![
            // Assignment A: Galen+Ada wire, Rosa+Lucien signal, Eli+Miriam witness
            choice("Send Rosa and Lucien to the signal tower", vec![
                set_text("ch9_assignment", "rosa_lucien_signal"),
            ], to_scene("lw_split_execution")),
            // Assignment B: Galen+Rosa wire, Eli+Miriam signal, Ada+Lucien witness
            choice("Send Eli and Miriam to the signal tower", vec![
                set_text("ch9_assignment", "eli_miriam_signal"),
            ], to_scene("lw_split_execution")),
            // Assignment C: Galen+Eli wire, Rosa+Ada signal, Miriam+Lucien witness
            choice("Send Miriam and Lucien to the witness route", vec![
                set_text("ch9_assignment", "miriam_lucien_witness"),
            ], to_scene("lw_split_execution")),
        ],
        vec![],
    )
}

/// Split execution — results come in.
pub fn split_execution() -> Scene {
    scene(
        "lw_split_execution", "breakwater_junction", "9_3",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The teams separate. The party becomes smaller. Who looked back, \
                 who didn't, who said something and who was silent.",
                EmotionTag::Quiet,
            ),
            // Generic — actual team results are applied via the split-party system
            narrate(
                "Hours pass. Reports come in. Not all of them good. Not all of \
                 them clean.",
            ),
        ],
        vec![
            choice("Receive the reports", vec![], to_scene("lw_reports")),
        ],
        vec![],
    )
}

/// Reports — team results as relationship updates.
pub fn reports_scene() -> Scene {
    scene(
        "lw_reports", "breakwater_junction", "9_4",
        PacingTag::Intimate,
        vec![
            // Assignment-specific report lines
            say_if_with("narrator",
                "Rosa held Pine Signal. Lucien disarmed the mast charge. \
                 Rosa has not thanked him. She won't.",
                vec![flag_eq("ch9_assignment", "rosa_lucien_signal")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Miriam and Eli kept the witness safe. Neither agrees on how.",
                vec![flag_eq("ch9_assignment", "rosa_lucien_signal")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Eli and Miriam held the signal tower through rhetoric and \
                 presence. The operator transmitted because Miriam asked, not \
                 because Eli threatened.",
                vec![flag_eq("ch9_assignment", "eli_miriam_signal")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Miriam and Lucien secured the witness. Miriam stood in front \
                 of damage without endorsing it. Lucien held the physical line \
                 without being asked. Neither spoke about it after.",
                vec![flag_eq("ch9_assignment", "miriam_lucien_witness")],
                EmotionTag::Quiet,
            ),
            // Eli pre-echo — he comes back different
            say_with("narrator",
                "Eli comes back quieter. Or steadier. You can't tell which. \
                 Something shifted during separation, and he hasn't named it.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Regroup for what comes next", vec![
                set_flag("eli_pre_echo", true),
            ], to_scene("lw_chapter_close")),
        ],
        vec![],
    )
}

/// Chapter close — partial truth in circulation.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "lw_chapter_close", "breakwater_junction", "9_5",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The party succeeds partially. One truth gets through. One truth \
                 is delayed. One lie still spreads. They regroup physically, \
                 but not emotionally intact.",
                EmotionTag::Tense,
            ),
            narrate(
                "We stopped them from fixing the whole lie tonight. But now the \
                 lie and the truth are both loose in the world.",
            ),
            narrate_with(
                "Chapter 10 will have to happen because partial truth is now \
                 in circulation.",
                EmotionTag::Tense,
            ),
        ],
        vec![],
        vec![
            set_flag("ch9_complete", true),
            set_flag("partial_truth_transmitted", true),
            set_flag("deadwater_necessary", true),
            memory("transmission_result"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("transmission_result"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch10")),
            },
        ],
    )
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "lw_junction_arrival" => Some(junction_arrival()),
        "lw_assignment" => Some(assignment_scene()),
        "lw_split_execution" => Some(split_execution()),
        "lw_reports" => Some(reports_scene()),
        "lw_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}
