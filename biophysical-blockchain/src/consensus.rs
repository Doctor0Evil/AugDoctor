use crate::types::{BioTokenState, HostEnvelope, SystemAdjustment};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// A single inner-ledger event, describing a guarded adjustment and its context.[file:1]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerEvent {
    pub host_id: String,
    pub prev_state_hash: String,
    pub new_state_hash: String,
    pub adjustment: SystemAdjustment,
    pub timestamp_utc: String,  // ISO-8601
    pub attested_by: String,    // issuer_did of system-daemon / validator
}

/// Simple hash utility (quantum-safe placeholder: BLAKE3).[file:1]
pub fn hash_state(host_id: &str, env: &HostEnvelope, state: &BioTokenState) -> String {
    let mut hasher = Hasher::new();
    hasher.update(host_id.as_bytes());
    hasher.update(serde_json::to_string(env).unwrap().as_bytes());
    hasher.update(serde_json::to_string(state).unwrap().as_bytes());
    let hash = hasher.finalize();
    hash.to_hex().to_string()
}
