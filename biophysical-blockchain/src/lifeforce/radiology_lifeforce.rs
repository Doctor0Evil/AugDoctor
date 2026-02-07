//! Radiology-aware LifeforceState extension and detox_micro actuation.
//!
//! - Treats radiological dose as a biosafety-critical resource, alongside BLOOD/OXYGEN/NANO.
//! - Adds tissue-specific radiosensitivity and recovery modelling to LifeforceState.
//! - Exposes a radiology_penalty_factor that shrinks WAVE/NANO budgets as dose accumulates.
//! - Integrates with NanoSwarmBioBoundaryMap + NanoLifebandRouter for detox_micro micro-steps.
//! - Purely non-financial, per-host, host-local; no ownership, no transfer, no staking.
//!
//! Radiobiology grounding:
//! - Tissues differ in radiosensitivity (hematologic, lens, gonads > muscle/bone). [web:33]
//! - ICRP eye-lens threshold ~0.35–0.5 Gy has driven stricter lens dose limits. [web:32][web:35]
//! - Proton PG/Compton imaging can provide mm-scale real-time range verification for dose
//!   deposition, enabling closed-loop dosimetry for nanoswarm guidance. [web:29][web:31][web:34]

use serde::{Deserialize, Serialize};

use crate::lifeforce::LifeforceBand;
use crate::types::BioTokenState;

// ---------------- Radiology types ------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadiosensitivityClass {
    VeryHigh, // e.g., lens, gonads, marrow, some CNS contexts. [web:33]
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RadiologyBand {
    Safe,
    SoftWarn,
    HardStop,
}

/// Per-region radiology limits, to be embedded or mirrored in NanoSwarmBioBoundaryMap. [file:6]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionRadiologyPolicy {
    pub region_id: String,
    pub radiosensitivity: RadiosensitivityClass,
    pub max_radiation_dose_session_mgy: f64,
    pub max_radiation_dose_daily_mgy: f64,
    /// Effective biological half-life for repair of sub-lethal damage, hours. [file:6]
    pub dose_recovery_half_life_hr: f64,
}

/// Extension of LifeforceState with radiology tracking. [file:6][file:10]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RadiologyState {
    /// Map of region_id -> cumulative dose in mGy (already decay-adjusted at last update).
    pub cumulative_dose_mgy: std::collections::HashMap<String, f64>,
    /// Last update wall-clock (ms since epoch) for dose recovery computation.
    pub last_update_ms_utc: i64,
}

impl RadiologyState {
    pub fn new(now_ms_utc: i64) -> Self {
        Self {
            cumulative_dose_mgy: std::collections::HashMap::new(),
            last_update_ms_utc: now_ms_utc,
        }
    }

    /// Exponential recovery toward 0 using region-specific half-life. [file:6]
    pub fn apply_recovery(
        &mut self,
        now_ms_utc: i64,
        region_policies: &std::collections::HashMap<String, RegionRadiologyPolicy>,
    ) {
        let dt_ms = (now_ms_utc - self.last_update_ms_utc).max(0) as f64;
        let dt_hr = dt_ms / 3_600_000.0;
        if dt_hr <= 0.0 {
            return;
        }

        for (region_id, dose) in self.cumulative_dose_mgy.iter_mut() {
            if let Some(policy) = region_policies.get(region_id) {
                let hl = policy.dose_recovery_half_life_hr.max(0.1);
                // Simple exponential: dose(t) = dose0 * 0.5^(dt / hl). [file:6]
                let factor = 0.5f64.powf(dt_hr / hl);
                *dose *= factor;
            }
        }

        self.last_update_ms_utc = now_ms_utc;
    }

    /// Add new delivered dose (e.g., from planned nanobeam or measured PG imaging). [web:29][web:31]
    pub fn accumulate_dose(&mut self, region_id: &str, delta_mgy: f64) {
        let e = self.cumulative_dose_mgy.entry(region_id.to_string()).or_insert(0.0);
        *e = (*e + delta_mgy.max(0.0)).max(0.0);
    }

    /// Compute the radiology band for a given region under its policy. [file:6]
    pub fn band_for_region(
        &self,
        region_id: &str,
        policy: &RegionRadiologyPolicy,
    ) -> RadiologyBand {
        let dose = self.cumulative_dose_mgy.get(region_id).cloned().unwrap_or(0.0);
        let frac_daily = dose / policy.max_radiation_dose_daily_mgy.max(1.0);
        let frac_sess = dose / policy.max_radiation_dose_session_mgy.max(1.0);

        let frac = frac_daily.max(frac_sess);

        if frac >= 1.0 {
            RadiologyBand::HardStop
        } else if frac >= 0.5 {
            RadiologyBand::SoftWarn
        } else {
            RadiologyBand::Safe
        }
    }
}

/// Radiology penalty that shrinks WAVE/NANO budgets as dose accumulates. [file:6]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RadiologyPenalty {
    /// Scalar 0.0–1.0 applied to effective WAVE budget.
    pub wave_factor: f64,
    /// Scalar 0.0–1.0 applied to effective NANO (detox_micro) budget in region.
    pub nano_factor: f64,
    /// Derived radiology band for introspection + logging.
    pub band: RadiologyBand,
}

impl RadiologyPenalty {
    /// Compute penalty from dose fraction and radiosensitivity class. [file:6][web:33]
    pub fn from_dose(
        frac_to_daily_limit: f64,
        radiosensitivity: &RadiosensitivityClass,
    ) -> Self {
        let f = frac_to_daily_limit.clamp(0.0, 2.0);
        let base = match radiosensitivity {
            RadiosensitivityClass::VeryHigh => 1.0 - 0.9 * f, // rapid throttling.
            RadiosensitivityClass::High => 1.0 - 0.7 * f,
            RadiosensitivityClass::Medium => 1.0 - 0.5 * f,
            RadiosensitivityClass::Low => 1.0 - 0.3 * f,
        };

        let band = if f >= 1.0 {
            RadiologyBand::HardStop
        } else if f >= 0.5 {
            RadiologyBand::SoftWarn
        } else {
            RadiologyBand::Safe
        };

        // Nano penalty slightly stronger than WAVE near limits for detox_micro. [file:6]
        let wave_factor = base.clamp(0.0, 1.0);
        let nano_factor = (base - 0.1 * f).clamp(0.0, 1.0);

        Self {
            wave_factor,
            nano_factor,
            band,
        }
    }
}

// ------------- Lifeforce + Radiology integration --------------------

/// Lifeforce + radiology composite, used by runtime for budget computation. [file:6][file:10]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceWithRadiology {
    pub lifeforce_band: LifeforceBand,
    pub eco_band: crate::lifeforce::EcoBand,
    pub radiology: RadiologyState,
}

impl LifeforceWithRadiology {
    /// Compute radiology_penalty_factor for a given region and policy. [file:6]
    pub fn radiology_penalty_for_region(
        &self,
        region_id: &str,
        policy: &RegionRadiologyPolicy,
    ) -> RadiologyPenalty {
        let dose = self
            .radiology
            .cumulative_dose_mgy
            .get(region_id)
            .cloned()
            .unwrap_or(0.0);
        let frac_daily = dose / policy.max_radiation_dose_daily_mgy.max(1.0);
        RadiologyPenalty::from_dose(frac_daily, &policy.radiosensitivity)
    }
}

/// Example hook: adjust effective WAVE/NANO budgets before forming SystemAdjustment. [file:10]
pub fn apply_radiology_penalty_to_budgets(
    state: &BioTokenState,
    lifeforce: &LifeforceWithRadiology,
    region_id: &str,
    policy: &RegionRadiologyPolicy,
    base_wave_budget: f64,
    base_nano_budget: f64,
) -> (RadiologyPenalty, f64, f64) {
    let penalty = lifeforce.radiology_penalty_for_region(region_id, policy);

    // HardStop: zero out radiological workloads in this region, independent of SCALE/DECAY. [file:6]
    if matches!(penalty.band, RadiologyBand::HardStop) {
        return (
            penalty,
            0.0,
            0.0, // detox_micro must not schedule in this region.
        );
    }

    // SoftWarn: aggressively shrink budgets but allow tiny micro-steps. [file:6]
    let effective_wave = (base_wave_budget * penalty.wave_factor)
        .min(state.wave)
        .max(0.0);
    let effective_nano = (base_nano_budget * penalty.nano_factor)
        .min(state.nano)
        .max(0.0);

    (penalty, effective_wave, effective_nano)
}

// ------------- detox_micro micro-actuation interface --------------------

/// Domain tag for detox_micro in routing/runtime. [file:6]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NanoDomain {
    DetoxMicro,
    RepairMicro,
    SensorHousekeeping,
    ComputeAssist,
}

/// One detox_micro micro-step description (non-token, non-financial). [file:6]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetoxMicroStep {
    pub region_id: String,
    pub requested_nano_fraction: f64,
    pub planned_blood_cost: f64,
    pub planned_oxygen_cost: f64,
    pub reason: String, // e.g., "detox-micro: remove cluster @ hepatic-lobe-2".
}

/// Result of routing + lifeforce + radiology for a detox_micro step. [file:6]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetoxMicroPlanOutcome {
    pub accepted: bool,
    pub router_decision: crate::nanoswarm::RouterDecision,
    pub radiology_band: RadiologyBand,
    pub effective_nano_fraction: f64,
}

/// Orchestration-time helper: given a proposed detox_micro step, tighten it using
/// NanoLifebandRouter and radiology_penalty_factor before forming a SystemAdjustment. [file:6]
pub fn plan_detox_micro_step<R: crate::nanoswarm::NanoLifebandRouter>(
    router: &R,
    obs_band: &crate::nanoswarm::NanoSwarmObservationBand,
    boundary_map: &crate::nanoswarm::NanoSwarmBioBoundaryMap,
    lifeforce_rad: &LifeforceWithRadiology,
    region_policy: &RegionRadiologyPolicy,
    base_wave_budget: f64,
    base_nano_budget: f64,
    step: &DetoxMicroStep,
    pain_corridor_active: bool,
) -> DetoxMicroPlanOutcome {
    // 1. Router advisory decision (includes radiology checks at hard policy level). [file:6]
    let decision_log = router.route(
        obs_band,
        boundary_map,
        NanoDomain::DetoxMicro,
        step.requested_nano_fraction,
        pain_corridor_active,
    );

    // If router denies, we stop regardless of radiology penalty. [file:6]
    if matches!(decision_log.router_decision, crate::nanoswarm::RouterDecision::Deny) {
        return DetoxMicroPlanOutcome {
            accepted: false,
            router_decision: decision_log.router_decision,
            radiology_band: decision_log.applied_radiology_band,
            effective_nano_fraction: 0.0,
        };
    }

    // 2. Apply radiology penalty as inner lifeforce pre-filter on budgets. [file:6]
    let (penalty, _wave_eff, nano_eff) = apply_radiology_penalty_to_budgets(
        &BioTokenState {
            // Narrow view: only WAVE/NANO matter here for budgeting.
            brain: 0.0,
            wave: base_wave_budget,
            blood: 0.0,
            oxygen: 0.0,
            nano: base_nano_budget,
            smart: 0.0,
            hostid: obs_band.host_id.clone(),
            lorentzts: crate::time::LorentzTimestamp(0, 0),
        },
        lifeforce_rad,
        &step.region_id,
        region_policy,
        base_wave_budget,
        base_nano_budget,
    );

    if matches!(penalty.band, RadiologyBand::HardStop) {
        return DetoxMicroPlanOutcome {
            accepted: false,
            router_decision: crate::nanoswarm::RouterDecision::Deny,
            radiology_band: penalty.band,
            effective_nano_fraction: 0.0,
        };
    }

    // 3. Effective nano fraction is bounded by both router-approved request and nano_eff. [file:6]
    let eff_fraction = step
        .requested_nano_fraction
        .min(nano_eff)
        .max(0.0);

    // SoftWarn: still "accepted" but higher-level scheduler may downsample passes. [file:6]
    DetoxMicroPlanOutcome {
        accepted: eff_fraction > 0.0,
        router_decision: decision_log.router_decision,
        radiology_band: penalty.band,
        effective_nano_fraction: eff_fraction,
    }
}
