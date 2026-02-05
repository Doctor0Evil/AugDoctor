//! Corridor gate checks for MORPH / POWER / ALN before any inner-ledger mutation.

use serde::{Deserialize, Serialize};
use aln_did_access::IdentityHeader;
use consent_governance::{ConsentVerifier, DemonstratedConsentShard};

/// Minimal ALN corridor descriptor, host-owned and non-financial.
/// Derived from ALN shards like neuromorph-eco-profile.aln and
/// neuromorph-evolution-budget.aln, not from token balances.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorProfile {
    /// Maximum morph depth for this call (0.0, 1.0].
    pub morph_limit: f32,
    /// Maximum power envelope (eco + WAVE budget) for this call.
    pub power_limit: f32,
    /// Required ALN profile ID that must be satisfied by the caller's DID/role.
    pub aln_profile_id: String,
    /// Minimum knowledge-factor required for this mutation (0.0, 1.0].
    pub required_knowledge_factor: f32,
}

/// Per-call corridor context assembled by orchestration layers.
/// This stays strictly per-host and non-financial.
#[derive(Clone, Debug)]
pub struct CorridorContext {
    /// Caller identity, including profile_id and knowledge_factor.
    pub identity: IdentityHeader,
    /// Host-resolved profile limits for this mutation corridor.
    pub profile: CorridorProfile,
    /// Host-resolved consent shard required for this mutation, if any.
    pub consent: Option<DemonstratedConsentShard>,
    /// Scalarized morph intensity for this mutation (0.0, 1.0].
    pub requested_morph: f32,
    /// Scalarized power cost (eco + WAVE) for this mutation (0.0, 1.0].
    pub requested_power: f32,
}

/// Unified gate error.
#[derive(Clone, Debug)]
pub enum CorridorError {
    MorphExceeded { requested: f32, limit: f32 },
    PowerExceeded { requested: f32, limit: f32 },
    AlnProfileMismatch { required: String, actual: String },
    KnowledgeTooLow { required: f32, actual: f32 },
    MissingConsent,
    ConsentRejected(String),
    IdentityRejected(String),
}

pub trait CorridorGate {
    fn check(&self, consent_verifier: &dyn ConsentVerifier) -> Result<(), CorridorError>;
}

impl CorridorGate for CorridorContext {
    fn check(&self, consent_verifier: &dyn ConsentVerifier) -> Result<(), CorridorError> {
        // MORPH gate.
        if self.requested_morph > self.profile.morph_limit {
            return Err(CorridorError::MorphExceeded {
                requested: self.requested_morph,
                limit: self.profile.morph_limit,
            });
        }

        // POWER gate.
        if self.requested_power > self.profile.power_limit {
            return Err(CorridorError::PowerExceeded {
                requested: self.requested_power,
                limit: self.profile.power_limit,
            });
        }

        // ALN / role profile gate (profile_id symmetry with access layer).
        let actual_profile = self.identity.profile_id.clone();
        if actual_profile != self.profile.aln_profile_id {
            return Err(CorridorError::AlnProfileMismatch {
                required: self.profile.aln_profile_id.clone(),
                actual: actual_profile,
            });
        }

        // Knowledge-factor symmetry with validate_identity_for_inner_ledger.
        let actual_k = self.identity.knowledge_factor;
        let required_k = self.profile.required_knowledge_factor;
        if actual_k < required_k {
            return Err(CorridorError::KnowledgeTooLow {
                required: required_k,
                actual: actual_k,
            });
        }

        // Consent gate for evolution / SMART-like changes only.
        //
        // Doctrine: evolution and SMART autonomy require DemonstratedConsentShard,
        // but trivial reversible operations can be allowed under lifeforce/eco guards
        // without a shard.[file:42][file:47]
        if self.profile.morph_limit > 0.0 {
            let shard = self
                .consent
                .as_ref()
                .ok_or(CorridorError::MissingConsent)?;
            if !consent_verifier.verify(shard, &self.identity) {
                return Err(CorridorError::ConsentRejected(
                    "DemonstratedConsentShard did not verify".to_string(),
                ));
            }
        }

        // Additional ALN/DID tier checks remain enforced in the inner ledger via
        // validate_identity_for_inner_ledger; we do not duplicate them here.[file:47]

        Ok(())
    }
}
