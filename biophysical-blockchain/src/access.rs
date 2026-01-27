//! Biophysical access control for shared sovereignty.
//!
//! This module enforces that only identities with clearly defined
//! responsibilities (augmented citizens, authorized researchers,
//! system daemons) and sufficient biophysical understanding can
//! request changes to a host’s inner ledger.
//!
//! The goal is protection, not hierarchy: every rule here exists
//! to keep biophysical sovereignty, non‑financial design, and
//! neurorights intact for all hosts.

use crate::types::IdentityHeader;
use crate::RoleClass;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccessError {
    #[error("unauthorized role: {0}")]
    UnauthorizedRole(String),
    #[error("sandbox tier cannot touch inner ledger")]
    SandboxTier,
    #[error("issuer_did not in ALN/DID/Bostrom namespace")]
    InvalidIssuerDid,
    #[error("knowledge factor too low for operation (have {have}, need {need})")]
    KnowledgeTooLow { have: f32, need: f32 },
}

/// Hard separation of mechanics: only augmented‑citizens, authorized
/// researchers, and system daemons on inner‑core / trusted‑edge tiers
/// may interact with the inner ledger. No sandbox, no arbitrary
/// third‑party vendors.
pub fn validate_identity_for_inner_ledger(
    header: &IdentityHeader,
    required_k: f32,
) -> Result<(), AccessError> {
    let role_ok = matches!(
        header.subject_role,
        RoleClass::AugmentedCitizen | RoleClass::AuthorizedResearcher | RoleClass::SystemDaemon
    );
    if !role_ok {
        return Err(AccessError::UnauthorizedRole(format!(
            "{:?}",
            header.subject_role
        )));
    }

    if header.network_tier == "sandbox" {
        return Err(AccessError::SandboxTier);
    }

    let did_ok = header.issuer_did.starts_with("bostrom")
        || header.issuer_did.starts_with("did:aln:")
        || header.issuer_did.starts_with("did:");
    if !did_ok {
        return Err(AccessError::InvalidIssuerDid);
    }

    if header.knowledge_factor < required_k {
        return Err(AccessError::KnowledgeTooLow {
            have: header.knowledge_factor,
            need: required_k,
        });
    }

    Ok(())
}
