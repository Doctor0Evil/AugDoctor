use serde::{Deserialize, Serialize};

/// Minimal ALN/DID identity header used for access control.
/// Must be verified by outer ALN/DID infrastructure before use.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityHeader {
    pub issuer_did: String,     // e.g., "bostrom18sd2u..." or "did:aln:..."
    pub subject_role: String,   // "augmented-citizen" | "authorized-researcher" | "system-daemon"
    pub network_tier: String,   // "inner-core" | "trusted-edge" | "sandbox"
    pub knowledge_factor: f32,  // 0.0 – 1.0; min threshold required for risky ops
}

/// Biophysical token state per host — inner, non-financial, non-transferable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioTokenState {
    pub brain: f64,
    pub wave: f64,
    pub blood: f64,
    pub oxygen: f64,
    pub nano: f64,
    pub smart: f64,
}

/// Host-level configuration and lifeforce limits.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostEnvelope {
    pub host_id: String,           // ALN/DID/Bostrom identifier of host
    pub brain_min: f64,            // must stay >= 0.0
    pub blood_min: f64,            // must stay  > 0.0
    pub oxygen_min: f64,           // must stay  > 0.0
    pub nano_max_fraction: f64,    // 0.0 – 1.0 fraction at which nano duties cap
    pub smart_max: f64,            // max allowed SMART autonomy
    pub eco_flops_limit: f64,      // eco FLOPs per epoch (host-bound)
}

/// System-requested adjustment to the inner ledger.
/// Only created by trusted system daemons, never by external users.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemAdjustment {
    pub delta_brain: f64,
    pub delta_wave: f64,
    pub delta_blood: f64,
    pub delta_oxygen: f64,
    pub delta_nano: f64,
    pub delta_smart: f64,
    /// Optional eco-cost (FLOPs, nJ, etc.) for accounting.
    pub eco_cost: f64,
    /// Operation label, e.g. "quantum-learning-step", "bioscale-upgrade".
    pub reason: String,
}
