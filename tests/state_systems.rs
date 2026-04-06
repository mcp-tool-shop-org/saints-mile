//! Tests for state/argument.rs, state/evidence.rs, state/investigation.rs.
//!
//! These modules contain the game's truth-assembly systems.
//! Tests prove: argument reactions are stance-specific, evidence verification
//! is branch-aware, and investigation convergence requires collective reading.

use saints_mile::types::*;
use saints_mile::state::argument::{self, ReactionType};
use saints_mile::state::evidence::{self, RecordCategory};
use saints_mile::state::investigation::{self, InvestigationDomain};

// ─── Argument System ──────────────────────────────────────────────

/// Each water-claim stance produces different party reactions.
#[test]
fn water_claim_stances_produce_different_reactions() {
    let force = argument::water_claim_argument("force");
    let con = argument::water_claim_argument("con");
    let negotiate = argument::water_claim_argument("negotiate");

    assert_eq!(force.id, "water_claim");
    assert_eq!(force.chapter, "ch4");
    assert_eq!(force.player_stance, "force");
    assert_eq!(con.player_stance, "con");
    assert_eq!(negotiate.player_stance, "negotiate");

    // Force: Rosa advocates, Ada objects
    let rosa_force = force.reactions.iter()
        .find(|r| r.character == CharacterId::new("rosa")).unwrap();
    assert_eq!(rosa_force.response, ReactionType::Advocated);
    let ada_force = force.reactions.iter()
        .find(|r| r.character == CharacterId::new("ada")).unwrap();
    assert_eq!(ada_force.response, ReactionType::Objected);

    // Con: Rosa objects, Eli advocates
    let rosa_con = con.reactions.iter()
        .find(|r| r.character == CharacterId::new("rosa")).unwrap();
    assert_eq!(rosa_con.response, ReactionType::Objected);
    let eli_con = con.reactions.iter()
        .find(|r| r.character == CharacterId::new("eli")).unwrap();
    assert_eq!(eli_con.response, ReactionType::Advocated);

    // Negotiate: Ada advocates, others comply
    let ada_negotiate = negotiate.reactions.iter()
        .find(|r| r.character == CharacterId::new("ada")).unwrap();
    assert_eq!(ada_negotiate.response, ReactionType::Advocated);
}

/// Reactions carry position text explaining their stance.
#[test]
fn reactions_carry_position_text() {
    let force = argument::water_claim_argument("force");
    for reaction in &force.reactions {
        if reaction.response != ReactionType::Silent {
            assert!(!reaction.position.is_empty(),
                "{:?} should have a position when not silent", reaction.character);
        }
    }
}

/// Unknown stance produces empty reactions.
#[test]
fn unknown_stance_produces_empty_reactions() {
    let unknown = argument::water_claim_argument("unknown_stance");
    assert!(unknown.reactions.is_empty(),
        "unknown stance should produce no reactions");
}

/// Miriam is silent in Ch4 (not present yet).
#[test]
fn miriam_silent_in_ch4() {
    let force = argument::water_claim_argument("force");
    let miriam = force.reactions.iter()
        .find(|r| r.character == CharacterId::new("miriam"));
    assert!(miriam.is_some(), "Miriam should be in the reaction list");
    assert_eq!(miriam.unwrap().response, ReactionType::Silent,
        "Miriam should be silent — she is not present in Ch4");
}

// ─── Evidence System ──────────────────────────────────────────────

/// Iron ledger archive has records in expected categories.
#[test]
fn iron_ledger_archive_has_records() {
    let archive = evidence::iron_ledger_archive();
    assert!(archive.len() >= 7, "archive should have at least 7 records");

    // Must have route manifest, payroll, contract, land claim, medical, security
    let categories: Vec<_> = archive.iter().map(|r| r.category).collect();
    assert!(categories.contains(&RecordCategory::RouteManifest));
    assert!(categories.contains(&RecordCategory::PayrollLedger));
    assert!(categories.contains(&RecordCategory::ContractFiling));
    assert!(categories.contains(&RecordCategory::LandClaim));
    assert!(categories.contains(&RecordCategory::MedicalConsignment));
    assert!(categories.contains(&RecordCategory::SecurityFile));
}

/// Each relay branch verifies a different subset of records.
#[test]
fn branch_verification_is_branch_specific() {
    let archive = evidence::iron_ledger_archive();

    let tom_results = evidence::verify_against_branch(&archive, "tom");
    let nella_results = evidence::verify_against_branch(&archive, "nella");
    let papers_results = evidence::verify_against_branch(&archive, "papers");

    // All branches should verify at least some records
    assert!(!tom_results.is_empty(), "tom branch should verify some records");
    assert!(!nella_results.is_empty(), "nella branch should verify some records");
    assert!(!papers_results.is_empty(), "papers branch should verify some records");

    // All verified results should be marked as verified
    for result in &tom_results {
        assert!(result.verified);
        assert_eq!(result.relay_branch_used, "tom");
        assert!(!result.proof.is_empty(), "verified records must have proof text");
    }

    // Papers should verify land_acquisition_chain (exclusive to papers)
    assert!(papers_results.iter().any(|r| r.record_id == "land_acquisition_chain"),
        "papers branch should verify land acquisition chain");
}

/// Branch verification counts reflect the archive design.
#[test]
fn branch_verification_counts_are_consistent() {
    let archive = evidence::iron_ledger_archive();
    let counts = evidence::branch_verification_counts(&archive);

    assert_eq!(counts.len(), 3);
    for (branch, count) in &counts {
        assert!(*count > 0, "{} branch should verify at least one record", branch);
    }
}

/// Lucien's custody status determines additional archive access.
#[test]
fn lucien_archive_contribution_varies_by_status() {
    let forced = evidence::lucien_archive_contribution("forced_guide");
    let prisoner = evidence::lucien_archive_contribution("prisoner");
    let judged = evidence::lucien_archive_contribution("judged");
    let unknown = evidence::lucien_archive_contribution("unknown");

    // All valid statuses contribute something
    assert!(!forced.is_empty());
    assert!(!prisoner.is_empty());
    assert!(!judged.is_empty());
    assert!(unknown.is_empty(), "unknown status should contribute nothing");

    // Judged gives the most (speaks freely)
    assert!(judged.len() >= forced.len(),
        "judged Lucien should contribute at least as much as forced guide");
}

// ─── Investigation System ─────────────────────────────────────────

/// Burned mission investigation has fragments for all domains.
#[test]
fn burned_mission_has_all_domains() {
    let inv = investigation::burned_mission_investigation();

    assert_eq!(inv.fragments.len(), 6, "burned mission should have 6 fragments");
    assert!(!inv.convergence_reached);
    assert!(inv.domains_read.is_empty());

    // Each domain is represented
    let domains: Vec<_> = inv.fragments.iter().map(|f| f.domain).collect();
    assert!(domains.contains(&InvestigationDomain::Medical));
    assert!(domains.contains(&InvestigationDomain::Financial));
    assert!(domains.contains(&InvestigationDomain::LandGrant));
    assert!(domains.contains(&InvestigationDomain::DeathRegister));
    assert!(domains.contains(&InvestigationDomain::FirePattern));
    assert!(domains.contains(&InvestigationDomain::Terrain));
}

/// Reading a fragment with the correct reader reveals it.
#[test]
fn correct_reader_reveals_fragment() {
    let mut inv = investigation::burned_mission_investigation();

    // Ada reads medical records
    let result = inv.read_fragment("medical_records", &CharacterId::new("ada"));
    assert!(result.is_some(), "Ada should be able to read medical records");
    assert!(result.unwrap().contains("medical records"),
        "revelation should describe medical findings");

    assert_eq!(inv.discovered_count(), 1);
    assert!(inv.domain_read(InvestigationDomain::Medical));
    assert!(!inv.domain_read(InvestigationDomain::Financial));
}

/// Wrong reader cannot reveal a fragment.
#[test]
fn wrong_reader_cannot_reveal() {
    let mut inv = investigation::burned_mission_investigation();

    // Galen cannot read medical records (Ada's domain)
    let result = inv.read_fragment("medical_records", &CharacterId::new("galen"));
    assert!(result.is_none(), "Galen should not be able to read medical records");
    assert_eq!(inv.discovered_count(), 0);
}

/// Reading a fragment twice returns None (already discovered).
#[test]
fn fragment_cannot_be_read_twice() {
    let mut inv = investigation::burned_mission_investigation();

    let first = inv.read_fragment("medical_records", &CharacterId::new("ada"));
    assert!(first.is_some());

    let second = inv.read_fragment("medical_records", &CharacterId::new("ada"));
    assert!(second.is_none(), "already-discovered fragment should return None");
}

/// Convergence requires all required domains to be read.
#[test]
fn convergence_requires_all_domains() {
    let mut inv = investigation::burned_mission_investigation();

    let required = [
        InvestigationDomain::Medical,
        InvestigationDomain::Financial,
        InvestigationDomain::LandGrant,
    ];

    // Not converged after one domain
    inv.read_fragment("medical_records", &CharacterId::new("ada"));
    assert!(!inv.check_convergence(&required));
    assert!(!inv.convergence_reached);

    // Not converged after two domains
    inv.read_fragment("financial_transfers", &CharacterId::new("eli"));
    assert!(!inv.check_convergence(&required));

    // Converged after all three required domains
    inv.read_fragment("land_grants", &CharacterId::new("galen"));
    assert!(inv.check_convergence(&required));
    assert!(inv.convergence_reached);
}

/// Investigation state tracks discovered count correctly.
#[test]
fn discovered_count_tracks_correctly() {
    let mut inv = investigation::burned_mission_investigation();
    assert_eq!(inv.discovered_count(), 0);

    inv.read_fragment("medical_records", &CharacterId::new("ada"));
    assert_eq!(inv.discovered_count(), 1);

    inv.read_fragment("financial_transfers", &CharacterId::new("eli"));
    assert_eq!(inv.discovered_count(), 2);

    inv.read_fragment("land_grants", &CharacterId::new("galen"));
    inv.read_fragment("death_register", &CharacterId::new("miriam"));
    inv.read_fragment("fire_pattern", &CharacterId::new("lucien"));
    inv.read_fragment("water_terrain", &CharacterId::new("rosa"));
    assert_eq!(inv.discovered_count(), 6);
}
