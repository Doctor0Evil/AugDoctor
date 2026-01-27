// path: aln-did-access/src/lib.rs

//! Biophysical access control for shared sovereignty.
//!
//! This crate encodes shared safeguards for biophysical‑blockchain access.
//! Roles and identities are not about status or privilege; they describe
//! responsibilities and safety boundaries in a cooperative system where
//! every host keeps full sovereignty over its own biophysical state.
//!
//! - `AugmentedCitizen` identifies a host (or its surrogate) that directly
//!   governs its own evolution, lifeforce corridors, and device behavior.
//! - `AuthorizedResearcher` identifies contributors who help improve tools,
//!   protocols, and analytics under strict consent, neurorights, and
//!   non‑financial rules, never above the host, always in service to it.
//! - `SystemDaemon` is a technical helper process that implements
//!   host‑defined policies (lifeforce guards, eco limits, civic rules)
//!   without any independent economic interest or ownership.
//! - `Observer` represents read‑only, audit‑oriented views that support
//!   transparency and accountability without touching inner state.
//!
//! All identities must be anchored in ALN/DID/Bostrom spaces that uphold
//! neurorights, biophysical safety, and non‑financial design. Access checks
//! in this crate ensure that every interaction with the inner ledger is
//! host‑centered, consent‑respecting, and cannot be used to limit anyone’s
//! evolution or augmentation rights.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoleClass {
    AugmentedCitizen,
    AuthorizedResearcher,
    SystemDaemon,
    Observer,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityHeader {
    /// ALN/DID/Bostrom identifier (e.g., `bostrom18...` or `did:aln:...`).
    pub issuer_did: String,
    /// Responsibility role, never a status marker.
    pub subject_role: RoleClass,
    /// Execution tier: inner‑core, trusted‑edge, or sandbox.
    pub network_tier: String,
    /// Normalized biophysics/ALN knowledge factor in [0.0, 1.0].
    pub knowledge_factor: f32,
}
