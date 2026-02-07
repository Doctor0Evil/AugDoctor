//! Nanoswarm observation, routing logs, and radiology-aware bio-boundary maps.
//!
//! Layer: Eibon biosphere-observation only.
//! - Purely descriptive, host-local structs (no tokens, no balances).
//! - Feed quantum-learning pre-filters and orchestrator policy.
//! - Never appear inside BioTokenState or mutation mechanics.
//!
//! Invariants:
//! - BRAIN/BLOOD/OXYGEN/NANO/SMART invariants remain the only inner guards.
//! - Radiology is modeled as a tightening band over WAVE/NANO, never a relaxant.
//! - PainCorridor and BCI signals are treated as HardStop equivalents for somatic domains.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::types::{
    LifeforceBand,
    EcoBandProfile,
};
use crate::types::IdentityHeader; // DID, role, tier; for audit pointers only.

/// Where nanoswarm packets are flowing.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NanoDomain {
    /// Pure compute assist, no direct somatic contact.
    ComputeAssist,
    /// Sensor housekeeping, calibration, diagnostics.
    SensorHousekeeping,
    /// Micro‑scale tissue repair.
    RepairMicro,
    /// Micro‑scale detoxification/removal of toxins, pathogens, debris.
    DetoxMicro,
}

/// Router decision outcome for a nanoswarm packet or workload slice.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NanoRouteDecision {
    Safe,
    Defer,
    Deny,
}

/// High‑level reason for a router decision, aligned with lifeforce/eco/radiology doctrine.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NanoRouteReasonCode {
    /// Inner lifeforce guard HardStop (BLOOD/OXYGEN/BRAIN floors, PainCorridor, etc.).
    HardStop,
    /// Eco band too high, ecocost near or above ecoflopslimit.
    EcoHigh,
    /// Persistent nociceptive pattern in region; treated as HardStop for somatic ops.
    PainCorridor,
    /// Clarity too low, BCI/EEG indicates unsafe cognitive state.
    LowClarity,
    /// Radiosensitivity and dose history place region into SoftWarn band.
    RadiologySoftWarn,
    /// Radiosensitivity and cumulative dose place region into HardStop band.
    RadiologyHardStop,
    /// Region is a nanoswarm no‑fly zone per boundary map.
    NoFlyZone,
    /// Generic soft guard; used when multiple soft‑constraints coincide.
    SoftGuard,
    /// Explicit manual safety override by host (still subordinate to invariants).
    HostOverrideSoft,
    /// Reserved for future typed reasons.
    Other(String),
}

/// Bioscale plane in which the region exists.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BioScalePlane {
    /// In‑vivo host tissue.
    InVivo,
    /// Ex‑vivo devices, scaffolds, or implants outside the body.
    ExVivo,
    /// In‑silico sandbox/simulation.
    InSilico,
}

/// Coarse radiosensitivity class for a region.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RadioSensitivityClass {
    /// Highly radiosensitive marrow, gonads, pediatric growth plates, etc.
    MarrowHigh,
    NeuralVeryHigh,
    /// Moderately sensitive soft tissue.
    SoftTissueMedium,
    /// Low sensitivity (cortical bone, certain inert implants).
    BoneLow,
    /// Explicitly unknown/unspecified; treated conservatively.
    Unknown,
}

/// Radiology safety band derived from cumulative dose and radiosensitivity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RadiologyBand {
    Safe,
    SoftWarn,
    HardStop,
}

/// Time‑aligned nanoswarm observation for quantum models and audit.
/// Lives strictly in Eibon biosphere‑observation; never as a token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoSwarmObservationBand {
    /// Host DID; binds observation to a single sovereign host.
    pub host_id: String,
    /// Monotonic ID for this observation series within host.
    pub seq_local: u64,
    /// Observation timestamp (UTC).
    pub ts_utc: DateTime<Utc>,

    /// Fraction of host NANO capacity in use [0.0, 1.0].
    pub nanoload_fraction: f64,
    /// Local temperature in degrees Celsius.
    pub local_temp_c: f64,
    /// Short tag for tissue type (e.g., "neural", "vascular", "hepatic").
    pub tissue_type: String,

    /// Current lifeforce band at the observation locus.
    pub lifeforce_band: LifeforceBand,
    /// Current eco profile at host level (Low/Medium/High, etc.).
    pub eco_band: EcoBandProfile,

    /// Cognitive/sensory clarity index [0.0, 1.0]; lower = more noise/confusion.
    pub clarity_index: f64,

    // Radiology context extensions:
    /// Cumulative local radiology dose in mGy (or domain‑specific unit).
    pub cumulative_radiology_dose_local: f64,
    /// Radiology safety band for this region (Safe/SoftWarn/HardStop).
    pub radiology_band: RadiologyBand,
    /// Flag from lab imaging/biosensors indicating infection marker presence.
    pub infection_marker_present: bool,

    /// Optional pointer into a CivicAuditLog entry (hex hash, CID, etc.).
    pub civic_audit_ref: Option<String>,
}

/// Router‑level decision log entry for a nanoswarm routing event.
/// This is labeled ground truth for risk models; *never* a control knob.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoRouteDecisionLog {
    /// Globally unique decision ID.
    pub decision_id: Uuid,
    /// Host‑local monotonically increasing index.
    pub host_seq: u64,
    /// Host DID.
    pub host_id: String,
    /// UTC decision timestamp.
    pub ts_utc: DateTime<Utc>,

    /// Outcome of the router.
    pub router_decision: NanoRouteDecision,
    /// Typed reason code for downstream analysis.
    pub reason_code: NanoRouteReasonCode,
    /// Domain this packet/workload belonged to.
    pub nano_domain: NanoDomain,

    /// Region identifier this decision pertains to (matches boundary map).
    pub region_id: String,
    /// Plane in which the region exists (in‑vivo, ex‑vivo, in‑silico).
    pub bioscale_plane: BioScalePlane,

    /// Snapshot of lifeforce and eco context at decision time.
    pub lifeforce_band: LifeforceBand,
    pub eco_band: EcoBandProfile,
    /// Pain corridor signal (0.0‑1.0) derived from EEG/neural metrics.
    pub pain_corridor_level: f32,
    /// Radiology band at decision time.
    pub radiology_band: RadiologyBand,

    /// Optional identity header hash for audit (no direct identities here).
    pub identity_header_hash: Option<String>,
    /// Optional pointer into CivicAuditLog entry for the triggering event.
    pub civic_audit_ref: Option<String>,
}

/// Nano density bounds for a given bioscale region.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoDensityRange {
    /// Minimum safe fraction [0.0, 1.0].
    pub min_fraction: f64,
    /// Maximum safe fraction [0.0, 1.0].
    pub max_fraction: f64,
}

/// Radiology envelope for a bioscale region.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RadiologyEnvelope {
    /// Maximum local dose per session (same units as cumulative_radiology_dose_local).
    pub max_dose_per_session: f64,
    /// Maximum local dose per day.
    pub max_dose_per_day: f64,
    /// Half‑life in hours for allowed dose recovery (policy, not physics).
    pub dose_recovery_half_life_hours: f64,
    /// Radiosensitivity class.
    pub radiosensitivity_class: RadioSensitivityClass,
}

/// Bio‑boundary entry for a specific region and bioscale plane.
/// This is a descriptive map only; inner invariants do all enforcement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoSwarmBioBoundaryRegion {
    /// Map version identifier; allows rollback/history.
    pub map_version_id: Uuid,
    /// Host DID this map belongs to.
    pub host_id: String,

    /// When this map version becomes active / expires.
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,

    /// Region identifier (e.g., "hepatic-lobe-2", "lumbar-marrow-L3").
    pub region_id: String,
    /// Bioscale plane (in‑vivo, ex‑vivo, in‑silico).
    pub bioscale_plane: BioScalePlane,

    /// Allowed nanoswarm density band for this region.
    pub allowed_nano_density: NanoDensityRange,
    /// Whether this region is a full nanoswarm no‑fly zone.
    pub is_no_fly_zone: bool,

    /// Optional ID linking to known pain-linked locus (BCI/PainCorridor map).
    pub associated_pain_locus_id: Option<String>,

    /// Radiology envelope for this region.
    pub radiology_envelope: RadiologyEnvelope,

    /// Optional ID of the quantum model that last recommended this boundary.
    pub last_updated_by_model_id: Option<String>,
    /// Optional pointer into CivicAuditLog for governance review.
    pub civic_audit_ref: Option<String>,
}

/// Full boundary map for a host (typically stored as an ALN shard).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoSwarmBioBoundaryMap {
    /// Unique map version ID.
    pub map_version_id: Uuid,
    /// Host DID this map applies to.
    pub host_id: String,
    /// Activation window.
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    /// Collection of per‑region entries.
    pub regions: Vec<NanoSwarmBioBoundaryRegion>,
}

/// Abstract interface for nanoswarm routing with radiology & pain corridors.
///
/// This trait is deliberately kept outside inner‑ledger mutation mechanics.
/// It consumes lifeforce/eco/pain/radiology context and emits *advisory*
/// decisions + logs for orchestrators; inner invariants still decide.
///
/// Implementors live in boundary/orchestrator crates (host‑local).
pub trait NanoLifebandRouter {
    /// Classify a nanoswarm workload for a region into Safe/Defer/Deny
    /// and produce a decision log entry.
    ///
    /// Inputs:
    /// - observation: most recent NanoSwarmObservationBand for the locus.
    /// - region: boundary region envelope (density + radiology).
    /// - domain: nanoswarm domain (compute, detox_micro, etc.).
    /// - pain_corridor_level: [0.0, 1.0] nociceptive signal for region.
    ///
    /// Contract:
    /// - PainCorridor hard veto for somatic domains when high.
    /// - Radiology HardStop veto when dose exceeds region envelope.
    /// - Radiology SoftWarn can still allow Safe/Defer but must downgrade.
    /// - Must *never* relax core invariants; decisions are hints only.
    fn classify_with_radiology(
        &self,
        observation: &NanoSwarmObservationBand,
        region: &NanoSwarmBioBoundaryRegion,
        domain: NanoDomain,
        pain_corridor_level: f32,
    ) -> NanoRouteDecisionLog;
}

/// A simple, host‑local policy router implementation suitable for lab use.
/// This lives in boundary/orchestrator crates, not in inner ledger.
#[derive(Clone, Debug)]
pub struct DefaultNanoLifebandRouterPolicy {
    /// Pain corridor HardStop threshold (e.g., 0.8).
    pub pain_hardstop_threshold: f32,
    /// Pain corridor SoftWarn threshold (e.g., 0.5).
    pub pain_softwarn_threshold: f32,
    /// Radiology SoftWarn multiplier on allowed nanoload (e.g., 0.5).
    pub radiology_soft_nano_factor: f64,
    /// Radiology HardStop cut‑off multiplier (e.g., 0.1).
    pub radiology_hard_nano_factor: f64,
}

impl DefaultNanoLifebandRouterPolicy {
    pub fn new() -> Self {
        Self {
            pain_hardstop_threshold: 0.8,
            pain_softwarn_threshold: 0.5,
            radiology_soft_nano_factor: 0.5,
            radiology_hard_nano_factor: 0.1,
        }
    }

    fn decide_internal(
        &self,
        obs: &NanoSwarmObservationBand,
        region: &NanoSwarmBioBoundaryRegion,
        domain: &NanoDomain,
        pain_corridor_level: f32,
    ) -> (NanoRouteDecision, NanoRouteReasonCode, RadiologyBand) {
        // 1. No‑fly zones: immediate Deny.
        if region.is_no_fly_zone {
            return (
                NanoRouteDecision::Deny,
                NanoRouteReasonCode::NoFlyZone,
                obs.radiology_band.clone(),
            );
        }

        // 2. Pain corridor as HardStop for somatic domains.
        let is_somatic_domain = matches!(
            domain,
            NanoDomain::RepairMicro | NanoDomain::DetoxMicro | NanoDomain::SensorHousekeeping
        );

        if is_somatic_domain && pain_corridor_level >= self.pain_hardstop_threshold {
            return (
                NanoRouteDecision::Deny,
                NanoRouteReasonCode::PainCorridor,
                obs.radiology_band.clone(),
            );
        }

        // 3. Lifeforce HardStop: Deny for all domains.
        if matches!(obs.lifeforce_band, LifeforceBand::HardStop) {
            return (
                NanoRouteDecision::Deny,
                NanoRouteReasonCode::HardStop,
                obs.radiology_band.clone(),
            );
        }

        // 4. EcoHigh: Defer for non‑critical domains when Eco is max/High.
        // (Exact band extraction is delegated to EcoBandProfile semantics.)
        let eco_high = obs.eco_band.is_high(); // Implement is_high() on EcoBandProfile.
        if eco_high && matches!(domain, NanoDomain::ComputeAssist) {
            return (
                NanoRouteDecision::Defer,
                NanoRouteReasonCode::EcoHigh,
                obs.radiology_band.clone(),
            );
        }

        // 5. Radiology envelope logic: tighten allowed nanoload.
        let density = obs.nanoload_fraction;
        let base_max = region.allowed_nano_density.max_fraction;
        let (rad_band, radiology_factor, rad_reason) = match obs.radiology_band {
            RadiologyBand::Safe => (RadiologyBand::Safe, 1.0, None),
            RadiologyBand::SoftWarn => (
                RadiologyBand::SoftWarn,
                self.radiology_soft_nano_factor,
                Some(NanoRouteReasonCode::RadiologySoftWarn),
            ),
            RadiologyBand::HardStop => (
                RadiologyBand::HardStop,
                self.radiology_hard_nano_factor,
                Some(NanoRouteReasonCode::RadiologyHardStop),
            ),
        };

        let max_allowed = base_max * radiology_factor;

        if density > max_allowed {
            // Hard radiology stop for somatic domains when beyond allowed band.
            let reason = rad_reason.unwrap_or(NanoRouteReasonCode::RadiologyHardStop);
            return (
                NanoRouteDecision::Deny,
                reason,
                rad_band,
            );
        }

        // 6. Soft pain corridor & clarity handling for non‑HardStop cases.
        if pain_corridor_level >= self.pain_softwarn_threshold && is_somatic_domain {
            return (
                NanoRouteDecision::Defer,
                NanoRouteReasonCode::PainCorridor,
                rad_band,
            );
        }

        // 7. Clarity too low: Defer to protect cognition.
        if obs.clarity_index < 0.3 {
            return (
                NanoRouteDecision::Defer,
                NanoRouteReasonCode::LowClarity,
                rad_band,
            );
        }

        // 8. Default: Safe.
        (NanoRouteDecision::Safe, NanoRouteReasonCode::SoftGuard, rad_band)
    }
}

impl NanoLifebandRouter for DefaultNanoLifebandRouterPolicy {
    fn classify_with_radiology(
        &self,
        observation: &NanoSwarmObservationBand,
        region: &NanoSwarmBioBoundaryRegion,
        domain: NanoDomain,
        pain_corridor_level: f32,
    ) -> NanoRouteDecisionLog {
        let decision_id = Uuid::new_v4();
        let (router_decision, reason_code, rad_band) =
            self.decide_internal(observation, region, &domain, pain_corridor_level);

        NanoRouteDecisionLog {
            decision_id,
            host_seq: observation.seq_local,
            host_id: observation.host_id.clone(),
            ts_utc: observation.ts_utc,

            router_decision,
            reason_code,
            nano_domain: domain,

            region_id: region.region_id.clone(),
            bioscale_plane: region.bioscale_plane.clone(),

            lifeforce_band: observation.lifeforce_band.clone(),
            eco_band: observation.eco_band.clone(),
            pain_corridor_level,
            radiology_band: rad_band,

            identity_header_hash: None,
            civic_audit_ref: observation.civic_audit_ref.clone(),
        }
    }
}
