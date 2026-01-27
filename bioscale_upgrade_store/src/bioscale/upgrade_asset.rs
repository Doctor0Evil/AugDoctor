use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Awareness profile for bioscale upgrades: is there living tissue, and how
/// tightly coupled is the upgrade to the organism.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioscaleAwarenessProfile {
    pub involves_living_organism: bool,
    pub tissue_interface: Vec<String>,
    pub organ_targets: Vec<String>,
    pub biosignal_channels: Vec<String>,
}

/// Consciousness compliance: guarantees that this upgrade cannot introduce,
/// modify, or quantify consciousness states.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsciousnessComplianceLevel {
    /// No interaction with any conscious substrate (purely peripheral).
    NoConsciousSubstrate,
    /// Indirect interaction, but all signals are one-way and non-identity-bearing.
    IndirectNonIdentity,
    /// Direct interaction, but with hard-coded immutability and no quantification.
    DirectImmutableNonQuantifying,
}

/// Hardware binding: what devices and IDs this upgrade is allowed to operate with.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HardwareBindingProfile {
    pub allowed_hardware_ids: Vec<String>,
    pub required_safety_modules: Vec<String>,
    pub bioscale_resolution_microns: u32,
}

/// A single bioscale upgrade asset in the store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioscaleUpgradeAsset {
    pub id: String,
    pub human_label: String,
    pub version: String,
    pub awareness_profile: BioscaleAwarenessProfile,
    pub consciousness_compliance: ConsciousnessComplianceLevel,
    pub hardware_binding: HardwareBindingProfile,
    pub tags: Vec<String>,
    pub metadata_hash: String,
}

impl BioscaleUpgradeAsset {
    pub fn new(
        human_label: &str,
        awareness_profile: BioscaleAwarenessProfile,
        consciousness_compliance: ConsciousnessComplianceLevel,
        hardware_binding: HardwareBindingProfile,
        tags: Vec<String>,
        metadata_hash: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            human_label: human_label.to_string(),
            version: String::from("0.1.0"),
            awareness_profile,
            consciousness_compliance,
            hardware_binding,
            tags,
            metadata_hash: metadata_hash.to_string(),
        }
    }

    /// Helper function to declare if this asset ever touches brain tokens or
    /// any identity-related channel (by tags).
    pub fn implies_brain_tokens(&self) -> bool {
        self.tags
            .iter()
            .any(|t| t.to_lowercase().contains("brain-token"))
    }

    pub fn implies_identity_pattern(&self) -> bool {
        self.tags
            .iter()
            .any(|t| t.to_lowercase().contains("identity") || t.to_lowercase().contains("soul"))
    }
}
