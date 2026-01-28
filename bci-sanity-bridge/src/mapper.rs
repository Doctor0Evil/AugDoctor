use serde::{Serialize, Deserialize};
use biophysical_blockchain::{SystemAdjustment};
use bci_bioledger_bridge::BciEvent;

/// Host-local SANITY, DECAY, SCALE, RADS snapshot (non-financial).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PsychoEnvSnapshot {
    pub sanity: f32,      // 0.0–1.0: current psychological margin
    pub decay: f32,       // 0.0–1.0: isolation / stasis accumulation
    pub indoor_bias: f32, // 0.0–1.0: fraction of time in indoor tasks
    pub social_score: f32 // 0.0–1.0: recent social contact index
}

/// Derive SANITY / DECAY deltas from EEG-BCI event + snapshot.
pub fn map_bci_to_sanity_decay(
    event: &BciEvent,
    snap: &PsychoEnvSnapshot
) -> (f32, f32) {
    // Low risk + social context → SANITY recovery, DECAY relief.
    let r = event.riskscore.clamp(0.0, 1.0) as f32;
    let social_factor = snap.social_score.clamp(0.0, 1.0);
    let indoor_factor = snap.indoor_bias.clamp(0.0, 1.0);

    // Base magnitudes are small and bounded; envelopes clamp further.
    let base_recover = 0.02_f32;
    let base_decay    = 0.01_f32;

    // Recovery stronger when risk is low and social is high.
    let sanity_delta = base_recover * (1.0 - r) * (0.5 + 0.5 * social_factor);

    // DECAY grows with indoor_bias and risk; social mitigates it.
    let decay_delta = base_decay * (0.5 + 0.5 * indoor_factor) * (0.5 + 0.5 * r)
        * (1.0 - 0.4 * social_factor);

    (sanity_delta, decay_delta)
}

/// Optional: convert SANITY / DECAY into a small, eco-bounded SystemAdjustment
/// for SCALE / WAVE drain (e.g., when isolation is high).
pub fn sanity_decay_to_adjustment(
    sanity_delta: f32,
    decay_delta: f32,
    eco_cost: f64
) -> SystemAdjustment {
    let sane = sanity_delta as f64;
    let dec  = decay_delta as f64;

    // Example: small negative WAVE when DECAY high, small positive BRAIN when SANITY improves.
    let deltabrain =  0.001 * sane;
    let deltawave  = -0.001 * dec;
    let deltanano  =  0.0_f64;
    let deltasmart =  0.0_f64;

    SystemAdjustment {
        deltabrain,
        deltawave,
        deltablood: 0.0,
        deltaoxygen: 0.0,
        deltanano,
        deltasmart,
        ecocost: eco_cost.min(10.0),
        reason: format!("sanity-decay-update: ΔS={:.4},ΔD={:.4}", sane, dec),
    }
}
