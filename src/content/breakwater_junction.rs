//! Chapter 11 — Breakwater Junction.
//!
//! Emotional law: Retaliation. The machine stops hiding. Peak party synthesis.
//! Galen's hand is wounded. The last time the full party fights at full capacity.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Scenes ────────────────────────────────────────────────────────

/// The retaliation force assembles.
pub fn junction_entry() -> Scene {
    scene(
        "bj_entry", "breakwater_junction", "11_1",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "Breakwater Junction: where three rail corridors, two telegraph \
                 routes, and the basin's primary water distribution all intersect. \
                 The machine is done pretending.",
                EmotionTag::Tense,
            ),
            narrate(
                "Rail enforcement, hired guns, and a territorial marshal's \
                 detachment converge on the junction. The party has hours, not days.",
            ),
            say_with("eli",
                "Deadwater hurt them. This is the answer.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Prepare for the battle", vec![], to_scene("bj_preparation")),
        ],
        vec![
            set_flag("ch11_started", true),
        ],
    )
}

/// Battle preparation — the last strategic assembly.
pub fn preparation() -> Scene {
    scene(
        "bj_preparation", "breakwater_junction", "11_2",
        PacingTag::Crisis,
        vec![
            narrate("This is the last time the full party operates at peak capacity."),
            say_with("rosa", "Hold everything. Don't yield a foot.", EmotionTag::Bitter),
            say_with("ada", "Evacuate civilians first. Then defend what's defensible.", EmotionTag::Tense),
            say_with("miriam",
                "Make the defense visible. Let the retaliation look like what it is.",
                EmotionTag::Neutral,
            ),
            say_with("eli", "Let them overcommit. Then the evidence of retaliation \
                      IS the next round of testimony.", EmotionTag::Dry),
            say_if_with("lucien",
                "Controlled demolition of the depot row. Deny them staging ground.",
                vec![flag_is("lucien_captured", true)],
                EmotionTag::Neutral,
            ),
            // Relay branch context — what's at stake shapes the defense
            say_if_with("narrator",
                "The relay's structural proof is in the evidence bundle. \
                 Tom's engineering — the load calculations that proved \
                 deliberate failure — gives the defense a foundation. \
                 They're not just holding ground. They're holding proof \
                 that ground was taken by design.",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Quiet,
            ),
            say_if_with("narrator",
                "Nella's testimony is in the evidence bundle. A living \
                 witness who saw the relay fire and named the men who \
                 lit it. If the junction falls, the machine buries one \
                 more voice. The defense is personal.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "The relay papers are in the evidence bundle. Transfer \
                 orders, payroll ghosts, the paper trail of arson. If \
                 Breakwater falls, the papers burn again — the same \
                 story, the same method, a generation later.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Hold Breakwater", vec![], to_combat("breakwater_battle")),
        ],
        vec![],
    )
}

/// The hand injury — Galen's permanent cost.
pub fn hand_injury() -> Scene {
    scene(
        "bj_hand_injury", "breakwater_junction", "11_4",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The battle turns. The party is holding. Then the price comes due.",
                EmotionTag::Grief,
            ),
            narrate(
                "Galen holds a position to let Ada evacuate a wounded civilian. \
                 A support beam drops. His dominant hand catches against steel \
                 and stone.",
            ),
            narrate_with(
                "The shot he fired was correct. The cost is permanent.",
                EmotionTag::Grief,
            ),
        ],
        vec![
            choice("Ada treats the hand", vec![
                StateEffect::ApplyInjury {
                    character: CharacterId::new("galen"),
                    injury: InjuryId::new("hand_injury_permanent"),
                },
                set_flag("hand_wounded", true),
            ], to_scene("bj_ada_treatment")),
        ],
        vec![],
    )
}

/// Ada's treatment scene — the bridge between adult and older Galen.
pub fn ada_treatment() -> Scene {
    scene(
        "bj_ada_treatment", "breakwater_junction", "11_4",
        PacingTag::Intimate,
        vec![
            narrate("Ada examines the hand. Clinical first."),
            say_with("ada",
                "The bones will set. Two of them clean, one with a ridge you'll \
                 feel when it's cold.",
                EmotionTag::Neutral,
            ),
            say_with("ada",
                "Tendon damage here. Partial. You'll have strength. Not speed.",
                EmotionTag::Neutral,
            ),
            say_with("ada",
                "You will hold a gun. You will aim a gun. You will fire a gun.",
                EmotionTag::Neutral,
            ),
            narrate("A beat."),
            say_with("ada",
                "You will not draw the way you used to. That nerve path is \
                 interrupted. It won't come back.",
                EmotionTag::Quiet,
            ),
            narrate_with(
                "Galen looks at the hand. His hand. The one Voss trained at \
                 the shooting post in Cedar Wake.",
                EmotionTag::Grief,
            ),
            say_with("ada",
                "The skill is still in you. It just has to travel a different \
                 road to your fingers now.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Continue", vec![], to_scene("bj_victory")),
        ],
        vec![],
    )
}

/// Victory — but the hand doesn't close right anymore.
pub fn victory() -> Scene {
    scene_with_memory(
        "bj_victory", "breakwater_junction", "11_5",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The retaliation force is broken. The junction is held. The water \
                 flows. The wire is up.",
                EmotionTag::Warm,
            ),
            narrate_with(
                "We won. And Galen's hand — Ada wrapped it, and she looked at him \
                 the way you look at something that won't be the same.",
                EmotionTag::Grief,
            ),
            narrate("We won. The hand doesn't close right anymore. We won."),
        ],
        vec![],
        vec![
            set_flag("ch11_complete", true),
            set_flag("breakwater_held", true),
            set_flag("retaliation_answered", true),
            memory("breakwater_victory"),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("hand_injury"),
                callback_type: MemoryCallbackType::Transform,
                target_chapter: Some(ChapterId::new("ch13")),
            },
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// The Battle of Breakwater Junction — peak party synthesis.
///
/// NOTE: This is the last battle before the hand injury. From Ch12 onward,
/// combat encounters should check `hand_wounded` / `hand_injury_permanent`
/// and apply accuracy/speed penalties to Galen via the injury system in
/// `combat/injuries.rs`. The stat penalty lives in combat/, not content/.
pub fn breakwater_battle() -> Encounter {
    Encounter {
        id: EncounterId::new("breakwater_battle"),
        phases: vec![
            CombatPhase {
                id: "main_assault".to_string(),
                description: "Full retaliation force. The machine answers truth with force.".to_string(),
                enemies: vec![
                    enemy_full("rail_enforcer_a", "Rail Enforcer", 32, 25, 11, 60, 7, 12, 7),
                    enemy_full("rail_enforcer_b", "Rail Enforcer", 30, 24, 10, 58, 8, 12, 7),
                    enemy_full("hired_gun_a", "Hired Gun", 28, 22, 9, 55, 7, 15, 6),
                    enemy_full("hired_gun_b", "Hired Gun", 28, 22, 9, 55, 6, 15, 6),
                    enemy("marshal_deputy", "Marshal's Deputy", 25, 20, 8, 52, 7),
                ],
                npc_allies: vec![],
                entry_conditions: vec![],
                phase_effects: vec![],
            },
        ],
        standoff: Some(Standoff {
            postures: vec![StandoffPosture::EarlyDraw, StandoffPosture::SteadyHand, StandoffPosture::Bait],
            allow_focus: true,
            eli_influence: true,
        }),
        party_slots: 4,
        terrain: Terrain {
            name: "Breakwater Junction".to_string(),
            cover: vec![
                CoverElement { name: "Rail car".to_string(), durability: 80, destructible: false },
                CoverElement { name: "Water works wall".to_string(), durability: 70, destructible: false },
                CoverElement { name: "Supply depot".to_string(), durability: 40, destructible: true },
                CoverElement { name: "Bridge railing".to_string(), durability: 25, destructible: true },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "hold_junction".to_string(),
                label: "Hold Breakwater Junction".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![set_flag("junction_lost", true)],
                success_consequence: vec![set_flag("junction_held", true)],
            },
            Objective {
                id: "protect_water_works".to_string(),
                label: "Protect the water works".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![set_flag("water_works_damaged", true)],
                success_consequence: vec![set_flag("water_works_intact", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "bj_entry" => Some(junction_entry()),
        "bj_preparation" => Some(preparation()),
        "bj_hand_injury" => Some(hand_injury()),
        "bj_ada_treatment" => Some(ada_treatment()),
        "bj_victory" => Some(victory()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "breakwater_battle" => Some(breakwater_battle()),
        _ => None,
    }
}
