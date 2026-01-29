use crate::domain::{NeuromorphDomain};
use biophysical_blockchain::{BioTokenState};
use bioscale_upgradeservice::lifeforce::LifeforceBandSeries; // conceptual
use crate::observation::NeuromorphEvent;

#[derive(Clone, Debug)]
pub struct NeuromorphContext {
    pub domain: NeuromorphDomain,
    pub lifeforce_bands: LifeforceBandSeries,
    pub pain_index: f32,          // aggregated from comfort/pain corridors
    pub decay_multiplier: f32,    // DECAY for this pattern
    pub host_consent_shard: bool, // neuromorph-auto-micro present?
}

#[derive(Clone, Debug)]
pub struct ReflexSenseEligibility {
    pub allowed: bool,
    pub decay_weight: f32,
    pub auto_allowed: bool,
}

pub fn check_neuromorph_eligibility(
    ctx: &NeuromorphContext,
    state: &BioTokenState,
    event: &NeuromorphEvent,
) -> ReflexSenseEligibility {
    // 1. Domain tagging: only three neuromorph micro domains are evolution-eligible.
    let evolution_tagged = matches!(
        ctx.domain,
        NeuromorphDomain::ReflexSafetyMicro
            | NeuromorphDomain::SensoryClarityMicro
            | NeuromorphDomain::AttentionBalanceMicro
    );

    if !evolution_tagged {
        return ReflexSenseEligibility { allowed: false, decay_weight: 0.0, auto_allowed: false };
    }

    // 2. Biocompatibility: reject if pain corridor or DECAY indicate strain.
    if ctx.pain_index > 0.0 || ctx.decay_multiplier < 1.0 {
        return ReflexSenseEligibility { allowed: false, decay_weight: 0.0, auto_allowed: false };
    }

    // Additional constraint: lifeforce must be comfortably in safe bands.
    if !ctx.lifeforce_bands.is_strongly_safe() {
        return ReflexSenseEligibility { allowed: false, decay_weight: 0.0, auto_allowed: false };
    }

    // 3. Explicit consent shard: must be present to allow auto-evolution.
    if !ctx.host_consent_shard {
        return ReflexSenseEligibility { allowed: true, decay_weight: ctx.decay_multiplier, auto_allowed: false };
    }

    ReflexSenseEligibility { allowed: true, decay_weight: ctx.decay_multiplier, auto_allowed: true }
}
