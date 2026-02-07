use crate::types::{
    BioTokenState,
    EcoBandProfile,
    HostEnvelope,
    LifeforceBand,
    LifeforceBandSeries,
    SafetyCurveWave,
    SystemAdjustment,
    SystemDomain,
};
use crate::morph::{EvolveBudget, MorphBudget, MorphDelta, MorphRiskBands};

use neural_roping::pain_corridor::PainCorridorSignal;

use thiserror::Error;

/// Errors raised by lifeforce, eco, WAVE, PainCorridor, and EVOLVE/MORPH guards.
#[derive(Debug, Error)]
pub enum LifeforceError {
    #[error("BRAIN would go below host minimum (death condition)")]
    BrainNegative,
    #[error("BLOOD would reach or cross zero (unsafe depletion)")]
    BloodDepletion,
    #[error("OXYGEN would reach or cross zero (unsafe depletion)")]
    OxygenDepletion,
    #[error("SMART autonomy would exceed host smart_max or BRAIN")]
    SmartOverMax,
    #[error("NANO exceeds host nano_max_fraction envelope")]
    NanoOverEnvelope,
    #[error("eco-cost {0} exceeds host eco_flops_limit")]
    EcoOverLimit(f64),
    #[error("lifeforce band is in HardStop")]
    LifeforceHardStop,
    #[error("WAVE would exceed safe ceiling under current fatigue")]
    WaveOverSafeCeiling,
    #[error("BRAIN below eco-neutral reserve for current eco band")]
    BrainBelowEcoNeutral,
    #[error("SystemAdjustment vetoed by sustained PainCorridor in relevant domain/region")]
    PainCorridorVeto,
    #[error("MORPH/EVOLVE corridor exceeded")]
    MorphExceedsEvolve,
    #[error("MORPH risk band violation without new evidence")]
    MorphRiskBandViolation,
}

// --- helpers ---------------------------------------------------------------

fn compute_fatigue_from_lifeforce(
    series: &LifeforceBandSeries,
) -> (f64, LifeforceBand) {
    series.compute_fatigue_and_band()
}

fn eco_neutral_brain_required(profile: &EcoBandProfile, state_brain: f64) -> f64 {
    profile.econeutral_brain_required(state_brain)
}

/// Determine if this adjustment touches a pain‑relevant somatic domain.
fn is_pain_relevant_domain(domain: &SystemDomain) -> bool {
    matches!(
        domain,
        SystemDomain::DetoxMicro
            | SystemDomain::Radiology
            | SystemDomain::NanoRepairMicro
            | SystemDomain::TeethClawMicro
    )
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

// --- canonical guarded mutation gate --------------------------------------

/// Canonical mutation gate: lifeforce + eco + WAVE + PainCorridor + EVOLVE/MORPH.
///
/// This is the **only** place BioTokenState balances change.
#[allow(clippy::too_many_arguments)]
pub fn apply_lifeforce_guarded_adjustment(
    state: &mut BioTokenState,
    env: &HostEnvelope,
    adj: &SystemAdjustment,
    lifeforce_series: &LifeforceBandSeries,
    eco_profile: &EcoBandProfile,
    wave_curve: &SafetyCurveWave,
    pain_signal: Option<&PainCorridorSignal>,
    morph_risk: &MorphRiskBands,
    has_new_evidence: bool,
) -> Result<(), LifeforceError> {
    // 1. Lifeforce band + fatigue.
    let (fatigue, band) = compute_fatigue_from_lifeforce(lifeforce_series);

    if matches!(band, LifeforceBand::HardStop) {
        return Err(LifeforceError::LifeforceHardStop);
    }

    // 2. Subjective veto: sustained PainCorridor blocks somatic domains.
    if let Some(pain) = pain_signal {
        if pain.is_sustained_hardstop() && is_pain_relevant_domain(&adj.domain) {
            return Err(LifeforceError::PainCorridorVeto);
        }
    }

    // 3. Projected biophysical tokens.
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

    // 4. Eco‑neutral BRAIN reserve.
    let eco_required = eco_neutral_brain_required(eco_profile, new_brain);
    if new_brain < eco_required {
        return Err(LifeforceError::BrainBelowEcoNeutral);
    }

    // 5. WAVE ceiling from BRAIN + fatigue.
    let safe_wave_ceiling = wave_curve.safe_wave_ceiling(new_brain, fatigue);
    if new_wave > safe_wave_ceiling {
        return Err(LifeforceError::WaveOverSafeCeiling);
    }

    // 6. SMART bounds.
    if new_smart > env.smart_max || new_smart > new_brain {
        return Err(LifeforceError::SmartOverMax);
    }

    // 7. NANO envelope (fraction).
    if new_nano > env.nano_max_fraction {
        return Err(LifeforceError::NanoOverEnvelope);
    }

    // 8. Eco envelope.
    if adj.eco_cost > env.eco_flops_limit {
        return Err(LifeforceError::EcoOverLimit(adj.eco_cost));
    }

    // 9. EVOLVE / MORPH invariants.
    let evolve_after = EvolveBudget {
        evolve_total: state.evolve.evolve_total,
        evolve_used:  (state.evolve.evolve_used + adj.delta_evolve).max(0.0),
    };
    if evolve_after.evolve_used > evolve_after.evolve_total {
        return Err(LifeforceError::MorphExceedsEvolve);
    }

    let morph_after = check_morph_vs_evolve(&evolve_after, &state.morph, &adj.delta_morph)?;
    check_morph_risk_monotone(&state.morph, &morph_after, morph_risk, has_new_evidence)?;

    // 10. All checks passed: commit.
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
