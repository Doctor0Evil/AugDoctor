use serde::{Deserialize, Serialize};
use biophysical_blockchain::{
    BioTokenState, HostEnvelope, IdentityHeader, InnerLedger, InnerLedgerError, SystemAdjustment,
};
use crate::domain::{NeuromorphPlane, NeuromorphDomain, NeuromorphPlaneTag};
use crate::observation::{NeuromorphEvent, NeuromorphEventKind};
use crate::orchestrator::{ReflexMicroOrchestrator, ReflexMicroPolicy, ReflexProposal};
use crate::policy_basic::BasicReflexPolicy;

/// Plane + domain tagging for all neuromorph reflex operations.
pub fn neuromorph_plane_tag(domain: NeuromorphDomain) -> NeuromorphPlaneTag {
    NeuromorphPlaneTag {
        plane: NeuromorphPlane::NeuromorphReflex,
        domain,
    }
}

/// Hard doctrine: neuromorph reflex lane is evolution-neutral.
/// It only ever calls InnerLedger::system_apply under lifeforce guards.
pub struct NeuromorphReflexSense<'a> {
    pub ledger: &'a mut InnerLedger,
    pub host_env: HostEnvelope,
    pub orchestrator: ReflexMicroOrchestrator<BasicReflexPolicy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphReflexResult {
    pub applied: bool,
    pub reason: String,
    pub plane: String,
    pub domain: String,
    pub prev_state_hash: String,
    pub new_state_hash: Option<String>,
}

/// Helper: wrap a SystemAdjustment in an InnerLedger call, with identity gating.
fn apply_adjustment_guarded(
    ledger: &mut InnerLedger,
    id_header: &IdentityHeader,
    required_k: f32,
    adj: SystemAdjustment,
    timestamp_utc: &str,
) -> Result<String, InnerLedgerError> {
    let prev_hash = ledger.laststatehash.clone();
    let evt = ledger.systemapply(id_header.clone(), required_k, adj, timestamp_utc)?;
    Ok(evt.newstatehash)
}

impl<'a> NeuromorphReflexSense<'a> {
    pub fn new(ledger: &'a mut InnerLedger, host_env: HostEnvelope) -> Self {
        let orchestrator = ReflexMicroOrchestrator::new(BasicReflexPolicy);
        Self { ledger, host_env, orchestrator }
    }

    /// Shadow mode: compute proposals but never touch the ledger.
    pub fn shadow_step(
        &self,
        state: &BioTokenState,
        event: &NeuromorphEvent,
    ) -> Option<ReflexProposal> {
        self.orchestrator.shadow_step(state, event)
    }

    /// Live mode: full-capability reflex path for this host.
    /// External actors cannot block it; failure only occurs on lifeforce or identity violation.
    pub fn live_step(
        &mut self,
        id_header: &IdentityHeader,
        required_k: f32,
        event: &NeuromorphEvent,
        timestamp_utc: &str,
    ) -> NeuromorphReflexResult {
        let plane = format!("{:?}", event.plane.plane);
        let domain = format!("{:?}", event.plane.domain);
        let prev_hash = self.ledger.laststatehash.clone();

        // Read current state snapshot.
        let state_snapshot = self.ledger.state.clone();

        // Compute proposal (small, deterministic).
        let maybe_prop = self.orchestrator.shadow_step(&state_snapshot, event);
        if maybe_prop.is_none() {
            return NeuromorphReflexResult {
                applied: false,
                reason: "no-reflex-proposal".to_string(),
                plane,
                domain,
                prev_state_hash: prev_hash,
                new_state_hash: None,
            };
        }

        let prop = maybe_prop.unwrap();
        let mut adj = prop.adjustment.clone();

        // Full-capability but still corridor-bounded:
        // scale is small; InnerLedger + lifeforce decide final limits.
        let result = apply_adjustment_guarded(
            self.ledger,
            id_header,
            required_k,
            adj,
            timestamp_utc,
        );

        match result {
            Ok(new_hash) => NeuromorphReflexResult {
                applied: true,
                reason: "lifeforce-guarded-reflex-applied".to_string(),
                plane,
                domain,
                prev_state_hash: prev_hash,
                new_state_hash: Some(new_hash),
            },
            Err(e) => NeuromorphReflexResult {
                applied: false,
                reason: format!("lifeforce-or-identity-guard-rejected: {:?}", e),
                plane,
                domain,
                prev_state_hash: prev_hash,
                new_state_hash: None,
            },
        }
    }
}
