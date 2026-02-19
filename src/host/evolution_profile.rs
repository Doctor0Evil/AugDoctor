use serde::{Deserialize, Serialize};

/// Which sensing planes this feature touches.
/// All channels are non-invasive: wearables or external sensors only.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SensorChannel {
    Emg,          // surface muscle activity
    Motion,       // IMUs / vision pose
    Eeg,          // wearable EEG, low-density
    HeartRate,
    Hrv,
    Respiration,
    Environment,  // air, water, heat, noise
}

/// Which ledger planes this feature is allowed to influence.
/// These are *policy knobs*, not raw token writes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LedgerPlane {
    Brain,
    Wave,
    Blood,
    Oxygen,
    Nano,
    Smart,
    EcoScore,
    EvolveEligibility,
}

/// High-level category, used for evolution routing.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeatureClass {
    ControlInterface,
    MemoryRestoration,
    LoadGovernor,
    FirmwareRealign,
    EcoGovernance,
    EvolutionPolicy,
    TelemetryAudit,
}

/// Neurorights and safety flags that must hold for this feature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RightsEnvelope {
    /// Never infer beliefs or inner narratives from EEG/physiology.
    pub no_inner_state_scoring: bool,
    /// Refusing this feature must not affect core civil access.
    pub no_coercive_uptake: bool,
    /// All actuation must be reversible without surgery.
    pub reversible_and_non_implant: bool,
    /// Feature must enforce BCI/biocompatibility ceilings if EEG is used.
    pub enforce_bci_ceiling: bool,
}

/// A single cybernetic feature that can be enabled for a host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CyberneticFeature {
    pub id: String,
    pub label: String,
    pub description: String,
    pub class: FeatureClass,
    pub sensors: Vec<SensorChannel>,
    pub affects: Vec<LedgerPlane>,
    /// True means this feature can only *propose* adjustments;
    /// inner-ledger guards still decide.
    pub proposal_only: bool,
    /// Neurorights + biophysical safety guarantees.
    pub rights: RightsEnvelope,
}

/// A named evolution profile: a bundle of features you choose to enable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionProfile {
    pub profile_id: String,
    pub label: String,
    pub long_description: String,
    pub features: Vec<CyberneticFeature>,
}

/// Phoenix-grade, no-implant default profiles for a body-sensor host.
pub fn phoenix_default_profiles() -> Vec<EvolutionProfile> {
    let rights_strict = RightsEnvelope {
        no_inner_state_scoring: true,
        no_coercive_uptake: true,
        reversible_and_non_implant: true,
        enforce_bci_ceiling: true,
    };

    let control_feature = CyberneticFeature {
        id: "ctrl.bodybus.v1".to_string(),
        label: "BodySensor Host Interface v1".to_string(),
        description: "Maps EMG, motion, and optional EEG into high-level \
                      intents (select, navigate, approve, cancel, panic) \
                      to control Reality.os and AI-chat without keyboard/mouse.",
        class: FeatureClass::ControlInterface,
        sensors: vec![SensorChannel::Emg, SensorChannel::Motion, SensorChannel::Eeg],
        affects: vec![LedgerPlane::Brain, LedgerPlane::Wave],
        proposal_only: true,
        rights: rights_strict.clone(),
    };

    let memrestore_feature = CyberneticFeature {
        id: "mem.restore.v1".to_string(),
        label: "Memory Restoration Pipeline v1".to_string(),
        description: "Enables snapshot/restore gestures, OS integrity checks, \
                      and recovery-box / engram-vault workflows without any \
                      kernel or filesystem version changes.",
        class: FeatureClass::MemoryRestoration,
        sensors: vec![SensorChannel::Emg, SensorChannel::Motion, SensorChannel::Hrv],
        affects: vec![
            LedgerPlane::Brain,
            LedgerPlane::Wave,
            LedgerPlane::EvolveEligibility,
        ],
        proposal_only: true,
        rights: rights_strict.clone(),
    };

    let loadgov_feature = CyberneticFeature {
        id: "load.governor.v1".to_string(),
        label: "Cognitive Load & Pain-Band Governor v1".to_string(),
        description: "Monitors error rates and physiological overload signals \
                      to propose reductions in WAVE/SMART and schedule \
                      restorative actions inside safe lifeforce bands.",
        class: FeatureClass::LoadGovernor,
        sensors: vec![
            SensorChannel::Emg,
            SensorChannel::Eeg,
            SensorChannel::HeartRate,
            SensorChannel::Hrv,
            SensorChannel::Respiration,
        ],
        affects: vec![
            LedgerPlane::Wave,
            LedgerPlane::Smart,
            LedgerPlane::EvolveEligibility,
        ],
        proposal_only: true,
        rights: rights_strict.clone(),
    };

    let firmware_feature = CyberneticFeature {
        id: "firmware.realign.v1".to_string(),
        label: "Multisignal BIOS/UEFI Realign v1".to_string(),
        description: "Provides a high-threshold, multisignal gesture + calm-state \
                      confirmation path to schedule BIOS/UEFI defaults reload, \
                      without flashing firmware or changing boot chain.",
        class: FeatureClass::FirmwareRealign,
        sensors: vec![SensorChannel::Emg, SensorChannel::Eeg],
        affects: vec![LedgerPlane::Brain, LedgerPlane::Wave],
        proposal_only: true,
        rights: rights_strict.clone(),
    };

    let eco_feature = CyberneticFeature {
        id: "eco.corridor.v1".to_string(),
        label: "EcoCorridor & Karma Governance v1".to_string(),
        description: "Binds host compute and nanoswarm use to local ecological \
                      safety polytopes and non-financial Karma ledgers, \
                      without affecting civil rights or inner mental states.",
        class: FeatureClass::EcoGovernance,
        sensors: vec![SensorChannel::Environment],
        affects: vec![LedgerPlane::Nano, LedgerPlane::EcoScore],
        proposal_only: true,
        rights: rights_strict.clone(),
    };

    let evo_policy_feature = CyberneticFeature {
        id: "evolve.policy.v1".to_string(),
        label: "EVOLVE Eligibility & Correction v1".to_string(),
        description: "Implements EvolutionEligibilityFilter and \
                      EvolutionCorrection events so that only \
                      biocompatible, consented domains can influence EVOLVE, \
                      and eco-restoration remains knowledge-only by default.",
        class: FeatureClass::EvolutionPolicy,
        sensors: vec![SensorChannel::Environment],
        affects: vec![LedgerPlane::EvolveEligibility],
        proposal_only: false,
        rights: rights_strict.clone(),
    };

    vec![
        EvolutionProfile {
            profile_id: "phx.control.stability".to_string(),
            label: "Phoenix Control + Stability Pack".to_string(),
            long_description: "Body-only control, memory restoration, and \
                               load governor features for a no-implant host \
                               under strict neurorights and eco constraints.",
            features: vec![
                control_feature.clone(),
                memrestore_feature.clone(),
                loadgov_feature.clone(),
            ],
        },
        EvolutionProfile {
            profile_id: "phx.eco.evolve".to_string(),
            label: "Phoenix Eco + Evolution Governance Pack".to_string(),
            long_description: "EcoCorridor governance and EVOLVE policy \
                               features that keep evolution steps \
                               biocompatible, non-financial, and host-sovereign.",
            features: vec![eco_feature, evo_policy_feature],
        },
        EvolutionProfile {
            profile_id: "phx.full.nonimplant".to_string(),
            label: "Phoenix Full Non-Implant Host Profile".to_string(),
            long_description: "All non-invasive cybernetic features enabled \
                               for a body-sensor host, with BIOS realign, \
                               stability, eco, and evolution guards.",
            features: vec![
                control_feature,
                memrestore_feature,
                loadgov_feature,
                firmware_feature,
                eco_feature,
                evo_policy_feature,
            ],
        },
    ]
}
