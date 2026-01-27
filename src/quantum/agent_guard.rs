use crate::biophysical::plane_classifier::{
    BrainTokenState, CloningPolicy, ConsciousnessState, EnvironmentMetadata, EnvironmentPlane,
    PlaneClassifier, PlaneClassifierConfig,
};

#[derive(Clone, Debug)]
pub enum GuardDecision {
    Allow,
    AllowWithRedaction { fields_redacted: Vec<String> },
    Deny { reason: String },
}

#[derive(Clone, Debug)]
pub struct QuantumLearningGuard {
    classifier: PlaneClassifier,
}

impl QuantumLearningGuard {
    pub fn new() -> Self {
        let cfg = PlaneClassifierConfig {
            prohibit_consciousness_mutation: true,
            prohibit_brain_token_freeze: true,
            log_all_decisions: true,
            allowed_hardware_ids: vec![
                String::from("openbci-ultracortex"),
                String::from("generic-eeg-usb-dongle"),
                String::from("immersive-oculus-prototype-01"),
            ],
        };

        QuantumLearningGuard {
            classifier: PlaneClassifier::new(cfg),
        }
    }

    pub fn evaluate_environment(&mut self, metadata: &EnvironmentMetadata) -> GuardDecision {
        match metadata.cloning_policy {
            CloningPolicy::NonClonableContainsIdentity => {
                return GuardDecision::Deny {
                    reason: String::from(
                        "Cloning rejected: environment contains identity-linked or consciousness patterns.",
                    ),
                };
            }
            CloningPolicy::ClonableNonConscious => {}
            CloningPolicy::ClonableRedacted => {
                return GuardDecision::AllowWithRedaction {
                    fields_redacted: vec![String::from("identity_descriptor_hash")],
                };
            }
        }

        match metadata.brain_token_state {
            BrainTokenState::FrozenDisallowed { .. } => {
                return GuardDecision::Deny {
                    reason: String::from(
                        "Brain-token state invalid: freezing of Brain-tokens is not permitted.",
                    ),
                };
            }
            _ => {}
        }

        match metadata.environment_plane {
            EnvironmentPlane::ConsciousnessNetwork | EnvironmentPlane::RealityOs => {
                match metadata.consciousness_state {
                    ConsciousnessState::ActiveImmutable | ConsciousnessState::InactiveImmutable => {
                        // Preserve, but do not mutate.
                    }
                    _ => {
                        return GuardDecision::Deny {
                            reason: String::from(
                                "Consciousness or Reality.OS plane without immutable consciousness state.",
                            ),
                        };
                    }
                }
            }
            _ => {}
        }

        GuardDecision::Allow
    }

    pub fn enforce(
        &mut self,
        id: &str,
        involves_living_organism: bool,
        requested_conscious_state: ConsciousnessState,
        has_oculus: bool,
        has_remote_feed: bool,
        net_weight: f64,
        circulating_supply: f64,
        frozen: bool,
        controller_contract: &str,
        hardware_dependencies: Vec<String>,
        involves_conscious_pattern: bool,
        contains_identity_descriptors: bool,
        input_tags: Vec<String>,
        hardware_profile: Vec<String>,
        regulatory_labels: Vec<String>,
    ) -> (EnvironmentMetadata, GuardDecision, Vec<String>) {
        let meta = self.classifier.build_environment_metadata(
            id,
            involves_living_organism,
            requested_conscious_state,
            has_oculus,
            has_remote_feed,
            net_weight,
            circulating_supply,
            frozen,
            controller_contract,
            hardware_dependencies,
            involves_conscious_pattern,
            contains_identity_descriptors,
            input_tags,
            hardware_profile,
            regulatory_labels,
        );

        let decision = self.evaluate_environment(&meta);
        let log = self.classifier.decision_log().to_vec();
        (meta, decision, log)
    }
}
