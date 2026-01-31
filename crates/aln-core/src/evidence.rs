use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EvidenceTagId {
    Cmro2,
    InflammationIndex,
    PerfusionIndex,
    ThermalMargin,
    NeuromorphicEnergyIndex,
    Hrv,
    SleepDebt,
    FatigueIndex,
    StressIndex,
    EcoImpactScoreBaseline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTag {
    pub id: EvidenceTagId,
    pub name: String,
    pub value: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub tags: Vec<EvidenceTag>,
}

impl EvidenceBundle {
    pub fn get(&self, id: EvidenceTagId) -> Option<f64> {
        self.tags.iter().find(|t| t.id == id).map(|t| t.value)
    }

    pub fn ensure_within_bounds(&self) -> bool {
        self.tags
            .iter()
            .all(|t| t.value >= t.lower_bound && t.value <= t.upper_bound)
    }
}
