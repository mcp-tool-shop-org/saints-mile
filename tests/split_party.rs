//! Tests for combat/split_party.rs — team assignment, synergy, resolution.

use saints_mile::types::*;
use saints_mile::combat::split_party::*;

#[test]
fn evaluate_synergy_strong_pairings() {
    // Galen + Eli: strong
    assert_eq!(
        evaluate_synergy(&[CharacterId::new("galen"), CharacterId::new("eli")]),
        TeamSynergy::Strong
    );
    // Rosa + Miriam: strong
    assert_eq!(
        evaluate_synergy(&[CharacterId::new("rosa"), CharacterId::new("miriam")]),
        TeamSynergy::Strong
    );
    // Galen + Ada: strong
    assert_eq!(
        evaluate_synergy(&[CharacterId::new("galen"), CharacterId::new("ada")]),
        TeamSynergy::Strong
    );
}

#[test]
fn evaluate_synergy_volatile_and_hostile() {
    // Eli + Lucien: volatile
    assert_eq!(
        evaluate_synergy(&[CharacterId::new("eli"), CharacterId::new("lucien")]),
        TeamSynergy::Volatile
    );
    // Miriam + Lucien: volatile
    assert_eq!(
        evaluate_synergy(&[CharacterId::new("miriam"), CharacterId::new("lucien")]),
        TeamSynergy::Volatile
    );
    // Rosa + Lucien: hostile
    assert_eq!(
        evaluate_synergy(&[CharacterId::new("rosa"), CharacterId::new("lucien")]),
        TeamSynergy::Hostile
    );
}

#[test]
fn long_wire_split_creates_three_teams() {
    let op = long_wire_split(
        vec![CharacterId::new("galen"), CharacterId::new("eli")],
        vec![CharacterId::new("rosa"), CharacterId::new("lucien")],
        vec![CharacterId::new("eli"), CharacterId::new("miriam")],
    );

    assert_eq!(op.teams.len(), 3);
    assert_eq!(op.teams[0].id, "wire_office");
    assert_eq!(op.teams[1].id, "signal_tower");
    assert_eq!(op.teams[2].id, "witness_route");

    // Wire office: Galen + Eli = Strong
    assert_eq!(op.teams[0].synergy, TeamSynergy::Strong);
    // Signal tower: Rosa + Lucien = Hostile
    assert_eq!(op.teams[1].synergy, TeamSynergy::Hostile);

    // All teams should have results
    assert_eq!(op.results.len(), 3);
    assert!(!op.recombined, "split starts uncombined");
}

#[test]
fn resolve_team_produces_flags() {
    let team = Team {
        id: "wire_office".to_string(),
        name: "Wire Office".to_string(),
        objective: "Hold the telegraph".to_string(),
        members: vec![CharacterId::new("galen"), CharacterId::new("eli")],
        synergy: TeamSynergy::Strong,
    };

    let result = resolve_team(&team);
    assert!(result.success);
    assert!(result.report.contains("dispatch got through"));
    assert!(result.flags.iter().any(|(k, _)| k == "wire_office_held"));
}

#[test]
fn hostile_signal_tower_still_succeeds() {
    let team = Team {
        id: "signal_tower".to_string(),
        name: "Signal Tower".to_string(),
        objective: "Secure the relay tower".to_string(),
        members: vec![CharacterId::new("rosa"), CharacterId::new("lucien")],
        synergy: TeamSynergy::Hostile,
    };

    let result = resolve_team(&team);
    assert!(result.success, "hostile pairing is ugly but effective");
    assert!(result.report.contains("Rosa"));
    // Grudging professional respect
    assert!(!result.relationship_deltas.is_empty());
}

#[test]
fn witness_route_eli_miriam_special_report() {
    let team = Team {
        id: "witness_route".to_string(),
        name: "Witness Route".to_string(),
        objective: "Protect the witness".to_string(),
        members: vec![CharacterId::new("eli"), CharacterId::new("miriam")],
        synergy: TeamSynergy::Functional,
    };

    let result = resolve_team(&team);
    assert!(result.success);
    assert!(result.report.contains("Miriam and Eli"));
    assert!(result.report.contains("Neither agrees"));
}
