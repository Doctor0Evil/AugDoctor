use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::sealed::inner::Sealed;

/// Discrete, machine-checkable lifeforce bands for host safety.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LifeforceBand {
    Safe,
    SoftWarn,
    HardStop,
}

/// Point-in-time lifeforce sample, normalized to [0.0, 1.0] where applicable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceSample {
    pub ts_utc: DateTime<Utc>,
    /// Scalar lifeforce index in [0.0, 1.0].
    pub lifeforce_l: f32,
    pub band: LifeforceBand,
    /// Normalized BLOOD proxy in [0.0, 1.0].
    pub blood_level: f32,
    /// Normalized OXYGEN proxy in [0.0, 1.0].
    pub oxygen_level: f32,
    /// Normalized cognitive clarity in [0.0, 1.0].
    pub clarity_index: f32,
}

/// Per-host lifeforce history.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceBandSeries {
    pub host_id: String,
    pub samples: Vec<LifeforceSample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioTokenState {
    pub brain:  f64,
    pub wave:   f64,
    pub blood:  f64,
    pub oxygen: f64,
    pub nano:   f64,
    pub smart:  f64,

    pub evolve: EvolveBudget,   // EVOLVE scalar
    pub morph:  MorphBudget,    // MORPH vector
}

// Seal core types so mutation traits cannot be implemented on foreign types.
impl Sealed for BioTokenState {}

/// Host-level configuration and lifeforce/eco limits.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostEnvelope {
    pub hostid: String,
    pub brainmin: f64,
    pub bloodmin: f64,
    pub oxygenmin: f64,
    pub nanomaxfraction: f64,
    pub smartmax: f64,
    pub ecoflopslimit: f64,
}

impl Sealed for HostEnvelope {}

/// System-only adjustment to the inner ledger (no transfer semantics).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemAdjustment {
    pub deltabrain:  f64,
    pub deltawave:   f64,
    pub deltablood:  f64,
    pub deltaoxygen: f64,
    pub deltanano:   f64,
    pub deltasmart:  f64,
    /// Eco-cost in FLOPs, nJ, etc. for accounting.
    pub ecocost:     f64,
    /// Human-readable reason label (e.g. "quantum-learning-step").
    pub reason:      String,
}

impl Sealed for SystemAdjustment {}

/// Per-host WAVE safety curve as a function of BRAIN and fatigue.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SafetyCurveWave {
    pub host_id: String,
    /// Maximum WAVE as a fraction of current BRAIN at zero fatigue.
    pub max_wave_factor: f32,
    /// Additional decay applied as fatigue approaches 1.0.
    pub fatigue_decay: f32,
}

impl SafetyCurveWave {
    /// Compute a safe WAVE ceiling given current BRAIN and fatigue in [0.0, 1.0].
    pub fn safe_wave_ceiling(&self, brain: f64, fatigue: f64) -> f64 {
        let fatigue_clamped = fatigue.clamp(0.0, 1.0);
        let decay = (1.0_f64 - fatigue_clamped * self.fatigue_decay as f64)
            .max(0.0);
        let factor = decay * self.max_wave_factor as f64;
        (brain * factor).max(0.0)
    }
}

/// Discrete ecological impact bands.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EcoBand {
    Low,
    Medium,
    High,
}

/// Per-host eco-cost profile and derived eco band.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcoBandProfile {
    pub host_id: String,
    pub avg_flops: f64,
    pub avg_nj: f64,
    pub eco_band: EcoBand,
}

impl EcoBandProfile {
    /// Minimum BRAIN required to sustain this eco-profile without overspending.
    pub fn econeutral_brain_required(&self, state_brain: f64) -> f64 {
        let multiplier = match self.eco_band {
            EcoBand::Low => 0.10,
            EcoBand::Medium => 0.20,
            EcoBand::High => 0.30,
        };
        (state_brain * multiplier).max(0.0)
    }
}
