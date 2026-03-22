//! Mission investigation — party-as-instrument truth assembly.
//!
//! No single character can solve the room. Each party member reads
//! a different domain. The revelation is collective.

use serde::{Deserialize, Serialize};
use crate::types::*;

/// A fragment of truth readable by a specific party member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationFragment {
    pub id: String,
    pub domain: InvestigationDomain,
    /// Which party member can read this.
    pub reader: CharacterId,
    /// What the fragment reveals.
    pub revelation: String,
    /// Whether this fragment has been read.
    pub discovered: bool,
}

/// The domains of investigation — each mapped to a party member's expertise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestigationDomain {
    /// Ada reads medical records, treatment patterns, mortality.
    Medical,
    /// Eli reads financial documents, transfers, money flows.
    Financial,
    /// Galen reads land grants, territorial claims, legal authority.
    LandGrant,
    /// Miriam reads the death register, absence patterns, unrecorded lives.
    DeathRegister,
    /// Lucien reads fire damage, blast patterns, professional destruction.
    FirePattern,
    /// Rosa reads the land itself — water, soil, terrain, what was here before.
    Terrain,
}

/// The assembled state of a multi-domain investigation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationState {
    pub fragments: Vec<InvestigationFragment>,
    /// Whether the full picture has been assembled.
    pub convergence_reached: bool,
    /// How many domains have been read.
    pub domains_read: Vec<InvestigationDomain>,
}

impl InvestigationState {
    /// Read a fragment with a specific party member.
    pub fn read_fragment(&mut self, fragment_id: &str, reader: &CharacterId) -> Option<&str> {
        if let Some(frag) = self.fragments.iter_mut().find(|f| f.id == fragment_id) {
            if frag.reader == *reader && !frag.discovered {
                frag.discovered = true;
                if !self.domains_read.contains(&frag.domain) {
                    self.domains_read.push(frag.domain);
                }
                return Some(&frag.revelation);
            }
        }
        None
    }

    /// Check if a specific domain has been read.
    pub fn domain_read(&self, domain: InvestigationDomain) -> bool {
        self.domains_read.contains(&domain)
    }

    /// Check convergence — all required domains read.
    pub fn check_convergence(&mut self, required: &[InvestigationDomain]) -> bool {
        let all_read = required.iter().all(|d| self.domains_read.contains(d));
        if all_read {
            self.convergence_reached = true;
        }
        all_read
    }

    /// Count discovered fragments.
    pub fn discovered_count(&self) -> usize {
        self.fragments.iter().filter(|f| f.discovered).count()
    }
}

/// Build the Burned Mission investigation.
pub fn burned_mission_investigation() -> InvestigationState {
    InvestigationState {
        fragments: vec![
            InvestigationFragment {
                id: "medical_records".to_string(),
                domain: InvestigationDomain::Medical,
                reader: CharacterId::new("ada"),
                revelation: "The mission's medical records show treatment patterns \
                             that predate the current fever by decades. The same water \
                             source, the same symptoms, the same communities.".to_string(),
                discovered: false,
            },
            InvestigationFragment {
                id: "financial_transfers".to_string(),
                domain: InvestigationDomain::Financial,
                reader: CharacterId::new("eli"),
                revelation: "Financial documents show the mission's original land \
                             grant was transferred, amended, and 'lost' through a \
                             chain of territorial re-filings. The money trail predates \
                             the railroad by forty years.".to_string(),
                discovered: false,
            },
            InvestigationFragment {
                id: "land_grants".to_string(),
                domain: InvestigationDomain::LandGrant,
                reader: CharacterId::new("galen"),
                revelation: "The mission's land grants prove the original claim \
                             covered the territory the rail is now claiming. The \
                             grants were never legally voided — they were burned.".to_string(),
                discovered: false,
            },
            InvestigationFragment {
                id: "death_register".to_string(),
                domain: InvestigationDomain::DeathRegister,
                reader: CharacterId::new("miriam"),
                revelation: "More names in the death register than markers in the \
                             cemetery. People were disappeared, not just killed. \
                             The count doesn't match. It has never matched.".to_string(),
                discovered: false,
            },
            InvestigationFragment {
                id: "fire_pattern".to_string(),
                domain: InvestigationDomain::FirePattern,
                reader: CharacterId::new("lucien"),
                revelation: "This was a job. Better than mine, but the same language. \
                             Directed ignition. Controlled enough to destroy the record \
                             rooms while leaving the walls standing long enough for \
                             people to believe the 'accident' story.".to_string(),
                discovered: false,
            },
            InvestigationFragment {
                id: "water_terrain".to_string(),
                domain: InvestigationDomain::Terrain,
                reader: CharacterId::new("rosa"),
                revelation: "The well sits on the valley's anchor water table. \
                             The mission founders built here because of the water. \
                             Whoever controls this well controls the valley. That \
                             has been true for eighty years.".to_string(),
                discovered: false,
            },
        ],
        convergence_reached: false,
        domains_read: Vec::new(),
    }
}
