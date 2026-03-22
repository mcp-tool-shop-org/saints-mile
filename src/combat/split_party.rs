//! Split-party operations — assignment as moral argument.
//!
//! The assignment IS the argument. Choosing who goes where reveals
//! what the player values: force, persuasion, evidence, protection, trust.

use serde::{Deserialize, Serialize};
use crate::types::*;

/// A split-party operation with parallel objectives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitOperation {
    /// Named teams with assigned members.
    pub teams: Vec<Team>,
    /// What each team accomplished.
    pub results: Vec<TeamResult>,
    /// Whether recombination has happened.
    pub recombined: bool,
}

/// A team in a split operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub objective: String,
    pub members: Vec<CharacterId>,
    /// How well this pairing works (affects outcome quality).
    pub synergy: TeamSynergy,
}

/// How well a team's members work together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamSynergy {
    /// Members work well together.
    Strong,
    /// Functional but tense.
    Functional,
    /// Volatile — high risk, high potential.
    Volatile,
    /// Hostile — real chance of failure or internal damage.
    Hostile,
}

/// What a team accomplished (generated after the split resolves).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamResult {
    pub team_id: String,
    pub success: bool,
    /// The narrative report — character consequence, not just mission outcome.
    pub report: String,
    /// Relationship changes caused by the pairing.
    pub relationship_deltas: Vec<(CharacterId, CharacterId, i32)>,
    /// Flags set by this team's outcome.
    pub flags: Vec<(String, FlagValue)>,
}

/// Evaluate a team's synergy based on its members.
pub fn evaluate_synergy(members: &[CharacterId]) -> TeamSynergy {
    let ids: Vec<&str> = members.iter().map(|m| m.0.as_str()).collect();

    // Strong pairings
    if ids.contains(&"galen") && ids.contains(&"eli") { return TeamSynergy::Strong; }
    if ids.contains(&"rosa") && ids.contains(&"miriam") { return TeamSynergy::Strong; }
    if ids.contains(&"galen") && ids.contains(&"ada") { return TeamSynergy::Strong; }
    if ids.contains(&"eli") && ids.contains(&"ada") { return TeamSynergy::Strong; }

    // Volatile pairings
    if ids.contains(&"eli") && ids.contains(&"lucien") { return TeamSynergy::Volatile; }
    if ids.contains(&"miriam") && ids.contains(&"lucien") { return TeamSynergy::Volatile; }

    // Hostile pairings
    if ids.contains(&"rosa") && ids.contains(&"lucien") { return TeamSynergy::Hostile; }

    TeamSynergy::Functional
}

/// Generate a team result based on the team's composition and objective.
pub fn resolve_team(team: &Team) -> TeamResult {
    let ids: Vec<&str> = team.members.iter().map(|m| m.0.as_str()).collect();

    let (success, report, deltas) = match (team.id.as_str(), &team.synergy) {
        // Wire office teams
        ("wire_office", TeamSynergy::Strong) => (
            true,
            format!("{} held the wire office. The dispatch got through.",
                team.members.iter().map(|m| m.0.as_str()).collect::<Vec<_>>().join(" and ")),
            vec![],
        ),
        ("wire_office", _) => (
            true,
            format!("{} held the wire office under pressure.",
                team.members.iter().map(|m| m.0.as_str()).collect::<Vec<_>>().join(" and ")),
            vec![],
        ),

        // Signal tower teams
        ("signal_tower", TeamSynergy::Hostile) => {
            // Rosa + Lucien: hostile but effective
            let report = if ids.contains(&"rosa") && ids.contains(&"lucien") {
                "Rosa held Pine Signal. Lucien disarmed the mast charge. \
                 Rosa has not thanked him. She won't.".to_string()
            } else {
                "The signal tower held, barely.".to_string()
            };
            (true, report, vec![
                (CharacterId::new("rosa"), CharacterId::new("lucien"), 1), // grudging professional respect
            ])
        },
        ("signal_tower", _) => (
            true,
            format!("{} secured the signal tower.",
                team.members.iter().map(|m| m.0.as_str()).collect::<Vec<_>>().join(" and ")),
            vec![],
        ),

        // Witness teams
        ("witness_route", _) if ids.contains(&"eli") && ids.contains(&"miriam") => (
            true,
            "Miriam and Eli kept the witness safe. Neither agrees on how.".to_string(),
            vec![
                (CharacterId::new("eli"), CharacterId::new("miriam"), 2),
            ],
        ),
        ("witness_route", _) => (
            true,
            format!("{} secured the witness route.",
                team.members.iter().map(|m| m.0.as_str()).collect::<Vec<_>>().join(" and ")),
            vec![],
        ),

        // Default
        _ => (
            true,
            format!("Team {} completed their objective.", team.id),
            vec![],
        ),
    };

    TeamResult {
        team_id: team.id.clone(),
        success,
        report,
        relationship_deltas: deltas,
        flags: if success {
            vec![(format!("{}_held", team.id), FlagValue::Bool(true))]
        } else {
            vec![(format!("{}_lost", team.id), FlagValue::Bool(true))]
        },
    }
}

/// Build a split operation for the Long Wire chapter.
pub fn long_wire_split(
    wire_team: Vec<CharacterId>,
    signal_team: Vec<CharacterId>,
    witness_team: Vec<CharacterId>,
) -> SplitOperation {
    let wire_synergy = evaluate_synergy(&wire_team);
    let signal_synergy = evaluate_synergy(&signal_team);
    let witness_synergy = evaluate_synergy(&witness_team);

    let teams = vec![
        Team {
            id: "wire_office".to_string(),
            name: "Wire Office".to_string(),
            objective: "Hold the telegraph office and transmit the party's version".to_string(),
            members: wire_team,
            synergy: wire_synergy,
        },
        Team {
            id: "signal_tower".to_string(),
            name: "Signal Tower".to_string(),
            objective: "Secure the relay tower and delay the counter-narrative".to_string(),
            members: signal_team,
            synergy: signal_synergy,
        },
        Team {
            id: "witness_route".to_string(),
            name: "Witness Route".to_string(),
            objective: "Protect the witness and get their testimony through".to_string(),
            members: witness_team,
            synergy: witness_synergy,
        },
    ];

    let results = teams.iter().map(resolve_team).collect();

    SplitOperation {
        teams,
        results,
        recombined: false,
    }
}
