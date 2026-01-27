use crate::types::BciEvent;
use biophysical_blockchain::SystemAdjustment;

/// Map BCI intent + risk → bounded SystemAdjustment.
/// No direct user control, no financial semantics.
pub fn map_bci_to_adjustment(event: &BciEvent) -> SystemAdjustment {
    // Normalize risk to [0, 1].
    let r = event.risk_score.clamp(0.0, 1.0) as f64;

    // Base magnitudes are intentionally small, host envelope will clamp further.
    let base_brain = 0.002;     // small credit for completed compute
    let base_wave  = 0.0015;    // tiny learning load per event
    let base_nano  = 0.0005;    // nano envelope usage
    let base_smart = 0.001;     // slight autonomy gain for stable operation

    // Higher risk → more conservative (smaller positive deltas).
    let safety_factor = 1.0 - r * 0.8;

    let delta_brain = base_brain * safety_factor;
    let delta_wave  = base_wave  * safety_factor;
    let delta_nano  = base_nano  * safety_factor;
    let delta_smart = base_smart * safety_factor;

    // BLOOD/OXYGEN are *not* spent here by default.
    // They are guarded by HostEnvelope minima and lifeforce module.
    SystemAdjustment {
        delta_brain,
        delta_wave,
        delta_blood: 0.0,
        delta_oxygen: 0.0,
        delta_nano,
        delta_smart,
        eco_cost: event.eco_cost_estimate,
        reason: format!(
            "bci-intent:{}:channel:{}:risk:{:.3}",
            event.intent_label, event.channel, event.risk_score
        ),
    }
}
