use serde::{Deserialize, Serialize};

/// Canonical BCI event passed into the bridge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BciEvent {
    pub session_id: String,
    pub host_id: String,           // maps to HostEnvelope.host_id
    pub environment_id: String,
    pub channel: String,           // "eeg", "emg", "eye-tracker", ...
    pub intent_label: String,      // "grab", "rotate", "scroll", ...
    pub risk_score: f32,           // classifier-estimated risk
    pub latency_budget_ms: u32,
    pub token_budget: u32,
    pub eco_cost_estimate: f64,    // FLOPs / nJ for this step
}

/// Result of a full BCI → inner-ledger path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BciLedgerResult {
    pub session_id: String,
    pub host_id: String,
    pub intent_label: String,
    pub applied: bool,
    pub reason: String,
    pub prev_state_hash: Option<String>,
    pub new_state_hash: Option<String>,
}
