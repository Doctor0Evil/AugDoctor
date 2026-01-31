use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuromorphicEnvelope {
    pub max_power_density: f64,
    pub max_spike_rate: f64,
    pub max_memory_events: f64,
    pub local_decode_only: bool,
    pub neuromorphic_energy_per_inference_max: f64,
}

impl NeuromorphicEnvelope {
    pub fn from_evidence(e: &EvidenceBundle) -> Self {
        let energy_idx = e
            .get(EvidenceTagId::NeuromorphicEnergyIndex)
            .unwrap_or(1.0);
        let thermal = e.get(EvidenceTagId::ThermalMargin).unwrap_or(1.0);

        let max_power_density = 1.0 * thermal;
        let max_spike_rate = 1000.0;
        let max_memory_events = 1e6;
        let local_decode_only = true;
        let neuromorphic_energy_per_inference_max = 1.0 / energy_idx.max(0.1);

        Self {
            max_power_density,
            max_spike_rate,
            max_memory_events,
            local_decode_only,
            neuromorphic_energy_per_inference_max,
        }
    }

    pub fn energy_per_inference(p: f64, t_inf: f64, n_tokens: f64) -> f64 {
        p * t_inf / n_tokens.max(1.0)
    }
}
