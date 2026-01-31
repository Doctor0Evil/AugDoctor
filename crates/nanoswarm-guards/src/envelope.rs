use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoswarmEnvelope {
    pub max_local_density: f64,
    pub max_kinetic_energy: f64,
    pub clearance_half_life_max: f64,
    pub toxicity_index_max: f64,
    pub n_risk_scalar: f64,
}

impl NanoswarmEnvelope {
    pub fn from_evidence(e: &EvidenceBundle) -> Self {
        let perfusion = e.get(EvidenceTagId::PerfusionIndex).unwrap_or(1.0);
        let thermal = e.get(EvidenceTagId::ThermalMargin).unwrap_or(1.0);
        let inflammation = e.get(EvidenceTagId::InflammationIndex).unwrap_or(0.5);

        let max_local_density = 1e6 * perfusion;
        let max_kinetic_energy = 1e-9 * thermal;
        let clearance_half_life_max = 60.0 / perfusion.max(0.1);
        let toxicity_index_max = (1.0 - inflammation).max(0.1);

        let rho = 0.5;
        let t_local = 0.2;
        let il6 = inflammation;
        let rho_max_e = 1.0;
        let t_max_e = 1.0;
        let il6_max = 1.0;
        let w_n = 1.0;
        let n_risk_scalar = w_n
            * (rho / rho_max_e + t_local / t_max_e + il6 / il6_max);

        Self {
            max_local_density,
            max_kinetic_energy,
            clearance_half_life_max,
            toxicity_index_max,
            n_risk_scalar,
        }
    }
}
