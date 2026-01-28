//! Biophysical Blockchain Module Extension v1.4
//! Adds SCALE, SANITY, DECAY, and RADS tokens under BRAIN/NANO/WAVE governance.

use crate::core::{BrainState, NanoState, WaveLevel};
use crate::safety::{RadiationMonitor, BioLoad, ConsciousnessLock};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ScaleToken {
    pub level: f32,           // 0.0 – 1.0: normalized mutation allowance per cycle
    pub range: (f32, f32),    // min/max upgrade bounds based on host performance
    pub scope: f32,           // upgrade complexity and depth modifier
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SanityToken {
    pub balance: f32,         // host-specific psychological margin
    pub decay_rate: f32,      // time indoors or isolation-based reduction rate
    pub stability: f32,       // recovered via social/civic interactions
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DecayToken {
    pub region_index: u32,    // spatial location or active region identity
    pub loss: f32,            // accumulated decay from inactivity
    pub threshold: f32,       // max resilience, derived from BRAIN state
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RadsToken {
    pub exposure: f32,        // radiation dose (reversible, measured)
    pub safe_limit: f32,      // maximum permitted per host
    pub wave_multiplier: f32, // WAVE-linked adaptive safety guard
}

/// Host-biophysical rule set implementing token governance logic.
pub struct HostGovernance;

impl HostGovernance {
    pub fn compute_scale(brain: &BrainState, nano: &NanoState) -> ScaleToken {
        let cap = (brain.cognitive_index * nano.synaptic_density).min(1.0);
        ScaleToken {
            level: cap,
            range: (0.1, cap),
            scope: brain.evo_bandwidth * 0.85,
        }
    }

    pub fn check_sanity(balance: f32, decay: &DecayToken) -> SanityToken {
        let recovery = (1.0 - decay.loss).max(0.0);
        SanityToken {
            balance: balance * recovery,
            decay_rate: decay.loss * 0.1,
            stability: recovery,
        }
    }

    pub fn manage_rads(rads: f32, wave: &WaveLevel, brain: &BrainState) -> RadsToken {
        let adaptive_limit = brain.bioload_safety * (wave.amplitude * 0.9);
        RadsToken {
            exposure: rads.min(adaptive_limit),
            safe_limit: adaptive_limit,
            wave_multiplier: wave.frequency * 0.02,
        }
    }
}
