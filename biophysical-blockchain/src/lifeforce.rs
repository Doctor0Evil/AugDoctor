use crate::types::{BioTokenState, HostEnvelope, SystemAdjustment};
use crate::morph::{EvolveBudget, MorphBudget, MorphDelta, MorphRiskBands};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LifeforceError {
    #[error("forbidden: BRAIN would go below brain_min (death condition)")]
    BrainNegative,
    #[error("forbidden: BLOOD would reach or cross blood_min (unsafe depletion)")]
    BloodDepletion,
    #[error("forbidden: OXYGEN would reach or cross oxygen_min (unsafe depletion)")]
    OxygenDepletion,
    #[error("forbidden: NANO exceeds host nano_max_fraction envelope")]
    NanoOverEnvelope,
    #[error("forbidden: SMART autonomy would exceed smart_max or BRAIN")]
    SmartOverMax,
    #[error("eco-cost {0} exceeds host eco_flops_limit")]
    EcoOverLimit(f64),
    #[error("MORPH L1 norm would exceed EVOLVE corridor")]
    MorphExceedsEvolve,
    #[error("MORPH risk band violation without new evidence")]
    MorphRiskBandViolation,
}

/// Internal: enforce ‖M‖₁ ≤ E_evolve.
fn check_morph_vs_evolve(
    evolve: &EvolveBudget,
    morph_before: &MorphBudget,
    delta: &MorphDelta,
) -> Result<MorphBudget, LifeforceError> {
    let after = morph_before.plus(delta);
    let m_l1 = after.m_eco + after.m_cyber + after.m_neuro + after.m_smart;
    if m_l1 > evolve.evolve_total {
        return Err(LifeforceError::MorphExceedsEvolve);
    }
    Ok(after)
}

/// Internal: monotone safety in risk directions unless evidence present.
fn check_morph_risk_monotone(
    before: &MorphBudget,
    after: &MorphBudget,
    risk: &MorphRiskBands,
    has_new_evidence: bool,
) -> Result<(), LifeforceError> {
    if !has_new_evidence {
        if after.m_cyber > before.m_cyber && after.m_cyber > risk.max_cyber {
            return Err(LifeforceError::MorphRiskBandViolation);
        }
        if after.m_neuro > before.m_neuro && after.m_neuro > risk.max_neuro {
            return Err(LifeforceError::MorphRiskBandViolation);
        }
        if after.m_smart > before.m_smart && after.m_smart > risk.max_smart {
            return Err(LifeforceError::MorphRiskBandViolation);
        }
    }
    Ok(())
}

/// Apply a system adjustment under biophysical + EVOLVE/MORPH constraints.
/// This is the only place token-like balances are changed.[file:42][file:47]
pub fn apply_lifeforce_guarded_adjustment(
    state: &mut BioTokenState,
    env: &HostEnvelope,
    adj: &SystemAdjustment,
    morph_risk: &MorphRiskBands,
    has_new_evidence: bool,
) -> Result<(), LifeforceError> {
    // Projected biophysical tokens.
    let new_brain  = state.brain  + adj.delta_brain;
    let new_wave   = state.wave   + adj.delta_wave;
    let new_blood  = state.blood  + adj.delta_blood;
    let new_oxygen = state.oxygen + adj.delta_oxygen;
    let new_nano   = state.nano   + adj.delta_nano;
    let new_smart  = state.smart  + adj.delta_smart;

    if new_brain < env.brain_min {
        return Err(LifeforceError::BrainNegative);
    }
    if new_blood <= env.blood_min {
        return Err(LifeforceError::BloodDepletion);
    }
    if new_oxygen <= env.oxygen_min {
        return Err(LifeforceError::OxygenDepletion);
    }

    // BRAIN still governs safe ranges for SMART implicitly via envelope.
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

    // EVOLVE / MORPH invariants.
    let evolve_after = EvolveBudget {
        evolve_total: state.evolve.evolve_total,
        evolve_used:  (state.evolve.evolve_used + adj.delta_evolve).max(0.0),
    };
    if evolve_after.evolve_used > evolve_after.evolve_total {
        return Err(LifeforceError::MorphExceedsEvolve);
    }

    let morph_after = check_morph_vs_evolve(&evolve_after, &state.morph, &adj.delta_morph)?;
    check_morph_risk_monotone(&state.morph, &morph_after, morph_risk, has_new_evidence)?;

    // If all checks pass, commit.
    state.brain  = new_brain;
    state.wave   = new_wave;
    state.blood  = new_blood;
    state.oxygen = new_oxygen;
    state.nano   = new_nano;
    state.smart  = new_smart;
    state.evolve = evolve_after;
    state.morph  = morph_after;

    Ok(())
}
