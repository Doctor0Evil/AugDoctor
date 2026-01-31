use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleSafetyEnvelopeV2 {
    pub intent_throughput_max: f64,
    pub fatigue_index_max: f64,
}

impl MuscleSafetyEnvelopeV2 {
    pub fn from_evidence(e: &EvidenceBundle) -> Self {
        let fatigue = e.get(EvidenceTagId::FatigueIndex).unwrap_or(0.5);
        let intent_throughput_max = (5.0 * (1.0 - fatigue)).max(1.0);
        let fatigue_index_max = fatigue;
        Self {
            intent_throughput_max,
            fatigue_index_max,
        }
    }
}
