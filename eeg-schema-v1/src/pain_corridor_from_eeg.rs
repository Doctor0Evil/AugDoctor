use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::EegFeatureSummaryV1;
use neural_roping::pain_corridor::{PainBand, PainCorridorSignal, SomaticRegionId};

/// Lightweight accumulator for per-region pain state across frames.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegionPainAccumulator {
    pub last_band: PainBand,
    pub sustained_seconds: u32,
}

impl RegionPainAccumulator {
    pub fn update(&mut self, new_band: PainBand, frame_dt_sec: u32) {
        if new_band as u8 >= self.last_band as u8 {
            // band is same or higher → accumulate
            self.sustained_seconds = self.sustained_seconds.saturating_add(frame_dt_sec);
        } else {
            // pain decreased → reset sustained counter
            self.sustained_seconds = 0;
        }
        self.last_band = new_band;
    }
}

/// Very conservative mapping from EEG features → PainBand.
/// In real deployments you’ll plug in a trained model; here we wire a deterministic policy
/// suitable for in‑silico/in‑vitro testing.
fn classify_pain_band_from_eeg(eeg: &EegFeatureSummaryV1) -> PainBand {
    // Example: use nociceptive gamma/alpha ratios + frontal theta + EMG surrogate.
    let nociceptive_score = eeg
        .metrics
        .get("nociceptive_gamma_alpha_ratio")
        .copied()
        .unwrap_or(0.0);

    let defensive_theta = eeg
        .metrics
        .get("frontal_theta_defensive")
        .copied()
        .unwrap_or(0.0);

    let emg_like = eeg
        .metrics
        .get("emg_surrogate")
        .copied()
        .unwrap_or(0.0);

    let composite = 0.5 * nociceptive_score + 0.3 * defensive_theta + 0.2 * emg_like;

    if composite < 0.15 {
        PainBand::None
    } else if composite < 0.35 {
        PainBand::Mild
    } else if composite < 0.55 {
        PainBand::Moderate
    } else if composite < 0.75 {
        PainBand::Severe
    } else {
        PainBand::HardStop
    }
}

/// Map EEG topography tags to a coarse somatic region.
/// This stays deliberately low-resolution to avoid any “soul/identity” encoding.
fn map_topography_to_region(topography: &str) -> SomaticRegionId {
    match topography {
        "vertex" | "central" | "frontal" => SomaticRegionId::Head,
        "parietal" | "occipital" => SomaticRegionId::Head,
        "left_motor" => SomaticRegionId::LeftArm,
        "right_motor" => SomaticRegionId::RightArm,
        "spinal_cervical" => SomaticRegionId::CervicalSpine,
        "spinal_thoracic" => SomaticRegionId::ThoracicSpine,
        "spinal_lumbar" => SomaticRegionId::LumbarSpine,
        "abdomen" => SomaticRegionId::Abdomen,
        "thorax" => SomaticRegionId::Thorax,
        "pelvis" => SomaticRegionId::Pelvis,
        _ => SomaticRegionId::Custom(topography.to_string()),
    }
}

/// Host-local state for deriving a PainCorridorSignal stream from EEG.
/// This lives in BCI boundary service, never in BioTokenState.
#[derive(Clone, Debug, Default)]
pub struct PainCorridorDeriver {
    pub per_region: std::collections::HashMap<SomaticRegionId, RegionPainAccumulator>,
    pub frame_dt_sec: u32,
}

impl PainCorridorDeriver {
    pub fn new(frame_dt_sec: u32) -> Self {
        Self {
            per_region: std::collections::HashMap::new(),
            frame_dt_sec,
        }
    }

    pub fn step(
        &mut self,
        host_id: &str,
        ts_utc: DateTime<Utc>,
        eeg: &EegFeatureSummaryV1,
    ) -> PainCorridorSignal {
        let band = classify_pain_band_from_eeg(eeg);
        let topo = eeg.topography.as_deref().unwrap_or("vertex");
        let region = map_topography_to_region(topo);

        let entry = self
            .per_region
            .entry(region.clone())
            .or_insert_with(RegionPainAccumulator::default);

        entry.update(band.clone(), self.frame_dt_sec);

        // Confidence increases with composite nociceptive score + band ordinal.
        let base_conf = eeg
            .metrics
            .get("nociceptive_gamma_alpha_ratio")
            .copied()
            .unwrap_or(0.0);
        let band_weight = match band {
            PainBand::None => 0.0,
            PainBand::Mild => 0.2,
            PainBand::Moderate => 0.5,
            PainBand::Severe => 0.7,
            PainBand::HardStop => 0.9,
        };

        let confidence = (0.5 * base_conf + 0.5 * band_weight)
            .clamp(0.0, 1.0);

        PainCorridorSignal {
            host_id: host_id.to_string(),
            ts_utc,
            region_id: region,
            band,
            sustained_seconds: entry.sustained_seconds,
            confidence,
        }
    }
}
