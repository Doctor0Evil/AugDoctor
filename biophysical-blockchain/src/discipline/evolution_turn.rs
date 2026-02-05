//! EvolutionTurnDiscipline: temporal + reversibility discipline for
//! evolution turns per host.
//!
//! Backed by neuromorph-evolution-budget.aln and evolutionturnpolicy.aln,
//! and enforced by daily turn logic plus irreversible-token rules.[file:39][file:42]

use serde::{Deserialize, Serialize};

use crate::discipline::programmatic::{DisciplinePlane, ProgrammaticDiscipline};
use crate::sealed::inner::Sealed;
use crate::types::IdentityHeader;

/// Host-authored evolution turn discipline envelope.
///
/// This struct is loaded from ALN shards and then treated as a hard
/// constraint for InnerLedger::system_apply on any evolution-related
/// SystemAdjustment.[file:39][file:42]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionTurnDiscipline {
    /// Canonical host DID (Bostrom / DID-ALN).
    pub host_did: String,

    /// Policy profile ID, e.g. "EvolutionTurnPolicy2026v1".
    pub profile_id: String,

    /// Maximum evolution turns per UTC day (governed by ALN, capped in code). 
    pub max_turns_per_day: u8,

    /// Minimum seconds between turns (spacing discipline).
    pub min_seconds_between_turns: u32,

    /// Whether burst mode is allowed (packing many micro-steps into one meta-turn).
    pub allow_burst: bool,

    /// Whether experimental irreversible patterns are allowed at all.
    pub allow_experimental_irreversible: bool,

    /// Whether an irreversible token is required when any pattern is irreversible.
    pub require_irreversible_token: bool,

    /// Microspace sovereignty profile ID this discipline assumes.
    pub microspace_profile_id: String,
}

impl Sealed for EvolutionTurnDiscipline {}

impl ProgrammaticDiscipline for EvolutionTurnDiscipline {
    fn plane(&self) -> DisciplinePlane {
        DisciplinePlane::EvolutionTurn
    }

    fn host_did(&self) -> &str {
        &self.host_did
    }

    fn profile_id(&self) -> &str {
        &self.profile_id
    }

    fn is_identity_eligible(&self, id: &IdentityHeader) -> bool {
        // Only identities from the same host-DID namespace and non-sandbox roles
        // may even *request* evolution turns; deeper checks still run elsewhere.[file:42][file:47]
        if !id.issuerdid.starts_with("bostrom")
            && !id.issuerdid.starts_with("didaln")
            && !id.issuerdid.starts_with("did:")
        {
            return false;
        }
        if id.networktier == "sandbox" {
            return false;
        }
        true
    }
}

impl EvolutionTurnDiscipline {
    /// Enforce hard ceilings and non-financial invariants on loaded values.
    ///
    /// This ensures ALN cannot weaken core safety properties; it may only
    /// choose values within the allowed corridor (e.g., max 10 turns/day).[file:42]
    pub fn normalized(self) -> Self {
        let capped_turns = self.max_turns_per_day.min(10).max(0);
        let min_spacing = self.min_seconds_between_turns.max(60);

        Self {
            host_did: self.host_did,
            profile_id: self.profile_id,
            max_turns_per_day: capped_turns,
            min_seconds_between_turns: min_spacing,
            allow_burst: false, // hard-coded: burst not allowed in inner ledger
            allow_experimental_irreversible: self.allow_experimental_irreversible,
            require_irreversible_token: self.require_irreversible_token,
            microspace_profile_id: self.microspace_profile_id,
        }
    }
}
