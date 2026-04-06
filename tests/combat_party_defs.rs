//! Tests for combat/party_defs.rs — party member definitions per age phase.

use saints_mile::types::*;
use saints_mile::combat::party_defs;

#[test]
fn galen_youth_has_basic_skills() {
    let g = party_defs::galen(AgePhase::Youth);
    assert_eq!(g.id, "galen");
    assert_eq!(g.name, "Galen Rook");
    assert_eq!(g.skills.len(), 4);
    assert!(g.skills.iter().any(|s| s.0 == "quick_draw"));
    assert!(g.skills.iter().any(|s| s.0 == "snap_shot"));
    assert!(g.skills.iter().any(|s| s.0 == "duck"));
    assert!(g.skills.iter().any(|s| s.0 == "sprint"));
    // Youth has no duo techs
    assert!(g.duo_techs.is_empty());
}

#[test]
fn galen_ages_gain_skills_lose_speed() {
    let youth = party_defs::galen(AgePhase::Youth);
    let young_man = party_defs::galen(AgePhase::YoungMan);
    let adult = party_defs::galen(AgePhase::Adult);
    let older = party_defs::galen(AgePhase::Older);

    // Speed decreases with age — judgment replaces reflex
    assert!(youth.speed > young_man.speed);
    assert!(young_man.speed > adult.speed);
    assert!(adult.speed > older.speed);

    // Accuracy increases with age
    assert!(youth.accuracy < young_man.accuracy);
    assert!(young_man.accuracy < adult.accuracy);
    assert!(adult.accuracy < older.accuracy);

    // Skills grow through young_man and adult, then refine in older
    assert!(young_man.skills.len() > youth.skills.len());
    assert!(adult.skills.len() > young_man.skills.len());
}

#[test]
fn galen_adult_has_duo_techs() {
    let adult = party_defs::galen(AgePhase::Adult);
    assert_eq!(adult.duo_techs.len(), 2);
    assert!(adult.duo_techs.iter().any(|d| d.0 == "loaded_deck"));
    assert!(adult.duo_techs.iter().any(|d| d.0 == "stay_with_me"));
}

#[test]
fn galen_older_is_most_accurate_slowest() {
    let older = party_defs::galen(AgePhase::Older);
    assert_eq!(older.accuracy, 75, "older Galen: most accurate ever");
    assert_eq!(older.speed, 8, "older Galen: much slower — hand is damaged");
    assert_eq!(older.damage, 12, "older Galen: hardest-hitting — one shot, certain");
    // Unique skills: judgment_shot, party_command, initiative_read
    assert!(older.skills.iter().any(|s| s.0 == "judgment_shot"));
    assert!(older.skills.iter().any(|s| s.0 == "party_command"));
    assert!(older.skills.iter().any(|s| s.0 == "initiative_read"));
}

#[test]
fn ch3_party_has_three_members() {
    let party = party_defs::ch3_party();
    assert_eq!(party.len(), 3);
    assert_eq!(party[0].0, "galen");
    assert_eq!(party[1].0, "eli");
    assert_eq!(party[2].0, "ada");
}

#[test]
fn ch3_pre_ada_has_two_members() {
    let party = party_defs::ch3_party_pre_ada();
    assert_eq!(party.len(), 2);
    assert_eq!(party[0].0, "galen");
    assert_eq!(party[1].0, "eli");
}

#[test]
fn ch4_party_adds_rosa() {
    let party = party_defs::ch4_party();
    assert_eq!(party.len(), 4);
    assert_eq!(party[3].0, "rosa");
}

#[test]
fn ch5_roster_exceeds_party_slots() {
    let roster = party_defs::ch5_roster();
    assert_eq!(roster.len(), 5, "5 members forces swap decisions");
    // All five are distinct
    let ids: Vec<&str> = roster.iter().map(|r| r.0.as_str()).collect();
    assert!(ids.contains(&"galen"));
    assert!(ids.contains(&"eli"));
    assert!(ids.contains(&"ada"));
    assert!(ids.contains(&"rosa"));
    assert!(ids.contains(&"miriam"));
}

#[test]
fn ada_is_the_medic() {
    let ada = party_defs::ada();
    assert_eq!(ada.id, "ada");
    assert!(ada.skills.iter().any(|s| s.0 == "treat_wounds"));
    assert!(ada.skills.iter().any(|s| s.0 == "tourniquet"));
    assert!(ada.skills.iter().any(|s| s.0 == "steady_nerves"));
    // Low combat stats — she's a doctor, not a fighter
    assert_eq!(ada.ammo, 4, "derringer only");
    assert!(ada.accuracy < 50);
}

#[test]
fn rosa_is_physical_not_fast() {
    let rosa = party_defs::rosa();
    assert_eq!(rosa.id, "rosa");
    assert!(rosa.skills.iter().any(|s| s.0 == "lariat"));
    assert!(rosa.skills.iter().any(|s| s.0 == "brace"));
    assert_eq!(rosa.damage, 11, "hardest-hitting besides older Galen");
    assert_eq!(rosa.speed, 9, "deliberate, not fast");
}

#[test]
fn to_combat_tuple_preserves_values() {
    let g = party_defs::galen(AgePhase::Youth);
    let t = g.to_combat_tuple();
    assert_eq!(t.0, "galen");
    assert_eq!(t.1, "Galen Rook");
    assert_eq!(t.2, g.hp);
    assert_eq!(t.3, g.nerve);
    assert_eq!(t.4, g.ammo);
    assert_eq!(t.5, g.speed);
    assert_eq!(t.6, g.accuracy);
    assert_eq!(t.7, g.damage);
    assert_eq!(t.8.len(), g.skills.len());
    assert_eq!(t.9.len(), g.duo_techs.len());
    assert!(t.10.is_empty(), "combat tuple starts with no wounds");
}
