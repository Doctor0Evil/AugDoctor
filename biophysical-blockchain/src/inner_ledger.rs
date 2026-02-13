use crate::access::validate_identity_for_inner_ledger;
use crate::consensus::{hash_state, LedgerEvent};
use crate::lifeforce::apply_lifeforce_guarded_adjustment;
use crate::types::{BioTokenState, HostEnvelope, IdentityHeader, SystemAdjustment};
use thiserror::Error;

/// Errors visible at orchestration / AI-Chat integration boundaries.
/// NOTE: There is no error variant that represents a reversible rollback.
/// All commits are append-only, biophysically-legible, and sovereignty-respecting.
#[derive(Debug, Error)]
pub enum InnerLedgerError {
    #[error("access error: {0}")]
    Access(#[from] crate::access::AccessError),

    #[error("lifeforce error: {0}")]
    Lifeforce(#[from] crate::lifeforce::LifeforceError),
}

/// Host-local inner ledger (no transfer, no bridge, no stake, no rollback). [file:10]
#[derive(Clone, Debug)]
pub struct InnerLedger {
    pub env: HostEnvelope,
    pub state: BioTokenState,
    pub last_state_hash: String,
}

impl InnerLedger {
    /// Initialize a new host ledger with safe starting values.
    /// The initial hash anchors sovereignty for this host only. [file:10]
    pub fn new(env: HostEnvelope, state: BioTokenState) -> Self {
        let hash = hash_state(&env.host_id, &env, &state);
        Self {
            env,
            state,
            last_state_hash: hash,
        }
    }

    /// System-only, lifeforce-guarded adjustment, with strict identity gating.
    ///
    /// This is the *only* mutation entrypoint.
    /// Properties:
    /// - Per-host only: `env.host_id` is the sole subject of mutation.
    /// - Non-financial: `SystemAdjustment` carries biophysical deltas only.
    /// - Non-reversible at capability level: no downgrade/rollback capability exists;
    ///   history is append-only, and any "correction" must be a new forward-safe delta.
    /// - Sovereign: requires valid ALN/DID/Bostrom identity and demonstrated consent
    ///   before any high-impact evolution or SMART change. [file:10]
    pub fn system_apply(
        &mut self,
        id_header: &IdentityHeader,
        required_knowledge_factor: f32,
        adj: &SystemAdjustment,
        timestamp_utc: &str,
    ) -> Result<LedgerEvent, InnerLedgerError> {
        // Awareness-check: InnerLedger never touches souls or consciousness,
        // only biophysical proxies (BRAIN/WAVE/BLOOD/OXYGEN/NANO/SMART) defined in BioTokenState. [file:10]

        // 1. Strict separation of mechanics: identity and tier guard.
        //    All actors (including authors, vendors, AI tools) must pass the same check.
        validate_identity_for_inner_ledger(id_header, required_knowledge_factor)?; [file:10]

        // 2. Lifeforce + eco guards: immutable soul, safe Tree-of-Life corridors only.
        //
        //    apply_lifeforce_guarded_adjustment:
        //    - Enforces BRAIN/BLOOD/OXYGEN floors.
        //    - Enforces SMART ≤ min(smart_max, BRAIN).
        //    - Enforces NANO ≤ host nano envelope.
        //    - Enforces eco-cost ≤ ecoflops_limit.
        //    - Rejects any HardStop band or eco-negative pattern as a hard veto. [file:10]
        //
        //    There is no branch here for "rollback" or "downgrade"; any unsafe request is simply rejected.
        apply_lifeforce_guarded_adjustment(&mut self.state, &self.env, adj)?; [file:10]

        // 3. Compute new hash for audit and consensus attestation.
        //    This is an append-only lineage; no function here can revert to a prior hash. [file:10]
        let new_hash = hash_state(&self.env.host_id, &self.env, &self.state);

        let event = LedgerEvent {
            host_id: self.env.host_id.clone(),
            prev_state_hash: self.last_state_hash.clone(),
            new_state_hash: new_hash.clone(),
            adjustment: adj.clone(),
            timestamp_utc: timestamp_utc.to_string(),
            attested_by: id_header.issuer_did.clone(),
        };

        // 4. Commit hash (one-way evolution).
        //    Once committed, this state is part of the immutable Tree-of-Life path
        //    for this host. Any future change must be a new, forward-safe adjustment,
        //    never a reversal of this commit. [file:10]
        self.last_state_hash = new_hash;

        Ok(event)
    }
}
