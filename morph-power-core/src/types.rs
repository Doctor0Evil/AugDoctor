use serde::{Deserialize, Serialize};

/// The four typed MORPH dimensions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MorphVector {
    /// Eco-upgrades: capacity for upgrades that reduce eco cost (FLOPs, nJ, device hours).
    pub eco: f32,
    /// Cybernetic influence: capacity to reshape other agents' environments (policy-gated).
    pub cyber: f32,
    /// Neuromorphic capabilities: capacity for neuromorph duty, spike density, E_chi, S_bio.
    pub neuro: f32,
    /// SMART capacity: capacity for planner depth, concurrent SMART channels, etc.
    pub smart: f32,
}

/// Scalar EVOLVE budget for this host and epoch.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MorphBudgetCorridorSpec {
    /// EVOLVE scalar Eevolve ≥ 0, non-mintable, host-issued only.
    pub evolve_scalar: f32,
    /// Optional per-dimension max caps to shape ∥M∥₁.
    pub max_eco: f32,
    pub max_cyber: f32,
    pub max_neuro: f32,
    pub max_smart: f32,
}

/// Per-upgrade MORPH usage declaration.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MorphUsage {
    /// Required MORPH slice to apply this upgrade (pre-application).
    pub required_morph: MorphVector,
    /// Delta consumed from the host's current MORPH state.
    pub delta_morph: MorphVector,
}

/// Evidence tags used to justify MORPH consumption.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MorphEvidenceTag {
    AtpLevel,
    Cmro2,
    Il6,
    ThermalLoad,
    FatigueIndex,
    PainProxy,
    EcoKwhDelta,
    DeviceHoursDelta,
    SpikeDensityDelta,
    MetabolicBurdenDelta,
}

/// Evidence bundle: at least 10 typed biophysical datapoints.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MorphEvidenceBundle {
    pub tags: Vec<MorphEvidenceTag>,
    /// Normalized values (host-local, 0..1 or signed deltas).
    pub values: Vec<f32>,
    /// Hex or ALN proof handle into QPU.Datashards / research corpus.
    pub proof_hex: String,
}

/// POWER: per-turn scalar envelope for agentic actions.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PowerBudget {
    /// Max "work surface area" for this turn (0..1, non-financial).
    pub corridor: f32,
    /// Hard-min corridor under any condition (e.g., 0.0 for full stop).
    pub min_corridor: f32,
    /// Hard-max corridor (e.g., 1.0 for fully open, but never > 1).
    pub max_corridor: f32,
}

/// Context needed to decide POWER for the next turn.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PowerContext {
    /// Lifeforce scalar 0..1 from LifeforceBandSeries.
    pub lifeforce_scalar: f32,
    /// SoftWarn / HardStop encoded as 0 (OK), 1 (SoftWarn), 2 (HardStop).
    pub lifeforce_band_code: u8,
    /// Recent civic / cryptographic impact score 0..1.
    pub civic_impact: f32,
    /// Recent neuromorph / eco load 0..1 (EcoBandProfile-normalized).
    pub eco_load: f32,
    /// Host eco-behavior score (longitudinal eco impact) 0..1.
    pub eco_score_longitudinal: f32,
    /// Number of high-impact actions in last N turns.
    pub recent_high_impact_actions: u32,
}

/// Immutable reasons why POWER is constrained this turn.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerProhibitionReason {
    HardStopLifeforce,
    SoftWarnLifeforce,
    HighCivicImpact,
    HighEcoLoad,
    PoorEcoBehavior,
    TooManyRecentHighImpactActions,
    DefaultFloor,
}

/// Final POWER decision for the agent this turn.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PowerDecision {
    pub allowed_corridor: f32,
    pub reasons: Vec<PowerProhibitionReason>,
    /// True if the agent MUST operate in propose-only mode.
    pub propose_only: bool,
    /// Upper bounds for this turn: max docs, max external calls, etc.
    pub max_documents: u32,
    pub max_external_calls: u32,
}
