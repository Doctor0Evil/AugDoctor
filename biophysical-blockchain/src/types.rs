use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LifeforceBand {
    Safe,
    SoftWarn,
    HardStop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceSample {
    pub ts_utc: DateTime<Utc>,
    pub lifeforce_l: f32,           // [0.0, 1.0]
    pub band: LifeforceBand,
    pub blood_level: f32,          // normalized BLOOD proxy
    pub oxygen_level: f32,         // normalized OXYGEN proxy
    pub clarity_index: f32,        // normalized cognitive clarity
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceBandSeries {
    pub host_id: String,
    pub samples: Vec<LifeforceSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafetyCurveWave {
    pub host_id: String,
    /// max WAVE as fraction of current BRAIN for given fatigue level.
    pub max_wave_factor: f32,
    pub fatigue_decay: f32,
}

impl SafetyCurveWave {
    pub fn safe_wave_ceiling(&self, brain: f64, fatigue: f64) -> f64 {
        let f = (1.0 - fatigue.max(0.0).min(1.0) * self.fatigue_decay as f64)
            * self.max_wave_factor as f64;
        (brain * f).max(0.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EcoBand {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcoBandProfile {
    pub host_id: String,
    pub avg_flops: f64,
    pub avg_nj: f64,
    pub eco_band: EcoBand,
}

impl EcoBandProfile {
    pub fn econeutral_brain_required(&self, state_brain: f64) -> f64 {
        match self.eco_band {
            EcoBand::Low => state_brain * 0.1,
            EcoBand::Medium => state_brain * 0.2,
            EcoBand::High => state_brain * 0.3,
        }
    }
}
