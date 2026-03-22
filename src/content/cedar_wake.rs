//! Chapter 1 — Cedar Wake.
//!
//! The real beginning. Love → clean work → crack.
//! Three phases: settling in, doing the work, the turn.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Phase A: Settling In ──────────────────────────────────────────

/// 1A1 — First ride into Cedar Wake.
pub fn arrival_scene() -> Scene {
    scene(
        "cw_arrival", "cedar_wake", "1a1",
        PacingTag::Exploration,
        vec![
            narrate_with(
                "Dust hangs gold in the light. A blacksmith's hammer rings somewhere \
                 off to the left. Laundry moves between two porches like flags nobody \
                 bothered to salute.",
                EmotionTag::Warm,
            ),
            narrate(
                "The town is small enough to know your name by supper, and big enough \
                 to pretend not to.",
            ),
            say_with("voss",
                "Slow your horse. You ride in fast, people think you need something.",
                EmotionTag::Neutral,
            ),
            say("galen", "What if I do?"),
            say("voss", "Then slow it more."),
        ],
        vec![
            choice("Look around town", vec![], to_scene("cw_mercantile")),
            choice("Report to the county office", vec![], to_scene("cw_voss_office")),
        ],
        vec![],
    )
}

/// Molly Breck — first meeting at the mercantile.
pub fn mercantile_scene() -> Scene {
    scene(
        "cw_mercantile", "cedar_wake_mercantile", "1a3",
        PacingTag::Exploration,
        vec![
            narrate(
                "A girl of maybe twelve is sitting on an upturned crate outside the \
                 mercantile, boot heels knocking the wood. She watches you like she's \
                 taking inventory.",
            ),
            say("molly", "That your horse?"),
            say("galen", "Last I checked."),
            say("molly", "Looks offended to be here."),
            say("galen", "He'll survive it."),
            say("molly", "Town might not. Marshal brought another one."),
            narrate(
                "She hops off the crate and walks a loose circle around the horse, \
                 not scared of either of you.",
            ),
            say("molly", "That knot slips wet."),
            narrate("You look down. She's right."),
            say_with("molly",
                "Bandits north of Juniper Rise. Skinny ones, not proper bandits. \
                 Hungry enough to count, though.",
                EmotionTag::Neutral,
            ),
            say("voss", "Remember that."),
            say("galen", "The knot or the bandits?"),
            say_with("voss", "Girl.", EmotionTag::Dry),
        ],
        vec![
            choice("Visit the livery", vec![], to_scene("cw_livery")),
            choice("Head to the county office", vec![], to_scene("cw_voss_office")),
        ],
        vec![
            set_flag("met_molly", true),
        ],
    )
}

/// Livery — meet Declan Oar.
pub fn livery_scene() -> Scene {
    scene(
        "cw_livery", "cedar_wake_livery", "1a3",
        PacingTag::Exploration,
        vec![
            narrate(
                "The livery smells like hay, horse, and iron. A quiet young man is \
                 brushing a roan mare with the focus of someone who prefers animals \
                 to conversation.",
            ),
            say("declan", "New runner?"),
            say("galen", "That obvious?"),
            say_with("declan",
                "Horse is trail-worn. You're not. Means you just started.",
                EmotionTag::Neutral,
            ),
            say("declan", "East road's soft after the rain. Take the ridge path if Voss sends you out."),
        ],
        vec![
            choice("Thank him and go", vec![], to_scene("cw_voss_office")),
        ],
        vec![
            set_flag("met_declan", true),
        ],
    )
}

/// Voss office — first assignment.
pub fn voss_office_scene() -> Scene {
    scene(
        "cw_voss_office", "cedar_wake_office", "1a2",
        PacingTag::Exploration,
        vec![
            narrate(
                "The county office: a single room with a desk, a gun rack, a territory \
                 map on the wall with pins in it, and a wood stove that makes the whole \
                 place smell like cedar smoke.",
            ),
            say_with("voss",
                "You're here because I asked for a runner, not a deputy. You ride, \
                 you deliver, you come back. Questions?",
                EmotionTag::Neutral,
            ),
            say("galen", "None yet."),
            say("voss", "Good. That'll change."),
        ],
        vec![
            choice(
                "Accept the first assignment",
                vec![set_flag("accepted_runner", true)],
                to_scene("cw_first_courier"),
            ),
        ],
        vec![],
    )
}

/// First courier run — summons delivery (Trail Eye first trigger).
pub fn first_courier_scene() -> Scene {
    scene(
        "cw_first_courier", "east_road", "1a4",
        PacingTag::Exploration,
        vec![
            narrate_with(
                "First trail ride. Short, scenic. The road opens into valley grass \
                 and red soil. Trail Eye triggers for the first time — you notice \
                 animal tracks crossing the road, a weather sign in the cloud line.",
                EmotionTag::Warm,
            ),
            narrate(
                "The homesteader is gruff. He accepts the summons and gives you a \
                 hard look.",
            ),
            say_with("homesteader", "You're Voss's new boy.", EmotionTag::Neutral),
        ],
        vec![
            choice("Ride back to Cedar Wake", vec![], to_scene("cw_evening")),
        ],
        vec![
            unlock("galen", "trail_eye"),
        ],
    )
}

/// Evening at the boarding house.
pub fn evening_scene() -> Scene {
    scene(
        "cw_evening", "cedar_wake_boarding", "1a5",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Evening. The boarding house is active. Someone plays the piano badly. \
                 Kerosene light in windows. A wagon rolling slow over the ruts.",
                EmotionTag::Warm,
            ),
            say_if_with("molly",
                "Ma says Cedar Wake's a good place if folks don't get ambitious. \
                 That true?",
                vec![flag_is("met_molly", true)],
                EmotionTag::Quiet,
            ),
            say_if("galen", "Don't know yet.",
                vec![flag_is("met_molly", true)],
            ),
            say_if_with("molly",
                "Well. Find out before somebody ruins it.",
                vec![flag_is("met_molly", true)],
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Rest. Tomorrow there's work.", vec![], to_scene("cw_shooting_post")),
        ],
        vec![
            set_flag("first_evening", true),
        ],
    )
}

// ─── Phase B: Doing the Work ───────────────────────────────────────

/// Shooting post — Voss teaches Steady Aim.
pub fn shooting_post_scene() -> Scene {
    scene(
        "cw_shooting_post", "cedar_wake_range", "1b1",
        PacingTag::Intimate,
        vec![
            narrate(
                "Behind the livery. Tin cans on fence posts. A weathered plank with \
                 chalk circles sketched rough across it.",
            ),
            say("voss", "Everybody thinks shooting's speed. Speed's for boys."),
            say("galen", "That sounds like cheating."),
            say("voss", "It sounds like living."),
            narrate("You take aim. He corrects your stance."),
            say_with("voss",
                "Don't look at what you want to hit. Look at what it's about to do.",
                EmotionTag::Warm,
            ),
            narrate("You lower the barrel a hair and fire. The can flips."),
            say("voss", "Again."),
            narrate("Second shot. Better."),
            say("voss", "There. That's not a hand. That's a thought."),
            say_with("voss",
                "Steady Aim isn't patience. People say that because it makes them \
                 feel decent. It's commitment. Once you choose, don't waver just \
                 because the target turns human on you.",
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Take the lesson", vec![], to_scene("cw_horse_thief_briefing")),
        ],
        vec![
            unlock("galen", "steady_aim"),
            set_flag("voss_taught_steady_aim", true),
        ],
    )
}

/// Horse thief briefing → combat.
pub fn horse_thief_briefing() -> Scene {
    scene(
        "cw_horse_thief_briefing", "cedar_wake_office", "1b2",
        PacingTag::Pressure,
        vec![
            say("voss", "Horse thief on the east road. Handle it."),
            say_with("renata", "I could've handled that faster.", EmotionTag::Dry),
        ],
        vec![
            choice("Ride out", vec![], to_combat("horse_thief")),
        ],
        vec![],
    )
}

/// Post horse-thief return.
pub fn horse_thief_return() -> Scene {
    scene(
        "cw_horse_thief_return", "cedar_wake", "1b2",
        PacingTag::Exploration,
        vec![
            say_with("voss", "Clean?", EmotionTag::Neutral),
            say("galen", "Clean."),
            say_if_with("molly",
                "Heard you caught somebody. Was it exciting?",
                vec![flag_is("met_molly", true)],
                EmotionTag::Warm,
            ),
        ],
        vec![
            choice("Continue to the next assignment", vec![], to_scene("cw_bandit_briefing")),
            choice("Visit the boardwalk", vec![], to_scene("cw_night_boardwalk")),
        ],
        vec![
            set_flag("horse_thief_done", true),
        ],
    )
}

/// Night boardwalk — optional warmth.
pub fn night_boardwalk() -> Scene {
    scene(
        "cw_night_boardwalk", "cedar_wake_boardwalk", "1b6",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "Music leaking from a bad upright inside the saloon. Kerosene light \
                 in windows. Somebody laughing too hard.",
                EmotionTag::Warm,
            ),
            say_if("molly", "You ever been farther west than this?",
                vec![flag_is("met_molly", true)],
            ),
            say_if("galen", "No.",
                vec![flag_is("met_molly", true)],
            ),
            say_if("molly", "Me neither. You think it gets better?",
                vec![flag_is("met_molly", true)],
            ),
            say_if("galen", "I think it gets farther.",
                vec![flag_is("met_molly", true)],
            ),
            say_if_with("molly", "That's not the same thing.",
                vec![flag_is("met_molly", true)],
                EmotionTag::Quiet,
            ),
            say_if_with("molly",
                "Marshal good?",
                vec![flag_is("met_molly", true)],
                EmotionTag::Quiet,
            ),
            say_if("galen", "Yes.",
                vec![flag_is("met_molly", true)],
            ),
            say_if_with("molly",
                "Mm. Just seems like if a man's good, folks ought not look relieved \
                 when he rides out.",
                vec![flag_is("met_molly", true)],
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Head in for the night", vec![], to_scene("cw_bandit_briefing")),
        ],
        vec![],
    )
}

/// Bandit camp briefing.
pub fn bandit_briefing() -> Scene {
    scene(
        "cw_bandit_briefing", "cedar_wake_office", "1b7",
        PacingTag::Pressure,
        vec![
            say("cal", "Camp in the low hills east of town. Three, maybe four men. \
                 They've been raiding supply wagons on the Millrace Road."),
            say("voss", "Take Cal. Clear it."),
        ],
        vec![
            choice(
                "Flank approach — come in from the side",
                vec![set_flag("bandit_approach", true)],
                to_combat("bandit_camp"),
            ),
            choice(
                "Main group — stay with Cal",
                vec![set_flag("bandit_approach", false)],
                to_combat("bandit_camp"),
            ),
        ],
        vec![],
    )
}

/// Post bandit-camp return — "Clean work."
pub fn bandit_camp_return() -> Scene {
    scene(
        "cw_bandit_camp_return", "cedar_wake", "1b8",
        PacingTag::Exploration,
        vec![
            say_with("voss", "Clean work.", EmotionTag::Warm),
            narrate("Two words. From Voss, those two words feel like the best \
                     praise Galen has ever received."),
            say_if_with("molly",
                "The teamster's wife will be glad about the tools.",
                vec![flag_is("met_molly", true)],
                EmotionTag::Warm,
            ),
        ],
        vec![
            choice("Continue", vec![], to_scene("cw_bitter_cut_dispatch")),
        ],
        vec![
            set_flag("bandit_camp_done", true),
            set_flag("clean_work", true),
        ],
    )
}

// ─── Phase C: The Turn ─────────────────────────────────────────────

/// Bitter Cut dispatch.
pub fn bitter_cut_dispatch() -> Scene {
    scene(
        "cw_bitter_cut_dispatch", "cedar_wake_office", "1c1",
        PacingTag::Pressure,
        vec![
            narrate("Voss calls Galen in alone. Not Cal, not Renata. Galen."),
            narrate("He hands you a folded, sealed paper."),
            say("voss", "Take this to Commander Hale at Bitter Cut. Time-sensitive. \
                 Ride direct. Don't open it."),
            say("galen", "What is it?"),
            say_with("voss", "Authority.", EmotionTag::Neutral),
        ],
        vec![
            choice("Ride to Bitter Cut", vec![], to_scene("cw_bitter_cut_arrival")),
        ],
        vec![
            set_flag("carrying_dispatch", true),
        ],
    )
}

/// Bitter Cut arrival — the labor camp.
pub fn bitter_cut_arrival() -> Scene {
    scene(
        "cw_bitter_cut_arrival", "bitter_cut", "1c2",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The road narrows into blasted stone and old rail grade. Heat sticks \
                 inside the cut like it was nailed there on purpose.",
                EmotionTag::Tense,
            ),
            narrate(
                "Below, the camp sits ugly and temporary: canvas lean-tos, cook smoke \
                 too thin to mean a real meal, tool racks, a water cart with guards on \
                 it, and men standing in knots with the posture of people trying not \
                 to become a crowd.",
            ),
            narrate("No one looks surprised to see Marshal Voss. That is the first bad sign."),
            say("voss", "Stay loose."),
            say("galen", "You said this was a wage dispute."),
            say_with("voss",
                "That's what men call it when they still want to believe it'll end \
                 in talking.",
                EmotionTag::Neutral,
            ),
        ],
        vec![
            choice("Deliver the dispatch to the foreman", vec![], to_scene("cw_bitter_cut_dispatch_delivery")),
        ],
        vec![],
    )
}

/// Dispatch delivery — Eben Sorrell and the escalation.
pub fn bitter_cut_dispatch_delivery() -> Scene {
    scene(
        "cw_bitter_cut_dispatch_delivery", "bitter_cut", "1c3",
        PacingTag::Crisis,
        vec![
            narrate(
                "A man steps forward — broad-shouldered, gray at the temples, holding \
                 a shovel like a cane, not a threat. Eben Sorrell.",
            ),
            say("eben", "That for us, or against us?"),
            say("galen", "For the foreman."),
            say_with("eben", "Then I reckon that answers it.", EmotionTag::Bitter),
            narrate("The foreman takes the dispatch with both hands. He reads. \
                     Something in his face settles. Not relief. Permission."),
            say("foreman",
                "Marshal, we are authorized to restore access to company property \
                 and remove obstructing persons by necessary force.",
            ),
            say("galen", "Remove?"),
            say("voss", "Easy word. People hate plain ones."),
            say_with("woman",
                "They brought paper for our hunger.",
                EmotionTag::Bitter,
            ),
            narrate("Nobody answers her. That line hangs there."),
            say("voss", "Rook. Ride left. Keep sight on the high crates."),
        ],
        vec![
            choice(
                "Hold on the workers",
                vec![set_flag("bitter_cut_focus", true)],
                to_combat("bitter_cut"),
            ),
            choice(
                "Hold on the water cart",
                vec![set_flag("bitter_cut_focus", false)],
                to_combat("bitter_cut"),
            ),
        ],
        vec![],
    )
}

/// Post Bitter Cut — Voss lesson and ride home.
pub fn bitter_cut_aftermath() -> Scene {
    scene(
        "cw_bitter_cut_aftermath", "bitter_cut", "1c5",
        PacingTag::Intimate,
        vec![
            narrate(
                "Dust makes everything look older than it is. One guard dead. Two \
                 workers down. The shotgun boy crying over a hand that no longer closes.",
            ),
            narrate(
                "Eben is on his knees with both palms visible, not because he \
                 surrendered, but because he finally understood this was never a \
                 fight he was allowed to win.",
            ),
            say("voss", "You froze once."),
            say("galen", "They weren't bandits."),
            say("voss", "No."),
            say("galen", "They were hungry."),
            say_with("voss",
                "Hungry men with tools still cave a skull in the same.",
                EmotionTag::Neutral,
            ),
            say("galen", "That paper you gave Colter— you knew what it would do."),
            say("voss", "I knew what it had to allow."),
            say("galen", "That's not the same thing."),
            say_with("voss",
                "It is if you plan to keep roads open.",
                EmotionTag::Neutral,
            ),
            say_with("voss",
                "Listen carefully, Galen. Out here, things turn human on you very \
                 fast. If you stop to mourn that every time, men who never stop will \
                 decide the day instead.",
                EmotionTag::Neutral,
            ),
            say_with("voss", "You did your job.", EmotionTag::Warm),
            narrate_with(
                "That is the lie Galen takes with him. Not because he believes it \
                 cleanly. Because he needs something to hold before the guilt \
                 finishes taking shape.",
                EmotionTag::Grief,
            ),
        ],
        vec![
            choice("Ride back to Cedar Wake", vec![], to_scene("cw_bitter_cut_return")),
        ],
        vec![],
    )
}

/// Return to Cedar Wake after Bitter Cut.
pub fn bitter_cut_return() -> Scene {
    scene(
        "cw_bitter_cut_return", "cedar_wake", "1c7",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The ride back is long enough for blood to go brown where it dried \
                 on the sleeve. No one talks for a while.",
                EmotionTag::Grief,
            ),
            narrate(
                "When Cedar Wake comes into view, it looks exactly like it did \
                 yesterday. That is insulting.",
            ),
            narrate(
                "Boardwalk. Laundry. Mercantile awning. A dog asleep in the same \
                 patch of shade.",
            ),
            say_if_with("molly",
                "You look stupid.",
                vec![flag_is("met_molly", true)],
                EmotionTag::Warm,
            ),
            narrate_with(
                "It's the meanest kind of kindness a child can manage.",
                EmotionTag::Warm,
            ),
            say_if("galen", "Long day.",
                vec![flag_is("met_molly", true)],
            ),
            say_if("molly", "Town still here.",
                vec![flag_is("met_molly", true)],
            ),
            say_if_with("molly",
                "You sure you are?",
                vec![flag_is("met_molly", true)],
                EmotionTag::Quiet,
            ),
            // If Molly wasn't met, different ending
            narrate_with(
                "Cedar Wake feels safe enough to believe in. That safety is the trap.",
                EmotionTag::Quiet,
            ),
        ],
        vec![],
        vec![
            set_flag("chapter1_complete", true),
            set_flag("bitter_cut_done", true),
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// Horse thief — the first clean fight.
pub fn horse_thief_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("horse_thief"),
        phases: vec![CombatPhase {
            id: "chase".to_string(),
            description: "A thief on the east road. Quick Draw shines.".to_string(),
            enemies: vec![
                enemy("thief", "Horse Thief", 18, 12, 6, 40, 7),
            ],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None, // No standoff for youth encounters
        party_slots: 4,
        terrain: Terrain {
            name: "East Road".to_string(),
            cover: vec![
                CoverElement { name: "Trail brush".to_string(), durability: 20, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![Objective {
            id: "catch_thief".to_string(),
            label: "Catch the horse thief".to_string(),
            objective_type: ObjectiveType::Primary,
            fail_consequence: vec![],
            success_consequence: vec![
                set_flag("horse_thief_caught", true),
            ],
        }],
        outcome_effects: vec![],
    }
}

/// Bandit camp — the "clean work" fight. First real multi-enemy encounter.
pub fn bandit_camp_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("bandit_camp"),
        phases: vec![CombatPhase {
            id: "raid".to_string(),
            description: "Clear the camp on Millrace Road. Cal fights alongside.".to_string(),
            enemies: vec![
                enemy_full("bandit_leader", "Camp Boss", 30, 22, 10, 55, 7, 20, 6),
                enemy("bandit_a", "Bandit", 20, 15, 7, 45, 8),
                enemy("bandit_b", "Bandit", 20, 15, 7, 45, 6),
            ],
            npc_allies: vec![
                NpcCombatant {
                    character: CharacterId::new("cal"),
                    behavior: NpcBehavior::Professional,
                    hp: 35,
                    nerve: 30,
                },
            ],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None, // No standoff in youth
        party_slots: 4,
        terrain: Terrain {
            name: "Bandit Camp — Millrace Hills".to_string(),
            cover: vec![
                CoverElement { name: "Supply crates".to_string(), durability: 30, destructible: true },
                CoverElement { name: "Rock overhang".to_string(), durability: 100, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![Objective {
            id: "clear_camp".to_string(),
            label: "Clear the bandit camp".to_string(),
            objective_type: ObjectiveType::Primary,
            fail_consequence: vec![],
            success_consequence: vec![
                set_flag("bandit_camp_cleared", true),
            ],
        }],
        outcome_effects: vec![],
    }
}

/// Bitter Cut suppression — same skills, different meaning.
pub fn bitter_cut_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("bitter_cut"),
        phases: vec![CombatPhase {
            id: "suppression".to_string(),
            description: "The workers scatter, but not like bandits. Like laborers \
                          in an accident too large to understand.".to_string(),
            enemies: vec![
                // Not real threats. Low stats. Low nerve. Desperate, not dangerous.
                enemy_full("worker_a", "Desperate Worker", 12, 6, 4, 30, 5, 5, 3),
                enemy_full("worker_b", "Shovel Man", 15, 8, 5, 35, 4, 10, 4),
                enemy_full("shotgun_boy", "Shotgun Boy", 10, 5, 6, 25, 6, 40, 4),
            ],
            npc_allies: vec![
                NpcCombatant {
                    character: CharacterId::new("cal"),
                    behavior: NpcBehavior::Professional,
                    hp: 35,
                    nerve: 30,
                },
                NpcCombatant {
                    character: CharacterId::new("renata"),
                    behavior: NpcBehavior::Professional,
                    hp: 30,
                    nerve: 25,
                },
            ],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None,
        party_slots: 4,
        terrain: Terrain {
            name: "Bitter Cut Labor Camp".to_string(),
            cover: vec![
                CoverElement { name: "Water cart".to_string(), durability: 40, destructible: false },
                CoverElement { name: "Tool crates".to_string(), durability: 20, destructible: true },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "maintain_order".to_string(),
                label: "Maintain order".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![],
                success_consequence: vec![
                    set_flag("bitter_cut_order_maintained", true),
                ],
            },
            // THE objective. First time in the game.
            Objective {
                id: "minimize_casualties".to_string(),
                label: "Minimize civilian casualties".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![
                    set_flag("bitter_cut_casualties_high", true),
                ],
                success_consequence: vec![
                    set_flag("bitter_cut_casualties_low", true),
                ],
            },
        ],
        outcome_effects: vec![],
    }
}

/// Young Galen's party data for Chapter 1.
pub fn youth_galen() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![(
        "galen".to_string(), "Galen Rook".to_string(),
        30, 20, 8,   // hp, nerve, ammo — lower than adult
        14, 65, 7,   // speed higher (youth = fast), accuracy/damage lower
        vec![
            SkillId::new("quick_draw"),
            SkillId::new("snap_shot"),
            SkillId::new("duck"),
            SkillId::new("sprint"),
            // steady_aim unlocked during shooting post scene
        ],
        vec![], // no duo techs yet — solo
        vec![],
    )]
}

// ─── Scene Registry ────────────────────────────────────────────────

/// Get a Cedar Wake scene by ID.
pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "cw_arrival" => Some(arrival_scene()),
        "cw_mercantile" => Some(mercantile_scene()),
        "cw_livery" => Some(livery_scene()),
        "cw_voss_office" => Some(voss_office_scene()),
        "cw_first_courier" => Some(first_courier_scene()),
        "cw_evening" => Some(evening_scene()),
        "cw_shooting_post" => Some(shooting_post_scene()),
        "cw_horse_thief_briefing" => Some(horse_thief_briefing()),
        "cw_horse_thief_return" => Some(horse_thief_return()),
        "cw_night_boardwalk" => Some(night_boardwalk()),
        "cw_bandit_briefing" => Some(bandit_briefing()),
        "cw_bandit_camp_return" => Some(bandit_camp_return()),
        "cw_bitter_cut_dispatch" => Some(bitter_cut_dispatch()),
        "cw_bitter_cut_arrival" => Some(bitter_cut_arrival()),
        "cw_bitter_cut_dispatch_delivery" => Some(bitter_cut_dispatch_delivery()),
        "cw_bitter_cut_aftermath" => Some(bitter_cut_aftermath()),
        "cw_bitter_cut_return" => Some(bitter_cut_return()),
        _ => None,
    }
}

/// Get a Cedar Wake encounter by ID.
pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "horse_thief" => Some(horse_thief_encounter()),
        "bandit_camp" => Some(bandit_camp_encounter()),
        "bitter_cut" => Some(bitter_cut_encounter()),
        _ => None,
    }
}
