//! Chapter 2 — Saint's Mile Convoy.
//!
//! The convoy as moving world. Three days, two nights, six named people.
//! Red Switch Wash → Hollow Pump → Saint's Mile Relay.
//! Damage → ambiguity → betrayal.

use crate::types::*;
use crate::scene::types::*;
use crate::combat::types::*;
use crate::content::builders::*;

// ─── Day 1 Scenes ──────────────────────────────────────────────────

/// Join the convoy. Meet the cast. Choose formation.
///
/// Chapter 2 entry point. Referenced by the chapter routing system.
pub fn convoy_join() -> Scene {
    scene(
        "convoy_join", "briar_line_road", "2d1",
        PacingTag::Exploration,
        vec![
            narrate_with(
                "The convoy is six wagons, twelve mules, and enough dust to bury \
                 a small ambition. You join it under a sky that can't decide between \
                 mercy and heat.",
                EmotionTag::Neutral,
            ),
            say_with("bale",
                "Captain Orrin Bale. You ride, you watch, you don't touch the \
                 payroll. Questions?",
                EmotionTag::Neutral,
            ),
            say_with("nella",
                "If you're here to complain, get in line behind the beans.",
                EmotionTag::Dry,
            ),
            say("tom", "Hold that mule's head if he tries theology."),
            say("galen", "Theology?"),
            say_with("tom",
                "That look they get right before they decide work is against God.",
                EmotionTag::Dry,
            ),
        ],
        vec![
            choice("Ride near the water cart",
                vec![set_text("formation", "water_cart")],
                to_scene("convoy_day1_road"),
            ),
            choice("Ride near the payroll coach",
                vec![set_text("formation", "payroll")],
                to_scene("convoy_day1_road"),
            ),
            choice("Scout ahead",
                vec![set_text("formation", "scout")],
                to_scene("convoy_day1_road"),
            ),
            choice("Guard the rear",
                vec![set_text("formation", "rear")],
                to_scene("convoy_day1_road"),
            ),
        ],
        vec![],
    )
}

/// Day 1 road → Red Switch Wash.
pub fn convoy_day1_road() -> Scene {
    scene(
        "convoy_day1_road", "briar_line_road", "2d1",
        PacingTag::Exploration,
        vec![
            narrate("The road opens through scrub country. Nella makes coffee. \
                     Tom checks wheels. Bale runs tight."),
            say_with("nella",
                "You want coffee? Depends. You calling that coffee? No. I'm calling \
                 it leverage.",
                EmotionTag::Dry,
            ),
        ],
        vec![
            choice("Continue to the wash crossing", vec![], to_combat("red_switch_wash")),
        ],
        vec![
            memory("nella_coffee"),
        ],
    )
}

// ─── Night 1 Scenes ────────────────────────────────────────────────

/// Night 1 camp — Eli enters the bloodstream.
pub fn night1_camp() -> Scene {
    scene(
        "night1_camp", "convoy_camp", "2n1",
        PacingTag::Intimate,
        vec![
            narrate_with(
                "The wagons are circled loose. Firelight catches wheel rims and \
                 tired faces. Somebody's boiling coffee to death.",
                EmotionTag::Warm,
            ),
            say_with("eli",
                "There he is. Marshal's favorite mistake.",
                EmotionTag::Dry,
            ),
            say("galen", "You always talk this much when nobody asked?"),
            say("eli", "Usually more. Means I'm tired."),
        ],
        vec![
            choice("Take the flask",
                vec![set_flag("took_flask", true), relate("galen", "eli", 3)],
                to_scene("night1_eli_talk"),
            ),
            choice("Refuse",
                vec![set_flag("refused_flask", true)],
                to_scene("night1_eli_talk"),
            ),
            choice("Ask where he got it",
                vec![set_flag("asked_flask", true)],
                to_scene("night1_eli_talk"),
            ),
        ],
        vec![
            memory("eli_flask"),
        ],
    )
}

/// Night 1 — Eli's deeper talk.
pub fn night1_eli_talk() -> Scene {
    scene(
        "night1_eli_talk", "convoy_camp", "2n1",
        PacingTag::Intimate,
        vec![
            say_with("eli",
                "You ever notice how paper makes men brave when coin doesn't?",
                EmotionTag::Neutral,
            ),
            say("galen", "What's that supposed to mean?"),
            say_with("eli",
                "Means folks think money's the thing, because money jingles. Usually \
                 it's the paper around it. Names. Totals. Signatures. Debts. Permission. \
                 That's what makes men kill tidy.",
                EmotionTag::Neutral,
            ),
            say_with("cask",
                "That's not a town. That's a habit.",
                EmotionTag::Dry,
            ),
        ],
        vec![
            choice("Get some sleep", vec![], to_scene("convoy_day2")),
        ],
        vec![],
    )
}

// ─── Day 2 Scenes ──────────────────────────────────────────────────

/// Day 2 — friction.
pub fn convoy_day2() -> Scene {
    scene(
        "convoy_day2", "briar_line_road", "2d2",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "The second day is meaner. Dust, heat, a skittish mule team, \
                 missing water. Growing distrust between security and teamsters.",
                EmotionTag::Tense,
            ),
            narrate("Trail Eye picks up wagon tracks where there shouldn't be any. \
                     Telegraph wire cut and re-spliced."),
        ],
        vec![
            choice("Continue to Hollow Pump", vec![], to_combat("hollow_pump")),
        ],
        vec![],
    )
}

// ─── Night 2 Scenes ────────────────────────────────────────────────

/// Night 2 — pressure seals.
pub fn night2_camp() -> Scene {
    scene(
        "night2_camp", "convoy_camp", "2n2",
        PacingTag::Intimate,
        vec![
            narrate("The second camp is tighter. Less laughter."),
            say_with("nella",
                "What happens when we hit Saint's Mile? To me? I steal a bath. \
                 Then I find bread that hasn't traveled more than I have. Then I \
                 sleep in a room with walls that don't breathe.",
                EmotionTag::Quiet,
            ),
            say("galen", "Ambitious."),
            say_with("nella",
                "You laugh, but that's civilization.",
                EmotionTag::Warm,
            ),
            say_with("nella",
                "Never trust relief too early. It makes fools of decent people.",
                EmotionTag::Quiet,
            ),
            say_with("tom",
                "Road's safest when everybody knows they're still on it.",
                EmotionTag::Neutral,
            ),
            say_if_with("tom",
                "Something about this run's been arranged too neat.",
                vec![flag_is("horse_thief_caught", true)], // proxy for player being attentive
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Walk the perimeter with Eli", vec![], to_scene("night2_eli_walk")),
            choice("Rest", vec![], to_scene("convoy_day3")),
        ],
        vec![
            memory("nella_bath_bread_roof"),
        ],
    )
}

/// Night 2 — Eli's perimeter warning.
pub fn night2_eli_walk() -> Scene {
    scene(
        "night2_eli_walk", "convoy_camp_perimeter", "2n2",
        PacingTag::Intimate,
        vec![
            say("eli", "You walk heavy when you're thinking."),
            say("galen", "What's at Saint's Mile?"),
            say_with("eli", "Tomorrow.", EmotionTag::Neutral),
            say("galen", "That's not funny."),
            say("eli", "Wasn't trying for funny."),
            say_with("eli",
                "You still think being good at a job makes the job clean.",
                EmotionTag::Neutral,
            ),
            say("galen", "If you know something, say it plain."),
            say_with("eli",
                "Plain gets men killed faster than lies in the wrong company.",
                EmotionTag::Tense,
            ),
            say_with("eli",
                "If tomorrow turns ugly, don't stay where somebody else would \
                 write you standing.",
                EmotionTag::Tense,
            ),
            say_with("eli",
                "For what it's worth, Rook — I like you better before the world \
                 explains itself.",
                EmotionTag::Quiet,
            ),
        ],
        vec![
            choice("Head back to camp", vec![], to_scene("convoy_day3")),
        ],
        vec![],
    )
}

// ─── Day 3 ─────────────────────────────────────────────────────────

/// Day 3 — the last leg to Saint's Mile.
pub fn convoy_day3() -> Scene {
    scene(
        "convoy_day3", "road_to_saints_mile", "2d3",
        PacingTag::Pressure,
        vec![
            narrate_with(
                "Cold dawn. Thin light on tack and dust. Eli is already mounted, \
                 coat buttoned, no flask in sight.",
                EmotionTag::Tense,
            ),
            say("galen", "You sleep at all?"),
            say("eli", "Poorly. Helps with longevity."),
            say_with("eli",
                "You ever see a room where everybody's pretending not to smell smoke?",
                EmotionTag::Tense,
            ),
            say("galen", "No."),
            say_with("eli", "You're about to.", EmotionTag::Tense),
            say_with("nella",
                "Duty. Try not to get shot before I get a real stove under me.",
                EmotionTag::Warm,
            ),
        ],
        vec![
            choice("Approach the relay", vec![], to_scene("relay_arrival")),
        ],
        vec![
            memory("nella_biscuit_cloth"),
        ],
    )
}

/// Relay arrival — thirty seconds of dread.
pub fn relay_arrival() -> Scene {
    scene(
        "relay_arrival", "saints_mile_relay", "2d3",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The relay sits low and square against a wind that forgot how to \
                 move. Something is wrong before you can name it.",
                EmotionTag::Tense,
            ),
            narrate("No stable chatter. No bucket noise. One lantern already lit \
                     though the sun's not gone."),
            say("tom", "That's funny."),
            say("hester", "What is?"),
            say_with("tom", "Silence.", EmotionTag::Tense),
            narrate("Fresh ash under the telegraph window. Water trough stirred \
                     muddy, recent. A hitch rail with one rope still swinging."),
            say("galen", "Wire's dead."),
            say("bale", "Hold formation."),
            say_with("eli",
                "Now there's a sentence I never enjoy hearing.",
                EmotionTag::Dry,
            ),
            narrate("A man steps out onto the porch in relay clothes that don't \
                     fit him right."),
            say("tom", "Renshaw limps left."),
            narrate_with(
                "The man on the porch shifts his weight to the wrong leg. \
                 That is the last clean moment in the chapter.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice("Brace for it", vec![], to_combat("saints_mile_relay")),
        ],
        vec![],
    )
}

// ─── Post-Relay Scenes ─────────────────────────────────────────────

/// Triage choice — save one. Not all three.
pub fn relay_triage() -> Scene {
    scene(
        "relay_triage", "saints_mile_relay", "2d3",
        PacingTag::Crisis,
        vec![
            narrate_with(
                "The powder wagon blows against the wall. After that, the field \
                 stops reading like a battle map. It becomes fragments.",
                EmotionTag::Grief,
            ),
            narrate("Nella on the ground, reaching. Tom under a wagon tongue, \
                     pinned. Hester's papers loose in the wind."),
            narrate_with(
                "You have one clean act left.",
                EmotionTag::Tense,
            ),
        ],
        vec![
            choice(
                "Save Tom — pull him from the wreck",
                vec![
                    set_flag("relay_saved", true),
                    set_text("relay_branch", "tom"),
                    set_flag("nella_died", true),
                ],
                to_scene("relay_aftermath"),
            ),
            choice(
                "Save Nella — get her clear of the fire",
                vec![
                    set_flag("relay_saved", true),
                    set_text("relay_branch", "nella"),
                    set_flag("tom_died", true),
                ],
                to_scene("relay_aftermath"),
            ),
            choice(
                "Grab the papers — the truth matters more",
                vec![
                    set_flag("relay_saved", true),
                    set_text("relay_branch", "papers"),
                    set_flag("tom_died", true),
                    set_flag("nella_died", true),
                ],
                to_scene("relay_aftermath"),
            ),
        ],
        vec![],
    )
}

/// Relay aftermath — the poster is born.
pub fn relay_aftermath() -> Scene {
    scene_with_memory(
        "relay_aftermath", "saints_mile_relay", "2d3",
        PacingTag::Crisis,
        vec![
            // Eli's deed
            narrate_with(
                "Eli snatches the ledger from Hester's body. For one impossible \
                 instant, the whole world pauses to watch what kind of man he is.",
                EmotionTag::Tense,
            ),
            narrate("Not guilty. Not innocent. Decided."),
            narrate("Then he runs. Not away from the shooting. Through the burning \
                     gap by the relay side wall. He disappears into firelight with \
                     the ledger under his coat."),
            narrate_with(
                "From your angle, it is theft. From a wiser angle, maybe not. \
                 You do not get the wiser angle.",
                EmotionTag::Grief,
            ),
            // Branch-specific lines
            say_if_with("tom",
                "Not your setup. Hear me? Not — yours —",
                vec![flag_eq("relay_branch", "tom")],
                EmotionTag::Grief,
            ),
            say_if_with("nella",
                "You were here. Tell them you were here.",
                vec![flag_eq("relay_branch", "nella")],
                EmotionTag::Grief,
            ),
            say_if_with("narrator",
                "The papers matter later, but no one watching mistakes that act \
                 for innocence.",
                vec![flag_eq("relay_branch", "papers")],
                EmotionTag::Grief,
            ),
            // The accusation
            say_with("guard",
                "Rook! Rook, what did you do?",
                EmotionTag::Tense,
            ),
            narrate_with(
                "Not 'what happened.' 'What did you do.' That is the chapter's knife.",
                EmotionTag::Tense,
            ),
            // The poster
            narrate("A territorial rider arrives late enough to be legal about it."),
            say("rider", "Name."),
            say("galen", "Galen Rook."),
            narrate_with(
                "The rider writes it down like a verdict already spoken elsewhere. \
                 Behind him, the cut telegraph wire ticks in the wind.",
                EmotionTag::Grief,
            ),
        ],
        vec![],
        vec![
            // Dead Drop unlocks — the mechanical scar
            unlock("galen", "dead_drop"),
            set_flag("poster_born", true),
            set_flag("chapter2_complete", true),
        ],
        vec![
            MemoryRef {
                object: MemoryObjectId::new("wanted_poster"),
                callback_type: MemoryCallbackType::Transform,
                target_chapter: Some(ChapterId::new("ch3")),
            },
        ],
    )
}

// ─── Encounters ────────────────────────────────────────────────────

/// Red Switch Wash — competence on the road.
pub fn red_switch_wash_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("red_switch_wash"),
        phases: vec![CombatPhase {
            id: "wash_ambush".to_string(),
            description: "Mounted thieves test the convoy from high ground.".to_string(),
            enemies: vec![
                enemy_full("rider_a", "Mounted Thief", 22, 15, 8, 50, 9, 25, 5),
                enemy("rider_b", "Trail Bandit", 20, 12, 7, 45, 8),
            ],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: Some(Standoff {
            postures: vec![StandoffPosture::EarlyDraw, StandoffPosture::SteadyHand, StandoffPosture::Bait],
            allow_focus: true,
            eli_influence: false, // Eli is NPC this chapter
        }),
        party_slots: 4,
        terrain: Terrain {
            name: "Red Switch Wash".to_string(),
            cover: vec![
                CoverElement { name: "Wagon wheel".to_string(), durability: 40, destructible: true },
                CoverElement { name: "Creek bank".to_string(), durability: 80, destructible: false },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "survive_wash".to_string(),
                label: "Repel the raiders".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![],
                success_consequence: vec![set_flag("wash_survived", true)],
            },
            Objective {
                id: "protect_water".to_string(),
                label: "Protect the water cart".to_string(),
                objective_type: ObjectiveType::Secondary,
                fail_consequence: vec![set_flag("water_cart_damaged", true)],
                success_consequence: vec![set_flag("water_cart_intact", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Hollow Pump — ambiguity, bad targets.
pub fn hollow_pump_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("hollow_pump"),
        phases: vec![CombatPhase {
            id: "pump_scuffle".to_string(),
            description: "Laborers and rail hands arguing over water. Not cleanly enemies.".to_string(),
            enemies: vec![
                enemy_full("laborer_a", "Angry Laborer", 14, 8, 5, 30, 5, 10, 4),
                enemy_full("laborer_b", "Desperate Worker", 12, 6, 4, 25, 6, 5, 3),
                enemy_full("rail_hand", "Rail Hand", 18, 12, 7, 40, 7, 15, 5),
            ],
            npc_allies: vec![],
            entry_conditions: vec![],
            phase_effects: vec![],
        }],
        standoff: None, // Mixed-pressure, not a clean standoff
        party_slots: 4,
        terrain: Terrain {
            name: "Hollow Pump Waystation".to_string(),
            cover: vec![
                CoverElement { name: "Pump housing".to_string(), durability: 60, destructible: false },
                CoverElement { name: "Supply crates".to_string(), durability: 25, destructible: true },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "restore_order".to_string(),
                label: "Restore order at the pump".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![],
                success_consequence: vec![set_flag("pump_resolved", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Saint's Mile Relay — the break. Three-phase authored collapse.
pub fn saints_mile_relay_encounter() -> Encounter {
    Encounter {
        id: EncounterId::new("saints_mile_relay"),
        phases: vec![
            // Phase 1 — Familiar escort combat
            CombatPhase {
                id: "relay_phase1".to_string(),
                description: "Familiar escort combat. The player thinks they understand.".to_string(),
                enemies: vec![
                    enemy_full("attacker_a", "Relay Raider", 25, 18, 9, 55, 8, 20, 6),
                    enemy_full("attacker_b", "Relay Raider", 25, 18, 9, 55, 7, 20, 6),
                    enemy("attacker_c", "Armed Rider", 22, 15, 8, 50, 9),
                ],
                npc_allies: vec![
                    NpcCombatant {
                        character: CharacterId::new("bale"),
                        behavior: NpcBehavior::Professional,
                        hp: 35, nerve: 30,
                        speed: 7, accuracy: 55, damage: 12,
                    },
                ],
                entry_conditions: vec![],
                phase_effects: vec![],
            },
            // Phase 2 — The turn: ally flips, objectives shift
            CombatPhase {
                id: "relay_phase2".to_string(),
                description: "A rail guard shoots Bale. The powder wagon starts rolling. \
                              Men come through the gate that should have been safe.".to_string(),
                enemies: vec![
                    enemy_full("traitor_guard", "Turned Guard", 28, 20, 10, 60, 7, 10, 7),
                    enemy("gate_man", "Gate Enforcer", 24, 16, 9, 50, 6),
                    // Desperate local used as cover
                    enemy_full("desperate_local", "Frightened Local", 10, 5, 3, 20, 5, 5, 3),
                ],
                npc_allies: vec![], // Bale is down
                entry_conditions: vec![],
                phase_effects: vec![
                    StateEffect::SetFlag {
                        id: FlagId::new("bale_dead"),
                        value: FlagValue::Bool(true),
                    },
                ],
            },
            // Phase 3 — Flight under accusation
            CombatPhase {
                id: "relay_phase3".to_string(),
                description: "Surviving guards turn on Galen. The evidence points at him.".to_string(),
                enemies: vec![
                    enemy_full("accusing_guard_a", "Hostile Guard", 22, 18, 8, 55, 7, 10, 6),
                    enemy_full("accusing_guard_b", "Hostile Guard", 22, 18, 8, 55, 6, 10, 6),
                ],
                npc_allies: vec![], // No allies in phase 3
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
            name: "Saint's Mile Relay".to_string(),
            cover: vec![
                CoverElement { name: "Payroll coach".to_string(), durability: 50, destructible: true },
                CoverElement { name: "Relay wall".to_string(), durability: 80, destructible: false },
                CoverElement { name: "Water trough".to_string(), durability: 30, destructible: true },
            ],
            hazards: vec![],
        },
        objectives: vec![
            Objective {
                id: "survive_relay".to_string(),
                label: "Survive the ambush".to_string(),
                objective_type: ObjectiveType::Primary,
                fail_consequence: vec![],
                success_consequence: vec![set_flag("relay_survived", true)],
            },
        ],
        outcome_effects: vec![],
        escapable: true,
    }
}

/// Young-man Galen for Chapter 2 (age 24, more skills than youth).
pub fn young_man_galen() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![(
        "galen".to_string(), "Galen Rook".to_string(),
        35, 25, 10,  // hp, nerve, ammo
        13, 68, 9,   // speed, accuracy, damage — between youth and adult
        vec![
            SkillId::new("quick_draw"),
            SkillId::new("snap_shot"),
            SkillId::new("duck"),
            SkillId::new("steady_aim"),
            SkillId::new("trail_eye"),
            SkillId::new("called_shot_basic"),
            SkillId::new("cold_read"),
            SkillId::new("grit"),
            // dead_drop unlocked during relay
        ],
        vec![], // no duo techs — Eli is NPC
        vec![],
    )]
}

// ─── Scene Registry ────────────────────────────────────────────────

/// Get a Saint's Mile Convoy scene by ID.
/// Entry point: `convoy_join` (chapter 2 start).
pub fn get_scene(id: &str) -> Option<Scene> {
    match id {
        // Chapter entry point
        "convoy_join" => Some(convoy_join()),
        "convoy_day1_road" => Some(convoy_day1_road()),
        "night1_camp" => Some(night1_camp()),
        "night1_eli_talk" => Some(night1_eli_talk()),
        "convoy_day2" => Some(convoy_day2()),
        "night2_camp" => Some(night2_camp()),
        "night2_eli_walk" => Some(night2_eli_walk()),
        "convoy_day3" => Some(convoy_day3()),
        "relay_arrival" => Some(relay_arrival()),
        "relay_triage" => Some(relay_triage()),
        "relay_aftermath" => Some(relay_aftermath()),
        _ => None,
    }
}

pub fn get_encounter(id: &str) -> Option<Encounter> {
    match id {
        "red_switch_wash" => Some(red_switch_wash_encounter()),
        "hollow_pump" => Some(hollow_pump_encounter()),
        "saints_mile_relay" => Some(saints_mile_relay_encounter()),
        _ => None,
    }
}
