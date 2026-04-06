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
            // ── Justice ending ──────────────────────────────────────
            say_if_with("narrator",
                "Voss falls at the post where he taught Galen to shoot. The \
                 same ground, the same angle of light. The man who decided \
                 cleanly is decided upon. His hand reaches for nothing. \
                 Galen's hand does not shake.",
                vec![flag_eq("ending_axis", "justice")],
                EmotionTag::Grief,
            ),
            say_if_with("narrator",
                "The territory records it as frontier violence — an old feud \
                 between a marshal and a fugitive, resolved the way such things \
                 are resolved out here. The deeper truth gets narrower. The \
                 names Voss protected never appear in the filing. The system \
                 that shaped him, funded him, armed him with procedure and \
                 silence — that system buries one man and promotes the next.",
                vec![flag_eq("ending_axis", "justice")],
                EmotionTag::Grief,
            ),
            say_if_with("narrator",
                "Ada closes her brother's file. Rosa walks her fence line \
                 without looking over her shoulder. Eli sleeps without the \
                 ledger under his coat for the first time in twenty years. \
                 Miriam keeps the rooms open one more season. They each carry \
                 a private closure that the public record will never grant.",
                vec![flag_eq("ending_axis", "justice")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "The machine that formed Voss survives him. It always does. \
                 But somewhere in a pump house or a courthouse or a fever \
                 shed, a younger clerk will find a gap in the paperwork and \
                 wonder who made it. That wondering is the only inheritance \
                 Galen can leave.",
                vec![flag_eq("ending_axis", "justice")],
                EmotionTag::Quiet,
            ),
            // Justice + relay branch callbacks
            say_if_with("narrator",
                "Tom's structural proof — the engineering that proved the \
                 relay was built to fail — sits in a file nobody requested. \
                 The math survived. The man who did the math did not make \
                 the front page.",
                vec![flag_eq("ending_axis", "justice"), flag_eq("relay_branch", "tom")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Nella's testimony — the human witness who saw the relay \
                 burn and named the men who lit it — becomes a footnote in \
                 a violence report. She told the truth. The truth outlived \
                 the man she told it about.",
                vec![flag_eq("ending_axis", "justice"), flag_eq("relay_branch", "nella")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "The relay papers — transfer orders, payroll ghosts, the \
                 paper trail that proved the fire was policy — sit in \
                 Galen's coat. They proved who gave the order. They did \
                 not prove who would care.",
                vec![flag_eq("ending_axis", "justice"), flag_eq("relay_branch", "papers")],
                EmotionTag::Quiet,
            ),

            // ── Exposure ending ───────────────────────────────────────
            say_if_with("narrator",
                "The full truth enters the record. Mission fire. Re-grant \
                 fraud. Payroll manipulation. Medical diversion. Rail \
                 consignment forgery. Names, dates, signatures. Every \
                 document Eli carried, every wound Ada catalogued, every \
                 fence line Rosa measured — all of it filed, stamped, and \
                 made permanent in territorial archives.",
                vec![flag_eq("ending_axis", "exposure")],
                EmotionTag::Grief,
            ),
            say_if_with("narrator",
                "Nobody likes the new plaque better. The territory rewrites \
                 Saint's Mile as a site of institutional failure, which is \
                 accurate and insufficient. The families who lost land get \
                 procedural acknowledgment but not acreage. The families who \
                 lost people get a line in an appendix. The clerk who filed \
                 the original lies retires on schedule.",
                vec![flag_eq("ending_axis", "exposure")],
                EmotionTag::Bitter,
            ),
            say_if_with("narrator",
                "Voss is stripped of rank and title. He lives another eight \
                 years in a boarding house two counties over, reading the \
                 territorial newspaper, watching his version lose ground \
                 sentence by sentence. He does not recant. He does not \
                 apologize. He simply becomes irrelevant, which is the one \
                 thing he never prepared for.",
                vec![flag_eq("ending_axis", "exposure")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "The territory carries the uglier version now. Children learn \
                 the corrected history in schools built on re-granted land. \
                 Nobody can pretend anymore, and that turns out to be a heavier \
                 gift than anyone expected. Truth does not console. It only \
                 prevents the specific lie that was told before.",
                vec![flag_eq("ending_axis", "exposure")],
                EmotionTag::Grief,
            ),
            // Exposure + relay branch callbacks
            say_if_with("narrator",
                "Tom's engineering report becomes Exhibit 14 in the territorial \
                 filing. The structural proof — load calculations, material \
                 failures, the mathematics of deliberate sabotage — enters \
                 the permanent record. The relay was built to burn. Now \
                 everyone knows the formula.",
                vec![flag_eq("ending_axis", "exposure"), flag_eq("relay_branch", "tom")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Nella testifies in open session. Her account of the relay \
                 fire — the faces, the timing, the smell of kerosene on \
                 uniforms — becomes the human center of an institutional \
                 indictment. She named the men. The territory recorded the names.",
                vec![flag_eq("ending_axis", "exposure"), flag_eq("relay_branch", "nella")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "The relay papers are published in full. Transfer orders, \
                 payroll manipulation, the administrative choreography of \
                 arson — every page stamped and archived. The fire had a \
                 filing system. Now everyone can read it.",
                vec![flag_eq("ending_axis", "exposure"), flag_eq("relay_branch", "papers")],
                EmotionTag::Quiet,
            ),

            // ── Peace ending ──────────────────────────────────────────
            say_if_with("narrator",
                "The negotiation takes three days in a room with no windows. \
                 Voss retires with a pension. Some records are sealed for \
                 twenty years. The mission land grants are partially restored \
                 — enough to matter, not enough to be whole. Rosa gets her \
                 fence line back but not the years behind it.",
                vec![flag_eq("ending_axis", "peace")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "The worst of it is acknowledged in rooms that never open to \
                 the public. Ada's medical evidence becomes a confidential \
                 appendix. Eli's ledger is cited but not published. Miriam's \
                 testimony is filed under a classification that means nobody \
                 will read it until the people it describes are already dead.",
                vec![flag_eq("ending_axis", "peace")],
                EmotionTag::Bitter,
            ),
            say_if_with("narrator",
                "The peace holds. The bleeding stops. Families go back to \
                 living in a territory that wronged them and then agreed to \
                 stop. The contamination remains in the soil, in the water \
                 table, in the way officials pause before answering certain \
                 questions. A livable compromise is not the same thing as \
                 justice. Everyone at the table knows this. Everyone signs \
                 anyway.",
                vec![flag_eq("ending_axis", "peace")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Galen rides out with a settlement that tastes like ash and \
                 functions like mercy. The livable version is not the true \
                 one. But people live in it, and some of them live better \
                 than they did before. That has to be enough. It is not \
                 enough. It has to be.",
                vec![flag_eq("ending_axis", "peace")],
                EmotionTag::Grief,
            ),
            // Peace + relay branch callbacks
            say_if_with("narrator",
                "Tom's structural proof is filed under a classification \
                 that means 'true but inconvenient.' The engineering that \
                 proved the relay was sabotage becomes a sealed appendix. \
                 The math is correct. The math is confidential.",
                vec![flag_eq("ending_axis", "peace"), flag_eq("relay_branch", "tom")],
                EmotionTag::Bitter,
            ),
            say_if_with("narrator",
                "Nella's testimony is taken in a closed room and never \
                 repeated in an open one. She named the men who lit the \
                 relay. Their names appear in a document nobody will read \
                 until the men are already dead. She told the truth to a \
                 room that agreed to forget it politely.",
                vec![flag_eq("ending_axis", "peace"), flag_eq("relay_branch", "nella")],
                EmotionTag::Bitter,
            ),
            say_if_with("narrator",
                "The relay papers are cited in the settlement but not \
                 published. The transfer orders, the payroll ghosts — \
                 referenced by number, never by content. The fire had \
                 a paper trail. The peace agreement has a longer one.",
                vec![flag_eq("ending_axis", "peace"), flag_eq("relay_branch", "papers")],
                EmotionTag::Bitter,
            ),

            // ── Burden ending ─────────────────────────────────────────
            say_if_with("narrator",
                "The plaque stands. The official version hardens into the \
                 kind of history that sounds true because nobody alive \
                 contradicts it loudly enough. Galen watches it set like \
                 concrete and says nothing. The real version stays in his \
                 coat pocket, in Eli's ledger, in Ada's medical notes that \
                 nobody requested.",
                vec![flag_eq("ending_axis", "burden")],
                EmotionTag::Grief,
            ),
            say_if_with("narrator",
                "The world gets the comfortable version: a frontier incident, \
                 regrettable but contained, resolved by the diligence of \
                 territorial authorities. Voss ages into respectability. His \
                 name appears on a school dedication. His grandson learns a \
                 version of history that makes the old man heroic. The lie \
                 becomes inherited.",
                vec![flag_eq("ending_axis", "burden")],
                EmotionTag::Bitter,
            ),
            say_if_with("narrator",
                "Galen carries what was real alone — or with whoever stayed \
                 close enough to hear it. Eli still has the ledger. Ada \
                 still has the wound charts. Rosa still knows where the \
                 fence was. Miriam still remembers every name that came \
                 through her door. They carry the truth like a second \
                 skeleton, private and load-bearing.",
                vec![flag_eq("ending_axis", "burden")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "The world gets stability. Galen gets the weight. In twenty \
                 years, when the last witness dies, the real version will \
                 exist only in objects: a ledger, a chart, a land survey, \
                 a coat pocket full of paper that means everything to nobody. \
                 Unless someone finds it. Unless someone asks.",
                vec![flag_eq("ending_axis", "burden")],
                EmotionTag::Grief,
            ),
            // Burden + relay branch callbacks
            say_if_with("narrator",
                "Tom's structural calculations stay in Galen's coat beside \
                 the land grants. The proof that the relay was engineered \
                 to fail — load numbers, material specs, the geometry of \
                 deliberate collapse — waits for someone who knows how to \
                 read it. Mathematics is patient.",
                vec![flag_eq("ending_axis", "burden"), flag_eq("relay_branch", "tom")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Nella is still alive. Still remembers the faces at the \
                 relay. Still carries the names of men the official record \
                 never asked about. Her testimony exists in a single \
                 living memory. When she dies, it becomes silence — unless \
                 someone asks her first.",
                vec![flag_eq("ending_axis", "burden"), flag_eq("relay_branch", "nella")],
                EmotionTag::Grief,
            ),
            say_if_with("narrator",
                "The relay papers ride in a coat pocket that grows heavier \
                 every year. Transfer orders, payroll ghosts, the \
                 administrative fingerprints of arson. The fire is \
                 documented. The documentation is private. The truth \
                 survives as cargo, not as record.",
                vec![flag_eq("ending_axis", "burden"), flag_eq("relay_branch", "papers")],
                EmotionTag::Quiet,
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
