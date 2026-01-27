use crate::types::{BioTokenState, HostEnvelope, SystemAdjustment};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LifeforceError {
    #[error("forbidden: BRAIN would go negative (death condition)")]
    BrainNegative,
    #[error("forbidden: BLOOD would reach or cross zero (unsafe depletion)")]
    BloodDepletion,
    #[error("forbidden: OXYGEN would reach or cross zero (unsafe depletion)")]
    OxygenDepletion,
    #[error("forbidden: NANO exceeds host nano_max_fraction envelope")]
    NanoOverEnvelope,
    #[error("forbidden: SMART autonomy would exceed smart_max")]
    SmartOverMax,
    #[error("eco-cost {0} exceeds host eco_flops_limit")]
    EcoOverLimit(f64),
}

/// Apply a system adjustment under biophysical constraints.
/// This is the **only** place token-like balances are changed.[file:1]
pub fn apply_lifeforce_guarded_adjustment(
    state: &mut BioTokenState,
    env: &HostEnvelope,
    adj: &SystemAdjustment,
) -> Result<(), LifeforceError> {
    let new_brain = state.brain + adj.delta_brain;
    let new_wave = state.wave + adj.delta_wave;
    let new_blood = state.blood + adj.delta_blood;
    let new_oxygen = state.oxygen + adj.delta_oxygen;
    let new_nano = state.nano + adj.delta_nano;
    let new_smart = state.smart + adj.delta_smart;

    if new_brain < env.brain_min {
        return Err(LifeforceError::BrainNegative);
    }
    if new_blood <= env.blood_min {
        return Err(LifeforceError::BloodDepletion);
    }
    if new_oxygen <= env.oxygen_min {
        return Err(LifeforceError::OxygenDepletion);
    }

    // BRAIN still governs safe ranges for WAVE / SMART implicitly via envelope.
    if new_smart > env.smart_max || new_smart > new_brain {
        return Err(LifeforceError::SmartOverMax);
    }

    // NANO compliance envelope: fraction of eco_flops_limit encoded as state.nano.
    if new_nano > env.nano_max_fraction {
        return Err(LifeforceError::NanoOverEnvelope);
    }

    // Eco envelope: forbid operations that exceed eco budget.
    if adj.eco_cost > env.eco_flops_limit {
        return Err(LifeforceError::EcoOverLimit(adj.eco_cost));
    }

    // If all checks pass, commit.
    state.brain = new_brain;
    state.wave = new_wave;
    state.blood = new_blood;
    state.oxygen = new_oxygen;
    state.nano = new_nano;
    state.smart = new_smart;

    Ok(())
}
