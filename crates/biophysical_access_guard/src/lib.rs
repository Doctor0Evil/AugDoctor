use augdoctor_core::biophysical::plane_classifier::{ConsciousnessState, EnvironmentPlane};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors for access guard violations.
#[derive(Debug, Error)]
pub enum AccessGuardError {
    #[error("unauthorized role: {0}")]
    UnauthorizedRole(String),
    #[error("invalid identity: {0}")]
    InvalidIdentity(String),
    #[error("insufficient knowledge factor: required {0}, provided {1}")]
    InsufficientKnowledge(f32, f32),
    #[error("cloning attempted on consciousness-linked asset")]
    CloningViolation,
    #[error("external system access denied")]
    ExternalAccessDenied,
}

/// Valid roles for biophysical-blockchain access.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AccessRole {
    AugmentedCitizen,
    AuthorizedResearcher,
    SystemDaemon,
}

/// Identity structure bound to DID/ALN/Bostrom.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SovereignIdentity {
    pub did: String,  // e.g., "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    pub knowledge_factor: f32,  // Minimum 0.7 for access, based on biophysics/cybernetics provability
    pub role: AccessRole,
    pub consciousness_state: ConsciousnessState,  // Immutable, sticky
}

/// Guard result with eco-reward points.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuardResult {
    pub allowed: bool,
    pub eco_points_earned: u64,  // Reward for compliant access
    pub audit_hex: String,  // Hex-encoded audit trail
}

/// Biophysical access guard enforcing strict separation.
pub struct BiophysicalAccessGuard {
    min_knowledge_factor: f32,
}

impl BiophysicalAccessGuard {
    pub fn new() -> Self {
        Self {
            min_knowledge_factor: 0.7,
        }
    }

    /// Enforce access for a biophysical operation.
    pub fn enforce_access(
        &self,
        identity: &SovereignIdentity,
        operation: &str,  // e.g., "spend_token", "append_rope"
        plane: EnvironmentPlane,
    ) -> Result<GuardResult, AccessGuardError> {
        // Role check: Only allowed roles
        if !matches!(identity.role, AccessRole::AugmentedCitizen | AccessRole::AuthorizedResearcher | AccessRole::SystemDaemon) {
            return Err(AccessGuardError::UnauthorizedRole(identity.role.to_string()));
        }

        // Identity validation: Must start with valid prefix
        if !identity.did.starts_with("did:aln:") && !identity.did.starts_with("bostrom") {
            return Err(AccessGuardError::InvalidIdentity(identity.did.clone()));
        }

        // Knowledge factor: Minimum for biophysics/cybernetics access
        if identity.knowledge_factor < self.min_knowledge_factor {
            return Err(AccessGuardError::InsufficientKnowledge(self.min_knowledge_factor, identity.knowledge_factor));
        }

        // Consciousness-state: Immutable, reject if not sticky
        if identity.consciousness_state != ConsciousnessState::ActiveImmutable {
            return Err(AccessGuardError::CloningViolation);
        }

        // Plane check: Reject external or non-biophysical planes
        if !matches!(plane, EnvironmentPlane::Biophysics | EnvironmentPlane::Bioscale) {
            return Err(AccessGuardError::ExternalAccessDenied);
        }

        // Operation-specific: e.g., neural-rope appends cannot introduce cloning
        if operation == "append_rope" && identity.consciousness_state == ConsciousnessState::ActiveImmutable {
            // Ensure no cloning by validating state (logic expanded as needed)
        }

        // Success: Grant access, earn eco-points for compliance
        let eco_points = if identity.knowledge_factor > 0.9 { 10 } else { 5 };
        let audit_hex = format!("0x{:X}", Uuid::new_v4().as_u128());

        Ok(GuardResult {
            allowed: true,
            eco_points_earned: eco_points,
            audit_hex,
        })
    }
}
