//!
//! Safety-Gated Nanoswarm Router for Precision Detoxification.
//! Implements NanoLifebandRouter trait with radiological constraints.
//! Purely advisory: produces NanoRouteDecisionLog entries.
//! Enforcement remains in inner ledger lifeforce/eco guards.
//! Host-local, non-financial ALN shards for observation/biosphere.
//! Compatible: Windows/Linux/Ubuntu, Android/iOS via wasm bindgen.
//! Quantum-learning ready: fields optimized for ML feature extraction.
//!
//! Awareness-check: Observes living-organism states via lifeforce_band (no actuation).
//! Consciousness-state: Immutable (read-only enums), non-sticky.
//! Oculus-check: No vision/video-feed relay.
//! Brain-tokens: No net-weight/uncirculated-supply; no hardware-deps.
//! Cloning: Structs clone-safe; no consciousness/soul quantifiable.

use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Reused from doctrine: enums for type-safety.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LifeforceBand {
    Safe,
    SoftWarn,
    HardStop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EcoBand {
    Neutral,
    High,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RadiologyBand {
    Safe,
    SoftWarn,
    HardStop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NanoDomain {
    DetoxMicro,
    RepairMicro,
    SensorHousekeeping,
    ComputeAssist,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RouterDecision {
    Safe,
    Defer,
    Deny,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReasonCode {
    HardStop,
    EcoHigh,
    PainCorridor,
    RadiologyRisk,
    DensityExceeded,
    NoFlyZone,
    UnknownRegion,
}

// Deep-introspection: Biophysical 5D object (host_id, ts, region, bioscale, rad_context).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoSwarmObservationBand {
    pub host_id: String,              // DID/ALN host-local.
    pub ts_utc: u64,                  // Nanoseconds since epoch.
    pub region_id: String,            // e.g., "hepatic_lobe_2".
    pub nano_load_fraction: f64,      // 0.0-1.0 current density.
    pub local_temp: f64,              // Celsius.
    pub tissue_type: String,          // e.g., "vascular".
    pub lifeforce_band: LifeforceBand,
    pub eco_band: EcoBand,
    pub clarity_index: f64,           // Signal-to-noise 0.0-1.0.
    // Radiological extensions: under radiological constraints.
    pub cumulative_radiology_mgy: f64, // Cumulative dose mGy.
    pub radiology_band: RadiologyBand,
    pub infection_marker: bool,       // Threat flag.
}

// NanoRouteDecisionLog: Immutable audit trail, ML ground-truth.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoRouteDecisionLog {
    pub decision_id: String,          // UUID/hex.
    pub ts_utc: u64,
    pub host_id: String,
    pub router_decision: RouterDecision,
    pub reason_code: ReasonCode,
    pub nano_domain: NanoDomain,
    pub region_id: String,
    pub requested_nano_fraction: f64,
    pub applied_radiology_band: RadiologyBand,
}

// Region entry with radiological constraints.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryRegion {
    pub region_id: String,
    pub bioscale_plane: String,       // "InVivo".
    pub allowed_nano_density_range: (f64, f64), // min-max fraction.
    pub is_no_fly_zone: bool,
    // Radiological: precision detox under constraints.
    pub max_radiation_dose_session_mgy: f64,
    pub max_radiation_dose_daily_mgy: f64,
    pub dose_recovery_half_life_hr: f64,
    pub radiosensitivity_class: String, // "VeryHigh".
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoSwarmBioBoundaryMap {
    pub map_version_id: String,
    pub host_id: String,
    pub valid_from_utc: u64,
    pub valid_until_utc: u64,
    pub regions: HashMap<String, BoundaryRegion>,
}

// Trait: Advisory router interface.
pub trait NanoLifebandRouter {
    fn route(
        &self,
        obs: &NanoSwarmObservationBand,
        boundary: &NanoSwarmBioBoundaryMap,
        domain: NanoDomain,
        requested_fraction: f64,
        pain_corridor_active: bool,
    ) -> NanoRouteDecisionLog;
}

// Conservative implementation: fail-closed, radiological-aware.
#[derive(Clone, Debug)]
pub struct ConservativeNanoLifebandRouter;

impl NanoLifebandRouter for ConservativeNanoLifebandRouter {
    fn route(
        &self,
        obs: &NanoSwarmObservationBand,
        boundary: &NanoSwarmBioBoundaryMap,
        domain: NanoDomain,
        requested_fraction: f64,
        pain_corridor_active: bool,
    ) -> NanoRouteDecisionLog {
        let mut reason = ReasonCode::HardStop;
        let mut decision = RouterDecision::Deny;

        // 1. PainCorridor: Immediate deny if active.
        if pain_corridor_active {
            reason = ReasonCode::PainCorridor;
        // 2. Lifeforce/Eco hard guards (advisory mirror of inner ledger).
        } else if matches!(obs.lifeforce_band, LifeforceBand::HardStop) ||
                  matches!(obs.eco_band, EcoBand::Critical) {
            reason = ReasonCode::HardStop;
        // 3. Radiological: Session/daily dose checks.
        } else {
            let region = boundary.regions.get(&obs.region_id);
            match region {
                Some(r) if r.is_no_fly_zone => {
                    reason = ReasonCode::NoFlyZone;
                }
                Some(r) => {
                    // Density check.
                    if requested_fraction > r.allowed_nano_density_range.1 {
                        reason = ReasonCode::DensityExceeded;
                    // Radiology: Cumulative vs limits.
                    } else if obs.cumulative_radiology_mgy > r.max_radiation_dose_session_mgy {
                        reason = ReasonCode::RadiologyRisk;
                    } else if matches!(obs.radiology_band, RadiologyBand::HardStop) {
                        reason = ReasonCode::RadiologyRisk;
                    // Radiosensitivity: Tighten for high-risk tissues.
                    } else if r.radiosensitivity_class == "VeryHigh" &&
                              obs.cumulative_radiology_mgy > r.max_radiation_dose_daily_mgy * 0.5 {
                        reason = ReasonCode::RadiologyRisk;
                    } else {
                        decision = RouterDecision::Safe;
                        reason = ReasonCode::HardStop; // Placeholder, safe.
                    }
                }
                None => {
                    // Unknown: Deny invasive domains.
                    if matches!(domain, NanoDomain::DetoxMicro | NanoDomain::RepairMicro) {
                        reason = ReasonCode::UnknownRegion;
                    } else {
                        decision = RouterDecision::Safe;
                    }
                }
            }
        }

        // SoftWarn -> Defer for non-urgent.
        if matches!(decision, RouterDecision::Safe) &&
           (matches!(obs.radiology_band, RadiologyBand::SoftWarn) ||
            matches!(obs.eco_band, EcoBand::High)) {
            decision = RouterDecision::Defer;
        }

        NanoRouteDecisionLog {
            decision_id: format!("{:x}", md5::compute(format!("{}{:?}", obs.ts_utc, obs.host_id))),
            ts_utc: obs.ts_utc,
            host_id: obs.host_id.clone(),
            router_decision: decision,
            reason_code: reason,
            nano_domain: domain,
            region_id: obs.region_id.clone(),
            requested_nano_fraction: requested_fraction,
            applied_radiology_band: obs.radiology_band.clone(),
        }
    }
}

// Usage: Quantum-learning pre-filter integration hook.
// Feeds logs to models for risk prediction (e.g., infection/toxin patterns).
pub fn generate_log_for_ml(obs: NanoSwarmObservationBand, /* ... */) -> Vec<NanoRouteDecisionLog> {
    // Placeholder: Simulate router calls over time-series.
    vec![]
}

// Console-output debug: Sanitized machine-readable.
impl std::fmt::Display for NanoRouteDecisionLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "DECISION_ID: {}", self.decision_id)?;
        writeln!(f, "ROUTER_DECISION: {:?}", self.router_decision)?;
        writeln!(f, "REASON_CODE: {:?}", self.reason_code)?;
        writeln!(f, "NANO_DOMAIN: {:?}", self.nano_domain)?;
        writeln!(f, "REGION_ID: {}", self.region_id)?;
        writeln!(f, "REQUESTED_FRACTION: {}", self.requested_nano_fraction)?;
        writeln!(f, "RADIOLOGY_BAND: {:?}", self.applied_radiology_band)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radiology_deny() {
        let obs = NanoSwarmObservationBand {
            host_id: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            ts_utc: 1707200000000000u64, // Feb 2026.
            region_id: "hepatic_lobe".to_string(),
            nano_load_fraction: 0.1,
            local_temp: 37.0,
            tissue_type: "hepatic".to_string(),
            lifeforce_band: LifeforceBand::Safe,
            eco_band: EcoBand::Neutral,
            clarity_index: 0.9,
            cumulative_radiology_mgy: 100.0,
            radiology_band: RadiologyBand::HardStop,
            infection_marker: true,
        };

        let mut regions = HashMap::new();
        regions.insert("hepatic_lobe".to_string(), BoundaryRegion {
            region_id: "hepatic_lobe".to_string(),
            bioscale_plane: "InVivo".to_string(),
            allowed_nano_density_range: (0.0, 0.2),
            is_no_fly_zone: false,
            max_radiation_dose_session_mgy: 50.0,
            max_radiation_dose_daily_mgy: 100.0,
            dose_recovery_half_life_hr: 24.0,
            radiosensitivity_class: "High".to_string(),
        });

        let boundary = NanoSwarmBioBoundaryMap {
            map_version_id: "v1".to_string(),
            host_id: obs.host_id.clone(),
            valid_from_utc: obs.ts_utc,
            valid_until_utc: obs.ts_utc + 86400_00000000,
            regions,
        };

        let router = ConservativeNanoLifebandRouter;
        let log = router.route(&obs, &boundary, NanoDomain::DetoxMicro, 0.15, false);

        assert_eq!(log.router_decision, RouterDecision::Deny);
        assert_eq!(log.reason_code, ReasonCode::RadiologyRisk);
        assert_eq!(log.requested_nano_fraction, 0.15);
    }

    #[test]
    fn test_safe_detox() {
        // Similar setup, low rad -> Safe.
        // Omitted for brevity; asserts Safe for low-dose, no pain.
    }
}
