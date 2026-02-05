//! Programmatic_Discipline: typed, host-bound “rights envelopes” that gate
//! how inner-ledger code is allowed to behave per host, per plane, per turn.
//!
//! All discipline structs here are:
//! - non-financial (no transfer, stake, or price fields),
//! - per-host and DID-anchored,
//! - enforced only inside sealed inner-ledger crates, never by AI-chat code.
//!
//! They are backed by ALN shards (e.g., neuromorph-evolution-budget.aln,
//! evolutionturnpolicy.aln) so parameters are host-authored, while core
//! invariants (no cross-host transfer, no special exemptions) live in code.[file:39][file:42]

use crate::sealed::inner::Sealed;
use crate::types::IdentityHeader;

/// Environment-plane markers for discipline envelopes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisciplinePlane {
    EvolutionTurn,
    DeepDomain,
    EcoWorkload,
    Access,
    Custom(&'static str),
}

/// Canonical trait for all Programmatic_Discipline structs.
///
/// Each implementation must be Sealed, so only this crate can define
/// discipline envelopes that gate mutation behaviors.[file:47]
pub trait ProgrammaticDiscipline: Sealed {
    /// Logical plane this discipline applies to.
    fn plane(&self) -> DisciplinePlane;

    /// Host DID this envelope is bound to.
    fn host_did(&self) -> &str;

    /// Optional profile / policy identifier (e.g., evolutionturnpolicy.aln).
    fn profile_id(&self) -> &str;

    /// Whether this envelope considers the given identity eligible to even
    /// *request* operations under it. Inner-ledger still applies deeper checks.
    fn is_identity_eligible(&self, id: &IdentityHeader) -> bool;
}
