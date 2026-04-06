//! Chapter 4 — Ropehouse Blood.
//!
//! Emotional law: Entanglement. Standing somewhere costs something.
//! Rosa joins. Full 4-person party. The game's JRPG spine arrives.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;
use crate::state::argument;

// ─── Entry ─────────────────────────────────────────────────────────

/// Chapter 4 entry — into Varela country.
pub fn varela_approach() -> Scene {
    scene(
        "rh_varela_approach", "varela_country", "4_1",
        PacingTag::Exploration,
        vec![
            narrate_with(
                "The sheriff's trail from Black Willow leads into stock country. \
                 Rolling grass, fence lines, cattle trails, windmill pumps. The \
                 landscape changes from swamp to bone-dry range.",
                EmotionTag::Neutral,
            ),
            // Rosa entry differs by prologue choice
            say_if_with("narrator",
                "A woman on horseback meets you at a boundary marker. She has \
                 a rope coiled on her saddle and the look of someone who decided \
                 about you before you opened your mouth. But there's a debt here \
                 — you helped her people once, at the homestead. She's testing \
                 whether that was real.",
                vec![flag_eq("beat5_choice", "homestead_first")],
                EmotionTag::Tense,
            ),
            say_if_with("narrator",
                "A woman on horseback finds you before you find the homestead. \
                 She has a rope coiled on her saddle and the look of someone who \
                 decided about you before you arrived. The sheriff's trail crosses \
                 Varela land, and she will not have outsiders moving through her \
                 territory without a Varela watching.",
                vec![flag_eq("beat5_choice", "town_direct")],
                EmotionTag::Tense,
            ),
            say_with("rosa",
                "Rosa Varela. You're on my land. You move when I say.",
                EmotionTag::Bitter,
            ),
            say_if_with("rosa",
                "And if you damage more than you're worth, I will make that plain.",
                vec![flag_eq("beat5_choice", "town_direct")],
                EmotionTag::Bitter,
            ),
            say_if_with("rosa",
                "My mother says you helped before. I'm here to see if \
                 that holds weight or was just geography.",
                vec![flag_eq("beat5_choice", "homestead_first")],
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Accept her terms", vec![
                relate("galen", "rosa", 0),
            ], to_scene("rh_homestead")),
        ],
        vec![],
    )
}

/// The Varela homestead — land under paper siege.
pub fn homestead_scene() -> Scene {
    scene(
        "rh_homestead", "varela_homestead", "4_2",
        PacingTag::Pressure,
        vec![
            narrate(
                "The Varela homestead: stone-and-timber house older than the rail. \
                 Corrals, a well, a kitchen garden feeding more people than planned. \
                 Claim papers sit next to ammunition in the same cabinet.",
            ),
            say_with("alma",
                "Alma Varela. My daughter tells me you're following the same \
                 paper trail that's eating our fence lines.",
                EmotionTag::Bitter,
            ),
            narrate(
                "The cattle are thinner than they should be. The creek bed \
                 downstream is cracked and white. A diversion dam — legal — is \
                 killing the Varela water supply slowly.",
            ),
        ],
        vec![
            choice("Visit the water claim", vec![], to_scene("rh_water_claim")),
        ],
        vec![],
    )
}

/// The water claim — the central argument.
pub fn water_claim_scene() -> Scene {
    scene(
        "rh_water_claim", "varela_water_claim", "4_3",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The creek junction where Varela water rights and a smaller \
                 homestead family's sold rights intersect with a Briar Line \
                 diversion dam. The dam is legal. The creek bed downstream \
                 is white and cracked.",
                EmotionTag::Tense,
            ),
            // The party argues
            say_with("rosa",
                "Cut the diversion. My cattle are dying. Legal channels have \
                 failed for three years.",
                EmotionTag::Bitter,
            ),
            say_with("ada",
                "Violence will bring marshals. Your legal position gets worse. \
                 More people get hurt.",
                EmotionTag::Tense,
            ),
            say_with("eli",
                "I can forge counter-filings. Create enough doubt to stall \
                 the claim. Nobody gets shot.",
                EmotionTag::Dry,
            ),
            say_with("ada",
                "That con might destroy the smaller family who sold their \
                 rights to survive. You're using the same paper logic being \
                 used against the Varelas.",
                EmotionTag::Bitter,
            ),
        ],
        vec![
            choice("Cut the diversion by force — Rosa is right", vec![
                set_text("ch4_stance", "force"),
                rep(ReputationAxis::Rancher, 10),
                rep(ReputationAxis::Railroad, -10),
                relate("galen", "rosa", 8),
                relate("galen", "ada", -5),
            ], to_scene("rh_ropehouse_approach")),
            choice("Use Eli's con — quieter, no blood", vec![
                set_text("ch4_stance", "con"),
                relate("galen", "eli", 5),
                relate("galen", "rosa", -3),
                relate("galen", "ada", -2),
            ], to_scene("rh_ropehouse_approach")),
            choice("Negotiate with medical evidence — Ada's way", vec![
                set_text("ch4_stance", "negotiate"),
                relate("galen", "ada", 8),
                relate("galen", "rosa", -2),
            ], to_scene("rh_ropehouse_approach")),
        ],
        vec![],
    )
}

/// Ropehouse approach — the hearing that breaks.
pub fn ropehouse_approach() -> Scene {
    scene(
        "rh_ropehouse_approach", "ropehouse", "4_5",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The Ropehouse: a repurposed barn where ranch country holds its \
                 own justice. Wooden benches, a table serving as dock and judge's \
                 bench, a rope coil on the wall everyone pretends is decorative.",
                EmotionTag::Tense,
            ),
            narrate(
                "A captured railroad surveyor sits at the table. Ranch families \
                 on one side, rail men on the other. The hearing is about the \
                 poisoned well, the water claim, and who owns the ground.",
            ),
            narrate("Then armed railroad men arrive to extract the prisoner."),
            say("rosa", "They're not taking him."),
            narrate_with(
                "The hearing breaks. The fight is inside and outside the Ropehouse.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Hold the Ropehouse", vec![], to_combat("ropehouse_fight")),
        ],
        vec![],
    )
}

/// Post-Ropehouse aftermath — "we won the fight and lost the room."
pub fn ropehouse_aftermath() -> Scene {
    scene(
        "rh_aftermath", "ropehouse", "4_5",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The Ropehouse is damaged. Benches splintered. The hearing is \
                 destroyed. Whatever justice was being attempted is now replaced \
                 by the fact of violence.",
                EmotionTag::Grief,
            ),
            narrate(
                "We won the fight and lost the room.",
            ),
            // Stance-dependent reactions
            say_if_with("rosa",
                "You stood with us. That's more than talk.",
                vec![flag_eq("ch4_stance", "force")],
                EmotionTag::Warm,
            ),
            say_if_with("ada",
                "The marshals will come now. I hope that was worth it.",
                vec![flag_eq("ch4_stance", "force")],
                EmotionTag::Bitter,
            ),
            say_if_with("eli",
                "The filings bought us time. Whether the family I burned \
                 forgives us is another matter.",
                vec![flag_eq("ch4_stance", "con")],
                EmotionTag::Quiet,
            ),
            say_if_with("rosa",
                "You used tricks where blood or iron would have been honest.",
                vec![flag_eq("ch4_stance", "con")],
                EmotionTag::Bitter,
            ),
            say_if_with("ada",
                "The evidence held. The water data shows both families are \
                 being poisoned by the same geology the diversion worsened.",
                vec![flag_eq("ch4_stance", "negotiate")],
                EmotionTag::Neutral,
            ),
            say_if_with("rosa",
                "Talk works until it doesn't. But you tried. I'll give you that.",
                vec![flag_eq("ch4_stance", "negotiate")],
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Continue", vec![], to_scene("rh_chapter_close")),
        ],
        vec![],
    )
}

/// Chapter close — Rosa joins permanently.
pub fn chapter_close() -> Scene {
    scene_with_memory(
        "rh_chapter_close", "varela_country", "4_6",
        PacingTag::Intimate,
        vec![
            say_with("rosa",
                "The sheriff's trail goes through country I know better than \
                 you. And my mother can't hold the land alone while the paper \
                 keeps moving.",
                EmotionTag::Neutral,
            ),
            say_if_with("rosa",
                "You stood. That's enough to ride with. For now.",
                vec![flag_eq("ch4_stance", "force")],
                EmotionTag::Quiet,
            ),
            say_if_with("rosa",
                "I don't trust your methods. But I trust you're still walking \
                 the right direction.",
                vec![flag_eq("ch4_stance", "con")],
                EmotionTag::Bitter,
            ),
            say_if_with("rosa",
                "You tried the decent thing first. That's rarer than you think.",
                vec![flag_eq("ch4_stance", "negotiate")],
                EmotionTag::Quiet,
            ),
            narrate_with(
                "Rosa joins the party. Not as friendship. As territorial necessity.",
                EmotionTag::Neutral,
            ),
        ],
        vec![],
        vec![
            StateEffect::AddPartyMember(CharacterId::new("rosa")),
            set_flag("rosa_joined", true),
            set_flag("ch4_complete", true),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("ropehouse_damage"),
                callback_type: MemoryCallbackType::Echo,
                target_chapter: Some(ChapterId::new("ch10")),
            },
        ],
    )
}

// ─── Encounter ─────────────────────────────────────────────────────

/// The Ropehouse fight — the first full 4-person party battle.
/// Inside: tight quarters, rope work, cover play.
/// Outside: wider yard, positioning, disruption.
pub fn ropehouse_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("ropehouse_fight"),
        phases: vec![
            // Phase 1 — Inside the Ropehouse
            CombatPhase {
                id: "inside".to_string(),
                description: "Tight quarters. Benches, table, rope coil. \
                              Rosa's territory.".to_string(),
                enemies: vec![
                    enemy_full("rail_enforcer_a", "Rail Enforcer", 28, 20, 9, 55, 7, 15, 6),
                    enemy_full("rail_enforcer_b", "Rail Enforcer", 26, 18, 8, 52, 8, 15, 6),
                ],
                npc_allies: vec![],
                entry_conditions: vec![],
                phase_effects: vec![],
            },
            // Phase 2 — Outside in the yard
            CombatPhase {
                id: "yard".to_string(),
                description: "The fight opens into the wider yard. Fence lines, \
                              troughs, a wagon. Positioning matters.".to_string(),
                enemies: vec![
                    enemy_full("hired_gun_a", "Hired Gun", 30, 22, 10, 58, 7, 20, 7),
                    enemy_full("hired_gun_b", "Hired Gun", 28, 20, 9, 55, 6, 15, 6),
                    enemy("ranch_hand_ally_turned", "Panicked Bystander", 12, 5, 3, 25, 5),
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
            name: "The Ropehouse".to_string(),
            cover: vec![
                CoverElement { name: "Overturned bench".to_string(), durability: 20, destructible: true },
                CoverElement { name: "Judge's table".to_string(), durability: 40, destructible: true },
                CoverElement { name: "Yard fence".to_string(), durability: 30, destructible: true },
                CoverElement { name: "Stone trough".to_string(), durability: 80, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "hold_ropehouse".to_string(),
                label: "Hold the Ropehouse".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![
                    set_flag("ropehouse_lost", true),
                ],
                success_consequence: vec![
                    set_flag("ropehouse_held", true),
                ],
            },
            Objective {
                id: "protect_witness".to_string(),
                label: "Protect the surveyor-witness".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![
                    set_flag("witness_extracted", true),
                ],
                success_consequence: vec![
                    set_flag("witness_secured", true),
                ],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

// ─── Scene Registry ────────────────────────────────────────────────

pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        "rh_varela_approach" => Some(varela_approach()),
        "rh_homestead" => Some(homestead_scene()),
        "rh_water_claim" => Some(water_claim_scene()),
        "rh_ropehouse_approach" => Some(ropehouse_approach()),
        "rh_aftermath" => Some(ropehouse_aftermath()),
        "rh_chapter_close" => Some(chapter_close()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "ropehouse_fight" => Some(ropehouse_encounter()),
        _ => None,
    }
}
