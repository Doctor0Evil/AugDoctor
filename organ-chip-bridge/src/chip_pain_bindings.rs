use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use neural_roping::pain_corridor::{PainBand, PainCorridorSignal, SomaticRegionId};

/// Minimal telemetry from organ-on-a-chip rigs used as nociception proxies.
/// No identity, no consciousness; just biophysical markers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChipPainTelemetry {
    pub chip_id: String,
    pub ts_utc: DateTime<Utc>,
    pub tissue_type: String,
    pub teer_ohm_cm2: f32,
    pub dissolved_o2_percent: f32,
    pub cytokine_il6_pg_ml: f32,
    pub cytokine_tnf_pg_ml: f32,
}

impl ChipPainTelemetry {
    /// Deterministic mapping from inflammation+barrier disruption → PainBand,
    /// used for in-vitro calibration of the in-vivo PainCorridor thresholds.
    pub fn to_pain_band(&self) -> PainBand {
        let inflam = (self.cytokine_il6_pg_ml + self.cytokine_tnf_pg_ml) / 2.0;
        let barrier_drop = if self.teer_ohm_cm2 < 300.0 { 1.0 } else { 0.0 };

        let score = 0.7 * (inflam / 100.0) + 0.3 * barrier_drop;

        if score < 0.15 {
            PainBand::None
        } else if score < 0.3 {
            PainBand::Mild
        } else if score < 0.5 {
            PainBand::Moderate
        } else if score < 0.8 {
            PainBand::Severe
        } else {
            PainBand::HardStop
        }
    }

    pub fn to_pain_corridor_signal(&self) -> PainCorridorSignal {
        let band = self.to_pain_band();
        let conf = 0.8; // high, but not unity – in-vitro proxy.

        PainCorridorSignal {
            host_id: format!("chip-host-{}", self.chip_id),
            ts_utc: self.ts_utc,
            region_id: SomaticRegionId::Custom(self.tissue_type.clone()),
            band,
            sustained_seconds: 10,
            confidence: conf,
        }
    }
}
