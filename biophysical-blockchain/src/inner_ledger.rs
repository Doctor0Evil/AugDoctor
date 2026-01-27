use crate::access::validate_identity_for_inner_ledger;
use crate::consensus::{hash_state, LedgerEvent};
use crate::lifeforce::apply_lifeforce_guarded_adjustment;
use crate::types::{BioTokenState, HostEnvelope, IdentityHeader, SystemAdjustment};
use thiserror::Error;

/// Errors visible at orchestration / AI-Chat integration boundaries.
#[derive(Debug, Error)]
pub enum InnerLedgerError {
    #[error("access error: {0}")]
    Access(#[from] crate::access::AccessError),
    #[error("lifeforce error: {0}")]
    Lifeforce(#[from] crate::lifeforce::LifeforceError),
}

/// Host-local inner ledger (no transfer, no bridge, no stake).[file:1]
#[derive(Clone, Debug)]
pub struct InnerLedger {
    pub env: HostEnvelope,
    pub state: BioTokenState,
    pub last_state_hash: String,
}

impl InnerLedger {
    /// Initialize a new host ledger with safe starting values.
    pub fn new(env: HostEnvelope, state: BioTokenState) -> Self {
        let hash = hash_state(&env.host_id, &env, &state);
        Self {
            env,
            state,
            last_state_hash: hash,
        }
    }

    /// System-only, lifeforce-guarded adjustment, with strict identity gating.
    /// Called by trusted orchestration (e.g., bioscale upgrades, quantum-learning). [file:1]
    pub fn system_apply(
        &mut self,
        id_header: &IdentityHeader,
        required_knowledge_factor: f32,
        adj: &SystemAdjustment,
        timestamp_utc: &str,
    ) -> Result<LedgerEvent, InnerLedgerError> {
        // 1. Strict separation of mechanics: identity and tier guard.
        validate_identity_for_inner_ledger(id_header, required_knowledge_factor)?;

        // 2. Biophysical / eco lifeforce guard: immutable soul, safe corridors only.
        apply_lifeforce_guarded_adjustment(&mut self.state, &self.env, adj)?;

        // 3. Compute new hash for audit and consensus attestation.
        let new_hash = hash_state(&self.env.host_id, &self.env, &self.state);

        let event = LedgerEvent {
            host_id: self.env.host_id.clone(),
            prev_state_hash: self.last_state_hash.clone(),
            new_state_hash: new_hash.clone(),
            adjustment: adj.clone(),
            timestamp_utc: timestamp_utc.to_string(),
            attested_by: id_header.issuer_did.clone(),
        };

        // 4. Commit hash.
        self.last_state_hash = new_hash;

        Ok(event)
    }
}
