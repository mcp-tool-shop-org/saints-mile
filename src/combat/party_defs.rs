//! Party member definitions — stats and skills per age phase.
//!
//! The command menu carries biography. Young Galen is not adult Galen
//! with smaller numbers. He is a different fighter because life changed him.

use crate::types::*;
use crate::combat::types::*;

/// Party member template — generates live combat data for an age phase.
pub struct PartyTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub hp: i32,
    pub nerve: i32,
    pub ammo: i32,
    pub speed: i32,
    pub accuracy: i32,
    pub damage: i32,
    pub skills: Vec<SkillId>,
    pub duo_techs: Vec<DuoTechId>,
}

/// Build Galen for a specific age phase.
pub fn galen(phase: AgePhase) -> PartyTemplate {
    match phase {
        AgePhase::Youth => PartyTemplate {
            id: "galen", name: "Galen Rook",
            hp: 30, nerve: 20, ammo: 8,
            speed: 14, accuracy: 65, damage: 7,
            skills: vec![
                SkillId::new("quick_draw"),
                SkillId::new("snap_shot"),
                SkillId::new("duck"),
                SkillId::new("sprint"),
            ],
            duo_techs: vec![],
        },
        AgePhase::YoungMan => PartyTemplate {
            id: "galen", name: "Galen Rook",
            hp: 35, nerve: 25, ammo: 10,
            speed: 13, accuracy: 68, damage: 9,
            skills: vec![
                SkillId::new("quick_draw"),
                SkillId::new("snap_shot"),
                SkillId::new("duck"),
                SkillId::new("steady_aim"),
                SkillId::new("trail_eye"),
                SkillId::new("called_shot_basic"),
                SkillId::new("cold_read"),
                SkillId::new("grit"),
            ],
            duo_techs: vec![],
        },
        AgePhase::Adult => PartyTemplate {
            id: "galen", name: "Galen Rook",
            hp: 40, nerve: 30, ammo: 12,
            speed: 12, // slower than youth — judgment replaces reflex
            accuracy: 70, // more accurate
            damage: 10, // harder-hitting
            skills: vec![
                SkillId::new("quick_draw"),
                SkillId::new("called_shot"),   // upgraded from basic
                SkillId::new("take_cover"),     // replaces duck — positional, not reactive
                SkillId::new("trail_sense"),    // passive upgrade from trail_eye
                SkillId::new("rally"),          // first outward Command skill
                SkillId::new("setup_shot"),     // party synergy — marks target
                SkillId::new("overwatch"),      // patience as power
                SkillId::new("steady_aim"),     // Voss's gift persists
                SkillId::new("cold_read"),
                SkillId::new("grit"),
            ],
            duo_techs: vec![
                DuoTechId::new("loaded_deck"),    // Galen + Eli
                DuoTechId::new("stay_with_me"),   // Galen + Ada (Ch3 unlock)
            ],
        },
        AgePhase::Older => PartyTemplate {
            id: "galen", name: "Galen Rook",
            hp: 35, nerve: 35, ammo: 10,
            speed: 8,     // much slower — hand is damaged
            accuracy: 75, // most accurate he's ever been
            damage: 12,   // hardest-hitting — one shot, certain
            skills: vec![
                SkillId::new("called_shot_precise"),  // one shot, chosen with certainty
                SkillId::new("overwatch"),             // his signature now
                SkillId::new("initiative_read"),       // predicts enemy action
                SkillId::new("party_command"),         // directs others' turns
                SkillId::new("judgment_shot"),         // devastating single strike
                SkillId::new("steady_aim"),            // the gift that outlives the giver
                SkillId::new("cold_read"),
                SkillId::new("grit"),
            ],
            duo_techs: vec![
                DuoTechId::new("loaded_deck"),
                DuoTechId::new("stay_with_me"),
            ],
        },
    }
}

/// Build Eli for the adult party.
pub fn eli_adult() -> PartyTemplate {
    PartyTemplate {
        id: "eli", name: "Eli Winter",
        hp: 30, nerve: 25, ammo: 8,
        speed: 10, accuracy: 50, damage: 6,
        skills: vec![
            SkillId::new("sidearm"),
            SkillId::new("fast_talk"),
            SkillId::new("quick_hands"),
            SkillId::new("bluff"),
            SkillId::new("dirty_trick"),
            SkillId::new("patch_up"),
            SkillId::new("read_the_room"),   // Ch3 unlock
            SkillId::new("double_down"),     // Ch3 unlock
            // Loyalty line: grayed out, not here yet
        ],
        duo_techs: vec![
            DuoTechId::new("loaded_deck"),
            DuoTechId::new("second_opinion"), // Eli + Ada
        ],
    }
}

/// Build Ada Mercer.
pub fn ada() -> PartyTemplate {
    PartyTemplate {
        id: "ada", name: "Dr. Ada Mercer",
        hp: 25, nerve: 30, ammo: 4,  // derringer only
        speed: 8, accuracy: 40, damage: 4,
        skills: vec![
            SkillId::new("treat_wounds"),
            SkillId::new("tourniquet"),
            SkillId::new("steady_nerves"),
            SkillId::new("smelling_salts"),
            SkillId::new("derringer"),
            // Diagnosis line unlocks Ch4
            // Tonics line unlocks Ch5
        ],
        duo_techs: vec![
            DuoTechId::new("stay_with_me"),   // Ada + Galen
            DuoTechId::new("second_opinion"), // Ada + Eli
        ],
    }
}

impl PartyTemplate {
    /// Convert to the tuple format the combat engine expects.
    pub fn to_combat_tuple(&self) -> (String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>) {
        (
            self.id.to_string(),
            self.name.to_string(),
            self.hp, self.nerve, self.ammo,
            self.speed, self.accuracy, self.damage,
            self.skills.clone(),
            self.duo_techs.clone(),
            vec![],
        )
    }
}

/// Build the Ch3 adult party (Galen + Eli + Ada after join).
pub fn ch3_party() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![
        galen(AgePhase::Adult).to_combat_tuple(),
        eli_adult().to_combat_tuple(),
        ada().to_combat_tuple(),
    ]
}

/// Build the Ch3 party before Ada joins (Galen + Eli only).
pub fn ch3_party_pre_ada() -> Vec<(String, String, i32, i32, i32, i32, i32, i32, Vec<SkillId>, Vec<DuoTechId>, Vec<Wound>)> {
    vec![
        galen(AgePhase::Adult).to_combat_tuple(),
        eli_adult().to_combat_tuple(),
    ]
}
