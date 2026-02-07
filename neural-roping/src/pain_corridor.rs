use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Somatic region classification; must line up with NanoSwarmBioBoundaryMap.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SomaticRegionId {
    Head,
    CervicalSpine,
    ThoracicSpine,
    LumbarSpine,
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
    Abdomen,
    Thorax,
    Pelvis,
    Custom(String),
}

/// Pain band collapses multimodal intensity into discrete, machine-checkable states.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PainBand {
    None,
    Mild,
    Moderate,
    Severe,
    HardStop, // treated as veto-equivalent to LifeforceBand::HardStop for somatic ops
}

/// Stable typing across BCI / router / lifeforce-guards.
/// This is the only thing NanoLifebandRouter and lifeforce guards need to know.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PainCorridorSignal {
    pub host_id: String,
    pub ts_utc: DateTime<Utc>,

    /// Region we believe the nociceptive focus is in.
    pub region_id: SomaticRegionId,

    /// Current band, derived from EEG/PPG/EMG models + subjective label alignment.
    pub band: PainBand,

    /// Duration in seconds that pain has remained at least Moderate in this region.
    /// Used to distinguish momentary spikes vs sustained aversive stimulation.
    pub sustained_seconds: u32,

    /// Confidence that this is true nociceptive/aversive content, 0.0–1.0.
    pub confidence: f32,
}

impl PainCorridorSignal {
    /// Conservative criterion for "sustained veto".
    /// Tunable later, but hard-coded here for lab safety.
    pub fn is_sustained_hardstop(&self) -> bool {
        // minimum thresholds can be adjusted via governance shards later,
        // but not made *less* strict without code change.
        let band_gate = matches!(self.band, PainBand::Severe | PainBand::HardStop);
        let time_gate = self.sustained_seconds >= 3; // ≥3 s sustained
        let conf_gate = self.confidence >= 0.85;

        band_gate && time_gate && conf_gate
    }
}
