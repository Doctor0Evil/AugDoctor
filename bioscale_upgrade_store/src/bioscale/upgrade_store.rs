use crate::bioscale::upgrade_asset::{
    BioscaleAwarenessProfile, BioscaleUpgradeAsset, ConsciousnessComplianceLevel,
    HardwareBindingProfile,
};
use augdoctor_core::biophysical::plane_classifier::{
    ConsciousnessState, EnvironmentMetadata, EnvironmentPlane,
};
use augdoctor_core::quantum::agent_guard::{GuardDecision, QuantumLearningGuard};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Configuration details for the bioscale upgrade store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BioscaleStoreConfig {
    pub allow_offline_registration: bool,
    pub default_regulatory_labels: Vec<String>,
    pub max_upgrade_assets: usize,
}

/// Errors that can occur while working with the bioscale upgrade store.
#[derive(Debug, Error)]
pub enum BioscaleStoreError {
    #[error("registry full: max assets={0}")]
    RegistryFull(usize),
    #[error("asset not found: {0}")]
    AssetNotFound(String),
    #[error("guard denied upgrade: {0}")]
    GuardDenied(String),
    #[error("hardware not compatible with upgrade: {0}")]
    HardwareMismatch(String),
    #[error("storage error: {0}")]
    StorageError(String),
}

/// Result of applying a bioscale upgrade to a given environment profile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeApplicationResult {
    pub environment_metadata: EnvironmentMetadata,
    pub guard_decision: String,
    pub redacted_fields: Vec<String>,
}

pub struct BioscaleUpgradeStore {
    cfg: BioscaleStoreConfig,
    registry: HashMap<String, BioscaleUpgradeAsset>,
    guard: QuantumLearningGuard,
}

impl BioscaleUpgradeStore {
    pub fn new(cfg: BioscaleStoreConfig) -> Self {
        BioscaleUpgradeStore {
            cfg,
            registry: HashMap::new(),
            guard: QuantumLearningGuard::new(),
        }
    }

    pub fn register_upgrade(
        &mut self,
        asset: BioscaleUpgradeAsset,
    ) -> Result<String, BioscaleStoreError> {
        if self.registry.len() >= self.cfg.max_upgrade_assets {
            return Err(BioscaleStoreError::RegistryFull(
                self.cfg.max_upgrade_assets,
            ));
        }
        let id = asset.id.clone();
        self.registry.insert(asset.id.clone(), asset);
        Ok(id)
    }

    pub fn get_upgrade(&self, id: &str) -> Result<&BioscaleUpgradeAsset, BioscaleStoreError> {
        self.registry
            .get(id)
            .ok_or_else(|| BioscaleStoreError::AssetNotFound(id.to_string()))
    }

    /// Check if the requested hardware profile is compatible with the upgrade's
    /// HardwareBindingProfile.
    fn hardware_compatible(
        binding: &HardwareBindingProfile,
        environment_hardware: &[String],
    ) -> bool {
        if binding.allowed_hardware_ids.is_empty() {
            return true;
        }
        binding.allowed_hardware_ids.iter().any(|allowed| {
            environment_hardware
                .iter()
                .any(|hw| hw.to_lowercase() == allowed.to_lowercase())
        })
    }

    /// Apply an upgrade asset to a given environment hardware and tagging
    /// configuration, enforcing all AugDoctor rules and returning a detailed
    /// decision.
    pub fn apply_upgrade_to_environment(
        &mut self,
        upgrade_id: &str,
        environment_id: &str,
        environment_hardware: Vec<String>,
        environment_tags: Vec<String>,
    ) -> Result<UpgradeApplicationResult, BioscaleStoreError> {
        let asset = self.get_upgrade(upgrade_id)?.clone();

        if !Self::hardware_compatible(&asset.hardware_binding, &environment_hardware) {
            return Err(BioscaleStoreError::HardwareMismatch(format!(
                "environment hardware {:?} incompatible with allowed {:?}",
                environment_hardware, asset.hardware_binding.allowed_hardware_ids
            )));
        }

        let awareness: &BioscaleAwarenessProfile = &asset.awareness_profile;
        let involves_living_organism = awareness.involves_living_organism;
        let has_oculus = environment_tags
            .iter()
            .any(|t| t.to_lowercase().contains("oculus") || t.to_lowercase().contains("vr"));
        let has_remote_feed = environment_tags
            .iter()
            .any(|t| t.to_lowercase().contains("remote-feed"));

        let requested_conscious_state = match asset.consciousness_compliance {
            ConsciousnessComplianceLevel::NoConsciousSubstrate => ConsciousnessState::None,
            ConsciousnessComplianceLevel::IndirectNonIdentity => ConsciousnessState::InactiveImmutable,
            ConsciousnessComplianceLevel::DirectImmutableNonQuantifying => {
                ConsciousnessState::ActiveImmutable
            }
        };

        let net_weight = if asset.implies_brain_tokens() { 1.0 } else { 0.0 };
        let circulating_supply = if asset.implies_brain_tokens() { 1000.0 } else { 0.0 };
        let frozen = false;
        let controller_contract = "0x519fC0eB4111323Cac44b70e1aE31c30e405802D";

        let involves_conscious_pattern = matches!(
            asset.consciousness_compliance,
            ConsciousnessComplianceLevel::DirectImmutableNonQuantifying
        );
        let contains_identity_descriptors = asset.implies_identity_pattern();

        let mut tags_combined = environment_tags.clone();
        tags_combined.extend(asset.tags.clone());

        let mut hardware_combined = environment_hardware.clone();
        hardware_combined.extend(asset.hardware_binding.allowed_hardware_ids.clone());

        let regulatory_labels = self.cfg.default_regulatory_labels.clone();

        let (meta, decision, _log) = self.guard.enforce(
            environment_id,
            involves_living_organism,
            requested_conscious_state,
            has_oculus,
            has_remote_feed,
            net_weight,
            circulating_supply,
            frozen,
            controller_contract,
            asset.hardware_binding.required_safety_modules.clone(),
            involves_conscious_pattern,
            contains_identity_descriptors,
            tags_combined,
            hardware_combined,
            regulatory_labels,
        );

        match meta.environment_plane {
            EnvironmentPlane::Bioscale | EnvironmentPlane::Biophysics => {}
            _ => {
                return Err(BioscaleStoreError::GuardDenied(
                    "environment not classified as bioscale/biophysics".to_string(),
                ));
            }
        }

        let (guard_decision, redacted_fields) = match decision {
            GuardDecision::Allow => ("Allow".to_string(), Vec::new()),
            GuardDecision::AllowWithRedaction { fields_redacted } => {
                ("AllowWithRedaction".to_string(), fields_redacted)
            }
            GuardDecision::Deny { reason } => {
                return Err(BioscaleStoreError::GuardDenied(reason));
            }
        };

        Ok(UpgradeApplicationResult {
            environment_metadata: meta,
            guard_decision,
            redacted_fields,
        })
    }
}
