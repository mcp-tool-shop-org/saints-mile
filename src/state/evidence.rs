//! Evidence convergence system — institutional truth assembly.
//!
//! Not a file-browser minigame. A system that tracks what the player
//! has brought into the archive and what it proves when compared
//! against institutional records.

use serde::{Deserialize, Serialize};
use crate::types::*;

/// A piece of verifiable evidence in the archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRecord {
    pub id: String,
    pub category: RecordCategory,
    pub description: String,
    /// What this record proves when standing alone.
    pub standalone_proof: String,
    /// What this record proves when verified against player evidence.
    pub verified_proof: Option<String>,
    /// Which relay branch evidence this can be verified against.
    pub verifiable_by: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordCategory {
    RouteManifest,
    PayrollLedger,
    ContractFiling,
    LandClaim,
    MedicalConsignment,
    SecurityFile,
}

/// Result of comparing player evidence against archive records.
#[derive(Debug, Clone)]
pub struct ConvergenceResult {
    pub record_id: String,
    pub verified: bool,
    pub proof: String,
    pub relay_branch_used: String,
}

/// The Iron Ledger archive — records that can be verified against player evidence.
pub fn iron_ledger_archive() -> Vec<ArchiveRecord> {
    vec![
        ArchiveRecord {
            id: "route_manifest_sm".to_string(),
            category: RecordCategory::RouteManifest,
            description: "Convoy route filing for the Saint's Mile corridor".to_string(),
            standalone_proof: "Shows the convoy route was filed as standard freight".to_string(),
            verified_proof: Some(
                "Route records confirm the convoy was set up along a corridor \
                 designed to fail at the relay. The road WAS wrong — by design.".to_string()
            ),
            verifiable_by: vec!["tom".to_string()],
        },
        ArchiveRecord {
            id: "payroll_ledger_convoy".to_string(),
            category: RecordCategory::PayrollLedger,
            description: "Payroll records for convoy escort and support staff".to_string(),
            standalone_proof: "Lists standard payroll for convoy operations".to_string(),
            verified_proof: Some(
                "Payroll names include people Nella knew — convoy staff, camp workers. \
                 Names that were people are now line items in a budget.".to_string()
            ),
            verifiable_by: vec!["nella".to_string()],
        },
        ArchiveRecord {
            id: "contract_demolition".to_string(),
            category: RecordCategory::ContractFiling,
            description: "Demolition contracts signed by intermediaries".to_string(),
            standalone_proof: "Shows contracted demolition work in the corridor".to_string(),
            verified_proof: Some(
                "Demolition contracts authorized by a territorial official whose name \
                 connects to Voss's jurisdiction. Lucien's work has a paper trail.".to_string()
            ),
            verifiable_by: vec!["tom".to_string(), "nella".to_string(), "papers".to_string()],
        },
        ArchiveRecord {
            id: "land_acquisition_chain".to_string(),
            category: RecordCategory::LandClaim,
            description: "Land-acquisition filings timed to follow demolition events".to_string(),
            standalone_proof: "Shows land transfers in the corridor region".to_string(),
            verified_proof: Some(
                "Each demolition was followed by a claim amendment within days. \
                 The destruction was a filing strategy, not frontier accident.".to_string()
            ),
            verifiable_by: vec!["papers".to_string()],
        },
        ArchiveRecord {
            id: "medical_routing".to_string(),
            category: RecordCategory::MedicalConsignment,
            description: "Medical supply routing memo".to_string(),
            standalone_proof: "Shows medical shipment routing through the corridor".to_string(),
            verified_proof: Some(
                "Black Willow's fever medicine was redirected, not lost — routed \
                 to a secondary depot that served the rail extension, not the \
                 settlement.".to_string()
            ),
            verifiable_by: vec!["tom".to_string(), "nella".to_string(), "papers".to_string()],
        },
        ArchiveRecord {
            id: "sheriff_security_ref".to_string(),
            category: RecordCategory::SecurityFile,
            description: "Security file referencing Sheriff Mercer".to_string(),
            standalone_proof: "References inquiry from a territorial sheriff".to_string(),
            verified_proof: Some(
                "Sheriff Mercer's name in a file labeled 'inquiries — referred \
                 to regional security.' He was being tracked by the same system \
                 he was investigating.".to_string()
            ),
            verifiable_by: vec!["tom".to_string(), "nella".to_string(), "papers".to_string()],
        },
        ArchiveRecord {
            id: "double_payroll".to_string(),
            category: RecordCategory::PayrollLedger,
            description: "Payroll records showing dual employment".to_string(),
            standalone_proof: "Shows some personnel on multiple payrolls".to_string(),
            verified_proof: Some(
                "Saint's Mile relay guards were on two payrolls — the Briar Line's \
                 and a private security fund with a territorial authorization stamp. \
                 The betrayal was budgeted.".to_string()
            ),
            verifiable_by: vec!["papers".to_string(), "nella".to_string()],
        },
    ]
}

/// Attempt to verify archive records against the player's relay branch.
/// Returns which records can be verified and what they prove.
pub fn verify_against_branch(archive: &[ArchiveRecord], relay_branch: &str) -> Vec<ConvergenceResult> {
    archive.iter()
        .filter(|r| r.verifiable_by.contains(&relay_branch.to_string()))
        .map(|r| ConvergenceResult {
            record_id: r.id.clone(),
            verified: true,
            proof: r.verified_proof.clone().unwrap_or_else(|| r.standalone_proof.clone()),
            relay_branch_used: relay_branch.to_string(),
        })
        .collect()
}

/// Count how many records each branch can verify.
pub fn branch_verification_counts(archive: &[ArchiveRecord]) -> Vec<(&str, usize)> {
    let branches = ["tom", "nella", "papers"];
    branches.iter()
        .map(|b| (*b, archive.iter().filter(|r| r.verifiable_by.contains(&b.to_string())).count()))
        .collect()
}

/// Check if Lucien's custody state enables additional archive access.
pub fn lucien_archive_contribution(lucien_status: &str) -> Vec<String> {
    match lucien_status {
        "forced_guide" => vec![
            "contract_demolition".to_string(),
            // Forced guide can read the demolition contract language
            // and identify authorization shorthand
        ],
        "prisoner" => vec![
            // Prisoner gives info reluctantly — only confirms what's found
            "contract_demolition".to_string(),
        ],
        "judged" => vec![
            // Judged may speak more freely about the system
            "contract_demolition".to_string(),
            "land_acquisition_chain".to_string(),
        ],
        _ => vec![],
    }
}
