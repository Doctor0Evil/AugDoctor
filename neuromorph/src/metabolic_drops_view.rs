//! MetabolicDropsView: UI- and chipset-friendly view over BLOOD + LifeforceBand.
//!
//! Layer: boundary/orchestrator ONLY.
//! - Reads host summary (BLOOD scalar + LifeforceBand).
//! - Exposes a 0–100 "metabolic drops" scale for neuromorph modules.
//! - Never writes BioTokenState or alters inner-ledger invariants.
//!
//! Doctrine-aligned:
//! - BLOOD stays a continuous, non-financial safety asset with hard floors.
//! - LifeforceBand (Safe / SoftWarn / HardStop) still gates high-risk load.
//! - Drops are a *view*, not a new token or mechanic.

use serde::{Deserialize, Serialize};
use crate::biophysical_client::HostStateSummary; // boundary-level summary type
use biophysical_blockchain::types::LifeforceBand; // from inner types crate

/// Configuration for mapping BLOOD into 0–100 drops.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetabolicDropsConfig {
    /// BLOOD value treated as "0 drops" (soft floor for neuromorph use).
    pub blood_soft_floor: f64,
    /// BLOOD value treated as "100 drops" (typical healthy upper operating point).
    pub blood_soft_ceiling: f64,
    /// Minimum drops value neuromorph is allowed to draw on even in Safe band.
    pub min_safe_drops: u8,
    /// Maximum drops neuromorph may use when LifeforceBand=Safe.
    pub max_safe_drops: u8,
    /// Maximum drops when LifeforceBand=SoftWarn.
    pub max_softwarn_drops: u8,
}

impl Default for MetabolicDropsConfig {
    fn default() -> Self {
        Self {
            // These are *policy* numbers; tune per host from calibration data.
            blood_soft_floor: 0.25,   // e.g., 25% of BLOOD capacity
            blood_soft_ceiling: 0.90, // e.g., 90% of BLOOD capacity
            min_safe_drops: 5,
            max_safe_drops: 100,
            max_softwarn_drops: 40,
        }
    }
}

/// View type supplied to neuromorph modules.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetabolicDropsView {
    /// Drops in [0, 100], after band gating.
    pub drops_available: u8,
    /// Original BLOOD scalar from summary (for diagnostics/telemetry only).
    pub blood_raw: f64,
    /// Current LifeforceBand (Safe / SoftWarn / HardStop).
    pub lifeforce_band: LifeforceBand,
}

impl MetabolicDropsView {
    /// Compute a drops view from host summary and config.
    ///
    /// Rules:
    /// - Map BLOOD linearly between blood_soft_floor -> 0 and blood_soft_ceiling -> 100.
    /// - Clamp to [0, 100].
    /// - Then tighten by LifeforceBand:
    ///   - HardStop  -> 0 drops.
    ///   - SoftWarn  -> min(raw, max_softwarn_drops).
    ///   - Safe      -> clamp between min_safe_drops and max_safe_drops.
    pub fn from_summary(summary: &HostStateSummary, cfg: &MetabolicDropsConfig) -> Self {
        let blood = summary.blood_level; // e.g., normalized [0.0, 1.0]

        // 1. Linear mapping BLOOD -> raw 0–100 drops.
        let raw_drops = if blood <= cfg.blood_soft_floor {
            0.0
        } else if blood >= cfg.blood_soft_ceiling {
            100.0
        } else {
            let span = cfg.blood_soft_ceiling - cfg.blood_soft_floor;
            let rel   = (blood - cfg.blood_soft_floor) / span;
            rel * 100.0
        };

        let mut drops = raw_drops.clamp(0.0, 100.0) as u8;
        let band = summary.lifeforce_band.clone();

        // 2. LifeforceBand gating.
        drops = match band {
            LifeforceBand::HardStop => 0,
            LifeforceBand::SoftWarn => {
                drops.min(cfg.max_softwarn_drops)
            }
            LifeforceBand::Safe => {
                drops
                    .max(cfg.min_safe_drops)
                    .min(cfg.max_safe_drops)
            }
        };

        MetabolicDropsView {
            drops_available: drops,
            blood_raw: blood,
            lifeforce_band: band,
        }
    }
}
