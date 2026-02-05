//! Corridor gate checks for MORPH / POWER / ALN before any inner-ledger mutation.

use serde::{Deserialize, Serialize};
use aln_did_access::IdentityHeader;
use consent_governance::{ConsentVerifier, DemonstratedConsentShard};

/// Minimal ALN corridor descriptor, host-owned and non-financial.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorProfile {
    /// Maximum morph depth for this call (0.0, 1.0].
    pub morph_limit: f32,
    /// Maximum power envelope (e.g. eco + WAVE budget) for this call.
    pub power_limit: f32,
    /// Required ALN profile ID that must be satisfied by the caller's DID/role.
    pub aln_profile_id: String,
}

/// Per-call corridor context assembled by orchestration layers.
#[derive(Clone, Debug)]
pub struct CorridorContext {
    pub identity: IdentityHeader,
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
    MissingConsent,
    ConsentRejected(String),
    IdentityRejected(String),
}

pub trait CorridorGate {
    fn check(&self, consent_verifier: &dyn ConsentVerifier) -> Result<(), CorridorError>;
}

impl CorridorGate for CorridorContext {
    fn check(&self, consent_verifier: &dyn ConsentVerifier) -> Result<(), CorridorError> {
        // MORPH gate
        if self.requested_morph > self.profile.morph_limit {
            return Err(CorridorError::MorphExceeded {
                requested: self.requested_morph,
                limit: self.profile.morph_limit,
            });
        }

        // POWER gate
        if self.requested_power > self.profile.power_limit {
            return Err(CorridorError::PowerExceeded {
                requested: self.requested_power,
                limit: self.profile.power_limit,
            });
        }

        // ALN / role profile gate (simple example: compare profile IDs).
        let actual = self.identity.profile_id.clone();
        if !actual.eq(&self.profile.aln_profile_id) {
            return Err(CorridorError::AlnProfileMismatch {
                required: self.profile.aln_profile_id.clone(),
                actual,
            });
        }

        // Consent gate for evolution / SMART changes.
        if let Some(shard) = &self.consent {
            if !consent_verifier.verify(shard, &self.identity) {
                return Err(CorridorError::ConsentRejected(
                    "DemonstratedConsentShard did not verify".to_string(),
                ));
            }
        } else {
            return Err(CorridorError::MissingConsent);
        }

        // You can add additional ALN/DID tier checks here, but the critical point
        // is that every mutation call must pass through this function.

        Ok(())
    }
}
