//! Chapter 15 — Saint's Mile Again.
//!
//! Emotional law: Testament. What is said. What is recorded.
//! What is buried. What is inherited. What the dead are forced to mean.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// Return to the place where Galen's life was written wrong.
pub fn return_to_saints_mile() -> Scene {
    scene(
        "sm_return", "saints_mile", "15_1",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Saint's Mile. The same stretch of trail. The relay rebuilt, \
                 renamed, commemorated. A plaque where bodies fell. A version \
                 of events carved into brass that makes the wrong men tragic \
                 and the right men necessary.",
                EmotionTag::Grief,
            ),
            narrate(
                "Galen stands where the poster was first nailed. The post is \
                 gone. The nail hole remains.",
            ),
            narrate_with(
                "Same ground. Older bones. One more story to tell.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Face what comes next", vec![], to_scene("sm_voss_confrontation")),
        ],
        vec![
            set_flag("ch15_started", true),
        ],
    )
}

/// The Voss confrontation — the game's father-shadow.
pub fn voss_confrontation() -> Scene {
    scene(
        "sm_voss_confrontation", "saints_mile", "15_2",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "Marshal Harlan Voss. Older. Diminished enough to be vulnerable. \
                 Powerful enough that sparing or exposing him still means something.",
                EmotionTag::Tense,
            ),
            narrate(
                "He does not rant. He does not confess. He speaks the way he \
                 always did — with the even authority of a man who believes \
                 order gives him the right to choose who gets buried beneath it.",
            ),
            say_with("voss",
                "You came back. I thought you might.",
                EmotionTag::Neutral,
            ),
            say_with("voss",
                "The territory is stable, Galen. Stable because men like me \
                 decided cleanly while everybody else was still finding words. \
                 Sound familiar?",
                EmotionTag::Neutral,
            ),
            narrate_with(
                "It does. Because he taught you that. At the shooting post. \
                 'Town like this survives because one or two men decide cleanly.' \
                 He still believes it.",
                EmotionTag::Quiet,
            ),
            // Hand injury — the shooting post callback
            say_if_with("narrator",
                "Galen's right hand rests on the holster. The grip is different \
                 now — the hand that Voss trained at that post works through a \
                 different nerve path. Still accurate. Never fast.",
                vec![flag_is("hand_wounded", true)],
                EmotionTag::Quiet,
            ),
            say_with("voss",
                "You have your truth. Your witnesses. Your paper. What do you \
                 think happens when you hand that to the territory? Do you think \
                 they build something better? Or do they just build the next \
                 version of me?",
                EmotionTag::Neutral,
            ),
            narrate_with(
                "That is his most dangerous argument. Not because it's wrong. \
                 Because it's partially right.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Continue to the final choice", vec![], to_scene("sm_final_choice")),
        ],
        vec![],
    )
}

/// The final choice — what kind of truth the world has to live with.
pub fn final_choice() -> Scene {
    scene(
        "sm_final_choice", "saints_mile", "15_3",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The question is not 'how does this end?' The question is: \
                 what version of Saint's Mile does the world carry forward?",
                EmotionTag::Tense,
            ),
            // Each ally's final truth
            say_with("ada",
                "The bodies are real. The wounds are documented. The medicine \
                 was redirected. That is not opinion. That is anatomy.",
                EmotionTag::Neutral,
            ),
            say_with("rosa",
                "The land was ours before it was theirs. The mission grants \
                 prove it. The fence held for fifteen years. That is ground.",
                EmotionTag::Bitter,
            ),
            say_with("miriam",
                "The rooms I held open are closing. The version on the plaque \
                 is the version being taught. If we don't speak now, the \
                 silence becomes the record.",
                EmotionTag::Quiet,
            ),
            say_with("eli",
                "I have the ledger. I've had it for twenty years. It says \
                 what it says. I'm done letting it sit in a coat pocket \
                 instead of a courtroom.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Kill Voss — justice, not mercy", vec![
                set_text("ending_axis", "justice"),
            ], to_scene("sm_testament")),
            choice("Expose everything — let the territory carry the uglier truth", vec![
                set_text("ending_axis", "exposure"),
            ], to_scene("sm_testament")),
            choice("Negotiate peace — spare Voss, seal some records, stop the bleeding", vec![
                set_text("ending_axis", "peace"),
            ], to_scene("sm_testament")),
            choice("Carry the burden — let the livable version stand, keep the real one private", vec![
                set_text("ending_axis", "burden"),
            ], to_scene("sm_testament")),
        ],
        vec![],
    )
}

/// Testament — what Saint's Mile means after the credits.
pub fn testament() -> Scene {
    scene_with_memory(
        "sm_testament", "saints_mile", "15_4",
        PacingTag::Intimate,
        vec![
            // Justice ending
            say_if_with("narrator",
                "Voss falls. The man who taught Galen to shoot is gone. The \
                 territory records it as frontier violence. The deeper truth \
                 gets narrower. The machine that formed Voss survives him.",
                vec![flag_eq("ending_axis", "justice")],
                EmotionTag::Grief,
            ),
            // Exposure ending
            say_if_with("narrator",
                "The full truth enters the record. Mission fire. Re-grant fraud. \
                 Payroll manipulation. Medical diversion. The territory carries \
                 the uglier version. Nobody likes the new plaque better. But \
                 nobody can pretend anymore.",
                vec![flag_eq("ending_axis", "exposure")],
                EmotionTag::Grief,
            ),
            // Peace ending
            say_if_with("narrator",
                "Voss retires. Some records are sealed. The worst is acknowledged \
                 in rooms that never open to the public. The peace holds. The \
                 silence costs. The livable version is not the true one.",
                vec![flag_eq("ending_axis", "peace")],
                EmotionTag::Quiet,
            ),
            // Burden ending
            say_if_with("narrator",
                "The plaque stands. The official version hardens. Galen carries \
                 what was real alone — or with whoever stayed close enough to \
                 hear it. The world gets stability. He gets the weight.",
                vec![flag_eq("ending_axis", "burden")],
                EmotionTag::Grief,
            ),

            // The bell — unresolved to the end
            narrate_with(
                "The bell in the valley may or may not ring as Galen leaves \
                 Saint's Mile for the last time. If it does, nobody turns around.",
                EmotionTag::Quiet,
            ),

            // The final feeling
            narrate_with(
                "We forced the world to carry more truth than it wanted. \
                 And now that truth belongs to whoever comes after us.",
                EmotionTag::Quiet,
            ),
        ],
        vec![],
        vec![
            set_flag("ch15_complete", true),
            set_flag("game_complete", true),
            memory("saints_mile_testament"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("saints_mile_testament"),
                callback_type: MemoryCallbackType::Transform,
                target_chapter: None, // no next chapter — this is the end
            },
        ],
    )
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "sm_return" => Some(return_to_saints_mile()),
        "sm_voss_confrontation" => Some(voss_confrontation()),
        "sm_final_choice" => Some(final_choice()),
        "sm_testament" => Some(testament()),
        _ => None,
    }
}
