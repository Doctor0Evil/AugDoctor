use serde::{Deserialize, Serialize};
use biophysical_blockchain::{BioTokenState, HostEnvelope};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiophysicalAura {
    pub kindness: f64,       // 0.0–1.0
    pub non_violence: f64,   // 0.0–1.0
    pub rescue: f64,         // 0.0–1.0
    pub eco_care: f64,       // 0.0–1.0
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EvolutionDomain {
    NeuromorphReflex,
    NeuromorphSense,
    NeuromorphAttention,
    DefensiveMorphology,   // e.g. “teeth/claws as protection”
    General,
}

pub fn safe_decay_multiplier(
    state: &BioTokenState,
    env: &HostEnvelope,
    aura: &BiophysicalAura,
    domain: EvolutionDomain,
) -> f64 {
    // Base: tighter decay if lifeforce is low.
    let lifeforce_margin = (state.blood - env.blood_min)
        .min(state.oxygen - env.oxygen_min)
        .max(0.0);
    let base = if lifeforce_margin <= 0.0 {
        0.0
    } else if lifeforce_margin < 0.1 {
        0.2
    } else if lifeforce_margin < 0.2 {
        0.5
    } else {
        0.9
    };

    // Karma bonus only for clearly protective, biosafe domains.
    let karma_bonus = match domain {
        EvolutionDomain::DefensiveMorphology
        | EvolutionDomain::NeuromorphReflex
        | EvolutionDomain::NeuromorphSense => {
            let avg = (aura.kindness + aura.non_violence + aura.rescue + aura.eco_care) / 4.0;
            0.1 * avg  // max +0.1
        }
        _ => 0.0,
    };

    // Never exceed 1.0, never below 0.0.
    let m = (base + karma_bonus).max(0.0).min(1.0);
    m
}
