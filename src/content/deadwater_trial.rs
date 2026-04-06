//! Chapter 10 — Deadwater Trial.
//!
//! Emotional law: Reckoning. Public truth is combat. Eli's defining act.
//! Loyalty line unlocks. Exposure without closure.

use crate::types::*;
use crate::scene::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Deadwater arrival — the room is already primed.
pub fn arrival() -> Scene {
    scene(
        "dw_arrival", "deadwater", "10_1",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Deadwater: a dried-reservoir county seat turned hearing town. \
                 Cracked plaster, heat trapped in benches, clerks with too much \
                 paper and too little courage.",
                EmotionTag::Tense,
            ),
            // Ch9 result changes the opening
            say_if_with("narrator",
                "The wire dispatch got through official channels first. The \
                 room resists dismissal. Procedure holds — for now.",
                vec![flag_eq("ch9_assignment", "rosa_lucien_signal")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "The newspaper story landed before the wire. The room is hot, \
                 volatile, leaning forward. Everyone already has an opinion.",
                vec![flag_eq("ch9_assignment", "eli_miriam_signal")],
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Prepare for the hearing", vec![], to_scene("dw_assembly")),
        ],
        vec![
            set_flag("ch10_started", true),
        ],
    )
}

/// Pre-hearing assembly — loading a weapon made of people.
pub fn assembly() -> Scene {
    scene(
        "dw_assembly", "deadwater_courthouse", "10_2",
        PacingTag::Pressure,
        vec![
            narrate(
                "The player must stage: which evidence goes first, which witness \
                 gets shielded, whether Lucien appears at all."),
            say_with("eli",
                "Sequence matters. Wrong order weakens later truth.",
                EmotionTag::Dry,
            ),
            say_with("ada",
                "The medical evidence is cleanest. Start with what they can't \
                 dispute on procedural grounds.",
                EmotionTag::Neutral,
            ),
            say_with("miriam",
                "The room's nerve is the real constraint. If we lose the crowd, \
                 the procedure follows.",
                EmotionTag::Tense,
            ),
            // Memory ref: ropehouse_damage echo from ch4
            say_if_with("rosa",
                "The Ropehouse. I still hear the benches splintering. We won \
                 the fight and lost the room. This time, the room has to hold.",
                vec![Condition::HasMemoryObject(MemoryObjectId::new("ropehouse_damage"))],
                EmotionTag::Bitter,
            ),
            // Memory ref: revival_memory echo from ch5
            say_if_with("miriam",
                "Silt Crossing taught me what a room looks like before it \
                 breaks. This one is close. But the crowd here came to listen, \
                 not to burn.",
                vec![Condition::HasMemoryObject(MemoryObjectId::new("revival_memory"))],
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Lead with medical evidence — Ada's way", vec![
                set_text("dw_sequence", "medical_first"),
            ], to_scene("dw_hearing")),
            choice("Lead with documentary chain — strongest proof", vec![
                set_text("dw_sequence", "documents_first"),
            ], to_scene("dw_hearing")),
            choice("Lead with territorial testimony — Rosa's cost", vec![
                set_text("dw_sequence", "territorial_first"),
            ], to_scene("dw_hearing")),
        ],
        vec![],
    )
}

/// The hearing opens.
pub fn hearing() -> Scene {
    scene(
        "dw_hearing", "deadwater_courthouse", "10_3",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The hearing begins. The opposition tries to narrow the issue: \
                 theft, outlaw panic, unfortunate frontier violence, isolated \
                 misconduct.",
                EmotionTag::Tense,
            ),
            narrate("The party starts breaking that frame."),
            // Sequence-specific opening
            say_if_with("ada",
                "Medical records from Black Willow, the mission, and the depot \
                 fire all show the same pattern: medicine redirected, not lost. \
                 Bodies are not paperwork.",
                vec![flag_eq("dw_sequence", "medical_first")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "The documentary chain unfolds: archive originals, relay \
                 fragments, mission land grants, demolition contracts. Each \
                 piece strengthens the next.",
                vec![flag_eq("dw_sequence", "documents_first")],
                EmotionTag::Tense,
            ),
            say_if_with("rosa",
                "My family's land. Our water. Our cattle dying while someone \
                 filed papers saying the creek was 'reallocated.'",
                vec![flag_eq("dw_sequence", "territorial_first")],
                EmotionTag::Bitter,
            ),
            // Relay branch determines which evidence categories are strongest
            say_if_with("narrator",
                "Tom Reed's freight knowledge anchors the logistics chain. \
                 Wagon routes, load weights, delivery schedules — the supply \
                 diversion is proven through physical movement, not just \
                 paper. The opposition struggles to dismiss tonnage.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "Nella's network of overheard conversations provides the \
                 human chain. Names connect to names. The clerk at Black \
                 Willow is linked to the depot manager who is linked to \
                 the filing office that reported supplies 'lost.' The \
                 opposition can challenge documents, but they cannot \
                 un-hear what was said in kitchens.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Neutral,
            ),
            say_if_with("narrator",
                "The scorched routing fragment is the strongest single piece \
                 of documentary evidence. It connects the relay fire to the \
                 medical diversion through filing numbers that could not be \
                 forged after the fact. But the opposition points to the \
                 fire itself — how convenient that only one fragment survived. \
                 The paper trail cuts both ways.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Continue the hearing", vec![], to_scene("dw_counterstrike")),
        ],
        vec![],
    )
}

/// The counterstrike — the opposition pushes back.
pub fn counterstrike() -> Scene {
    scene(
        "dw_counterstrike", "deadwater_courthouse", "10_4",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The enemy pushes back. The opposition's strategy depends on \
                 what was presented first — they attack the foundation.",
                EmotionTag::Tense,
            ),
            // Medical-first opposition: attack credentials and motive
            say_if_with("narrator",
                "The opposition counsel calls Ada's credentials into question. \
                 An unlicensed practitioner with a personal vendetta — her \
                 brother missing, her objectivity compromised. The medical \
                 evidence is reframed as grief looking for a target. The \
                 crowd shifts. Sympathy turns conditional.",
                vec![flag_eq("dw_sequence", "medical_first")],
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "The crowd murmurs. Several women who lost children to the \
                 fever stand, but the clerk waves them down. Procedure. The \
                 bodies are real, but the room treats them as exhibits, not \
                 people. Ada's hands grip the bench rail until her knuckles \
                 go white.",
                vec![flag_eq("dw_sequence", "medical_first")],
                EmotionTag::Bitter,
            ),
            // Documents-first opposition: challenge authenticity and chain of custody
            say_if_with("narrator",
                "The opposition attacks the documents themselves. Chain of \
                 custody — who held these papers, where, for how long? A \
                 fugitive, a known thief, and a woman with land claims. The \
                 documents are reframed as fabrication by interested parties. \
                 The crowd looks at the paper and sees either proof or \
                 forgery, depending on which version of the territory they \
                 already believe in.",
                vec![flag_eq("dw_sequence", "documents_first")],
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "A clerk in the third row shakes his head. He's seen these \
                 filing stamps before — he knows they're real. But he stays \
                 seated. The crowd is split between those who read and those \
                 who trust. The readers are outnumbered.",
                vec![flag_eq("dw_sequence", "documents_first")],
                EmotionTag::Quiet,
            ),
            // Territorial-first opposition: dismiss as personal grievance
            say_if_with("narrator",
                "The opposition reframes Rosa's testimony as a property \
                 dispute dressed up as justice. Land claims. Water rights. \
                 A rancher who lost a fence line and wants someone to blame. \
                 The systemic rot is reduced to a neighbor's grudge. The \
                 crowd — half of whom have their own fence-line stories — \
                 leans back. Some of them recognize the tactic. Some of \
                 them fall for it.",
                vec![flag_eq("dw_sequence", "territorial_first")],
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "Rosa's jaw sets. She's heard this before — her cost reduced \
                 to paperwork, her family's ground turned into a line item. \
                 Three ranchers in the back of the room stand and leave. \
                 Whether in solidarity or disgust, nobody can tell. The \
                 crowd thins in the wrong direction.",
                vec![flag_eq("dw_sequence", "territorial_first")],
                EmotionTag::Bitter,
            ),
            narrate(
                "The room is about to close around the wrong version.",
            ),
            say_with("narrator",
                "This is where Eli stops surviving like Eli.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Let Eli speak", vec![], to_scene("dw_eli_act")),
        ],
        vec![],
    )
}

/// Eli's defining act — the chapter's soul.
pub fn eli_act() -> Scene {
    scene(
        "dw_eli_act", "deadwater_courthouse", "10_5",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Eli steps into the room's center. He identifies himself as the \
                 man who took the ledger at Saint's Mile.",
                EmotionTag::Quiet,
            ),
            narrate(
                "He tells the truth in plain language that damages him as much \
                 as anyone else. He confirms the theft-looking survival was \
                 preservation. He names his own part without polishing it. He \
                 refuses the exit route he would once have taken.",
            ),
            say_with("eli",
                "I could've lived crooked another ten years. Don't mistake this \
                 for virtue. I'm just done letting better men wear what was mine.",
                EmotionTag::Quiet,
            ),
            // Eli's act lands differently depending on what the room just rejected
            say_if_with("narrator",
                "After the medical evidence was dismissed as grief, Eli's \
                 confession reframes it. He is not grieving. He is not a \
                 victim. He is a man who profited from the same system and \
                 is now testifying against himself. The crowd hears a \
                 co-conspirator, not an advocate. That is harder to dismiss.",
                vec![flag_eq("dw_sequence", "medical_first")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "After the documents were challenged as forgeries, Eli's \
                 testimony becomes the chain of custody the opposition \
                 demanded. He held these papers. He can describe every \
                 crease, every date, every signature he watched being \
                 written. The crowd watches a man authenticate his own \
                 damnation. That is harder to fabricate.",
                vec![flag_eq("dw_sequence", "documents_first")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "After Rosa's cost was reduced to a property dispute, Eli \
                 stands and names the property. Not fence lines — bodies. \
                 Supply routes. Medicine that went somewhere else while \
                 people died here. He turns the territorial argument from \
                 land back to lives. The three ranchers who left stop in \
                 the doorway. Two of them come back.",
                vec![flag_eq("dw_sequence", "territorial_first")],
                EmotionTag::Quiet,
            ),
            narrate_with(
                "This is the first time Eli chooses to be held by the truth \
                 instead of merely using it.",
                EmotionTag::Grief,
            ),
        ],
        vec![
            choice("The room shifts", vec![
                set_flag("eli_defining_act", true),
                unlock("eli", "stand_firm"),
                unlock("eli", "take_the_bullet"),
                set_flag("loyalty_line_unlocked", true),
            ], to_scene("dw_verdict")),
        ],
        vec![],
    )
}

/// Verdict without closure.
pub fn verdict() -> Scene {
    scene_with_memory(
        "dw_verdict", "deadwater_courthouse", "10_6",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The room shifts. Enough truth lands that the official version \
                 cannot remain untouched. Some names are exposed. Some filings \
                 are seized. Some accusations are publicly damaged.",
                EmotionTag::Tense,
            ),
            narrate(
                "But Voss is not fully captured. The institution narrows blame \
                 where it can. The machine begins adapting already.",
            ),
            narrate_with(
                "We won enough to matter, and that made everything worse in \
                 a way we chose.",
                EmotionTag::Grief,
            ),
            narrate(
                "We said it out loud. And now it cannot be unsaid.",
            ),
        ],
        vec![],
        vec![
            set_flag("ch10_complete", true),
            set_flag("public_truth_established", true),
            set_flag("voss_threatened", true),
            // Voss is NOT captured
            memory("deadwater_testimony"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("deadwater_testimony"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch15")),
            },
        ],
    )
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "dw_arrival" => Some(arrival()),
        "dw_assembly" => Some(assembly()),
        "dw_hearing" => Some(hearing()),
        "dw_counterstrike" => Some(counterstrike()),
        "dw_eli_act" => Some(eli_act()),
        "dw_verdict" => Some(verdict()),
        _ => None,
    }
}
