use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveLoadEnvelopeV2 {
    pub theta_alpha_ratio_max: f64,
    pub error_rate_max: f64,
    pub microbreak_interval_min: f64,
}

impl CognitiveLoadEnvelopeV2 {
    pub fn from_evidence(e: &EvidenceBundle) -> Self {
        let cmro2 = e.get(EvidenceTagId::Cmro2).unwrap_or(1.0);
        let hrv = e.get(EvidenceTagId::Hrv).unwrap_or(1.0);
        let fatigue = e.get(EvidenceTagId::FatigueIndex).unwrap_or(0.5);
        let stress = e.get(EvidenceTagId::StressIndex).unwrap_or(0.5);

        let theta_alpha_ratio_max = 2.0 * cmro2;
        let error_rate_max = 0.1 + 0.2 * (1.0 - hrv).clamp(0.0, 1.0);
        let g0 = 30.0;
        let g1 = 10.0;
        let fi = 0.5 * fatigue + 0.5 * stress;
        let microbreak_interval_min = (g0 - g1 * fi).max(5.0);

        Self {
            theta_alpha_ratio_max,
            error_rate_max,
            microbreak_interval_min,
        }
    }
}
