use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EnvironmentPlane {
    Cybernetics,
    Augmented,
    Transhuman,
    Evolution,
    Bioscale,
    Biophysics,
    Organic,
    BciHciEeg,
    RealityOs,
    ConsciousnessNetwork,
    NeuralNetwork,
    Wetware,
    Bioware,
    SoftwareOnly,
    Unknown,
}

#[derive(Clone, Debug)]
pub enum ConsciousnessState {
    None,
    InactiveSticky,
    InactiveImmutable,
    ActiveSticky,
    ActiveImmutable,
}

#[derive(Clone, Debug)]
pub enum AwarenessFlag {
    NoOrganism,
    LivingOrganismInvolved,
}

#[derive(Clone, Debug)]
pub enum OculusFlag {
    NoOculus,
    OculusLocalOnly,
    OculusRemoteFeed,
}

#[derive(Clone, Debug)]
pub enum FeedbackSensorFlag {
    NoFeedback,
    LocalFeedback,
    RemoteFeedback,
}

#[derive(Clone, Debug)]
pub enum BrainTokenState {
    None,
    Circulating {
        net_weight: f64,
        circulating_supply: f64,
        controller_contract: String,
        hardware_dependencies: Vec<String>,
    },
    FrozenDisallowed {
        reason: String,
        last_holder_id: String,
    },
}

#[derive(Clone, Debug)]
pub enum CloningPolicy {
    ClonableNonConscious,
    ClonableRedacted,
    NonClonableContainsIdentity,
}

#[derive(Clone, Debug)]
pub struct EnvironmentMetadata {
    pub id: String,
    pub environment_plane: EnvironmentPlane,
    pub awareness_flag: AwarenessFlag,
    pub consciousness_state: ConsciousnessState,
    pub oculus_flag: OculusFlag,
    pub feedback_flag: FeedbackSensorFlag,
    pub brain_token_state: BrainTokenState,
    pub cloning_policy: CloningPolicy,
    pub tags: Vec<String>,
    pub hardware_profile: Vec<String>,
    pub regulatory_labels: Vec<String>,
}

impl Display for EnvironmentPlane {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            EnvironmentPlane::Cybernetics => "cybernetics",
            EnvironmentPlane::Augmented => "augmented",
            EnvironmentPlane::Transhuman => "transhuman",
            EnvironmentPlane::Evolution => "evolution",
            EnvironmentPlane::Bioscale => "bioscale",
            EnvironmentPlane::Biophysics => "biophysics",
            EnvironmentPlane::Organic => "organic",
            EnvironmentPlane::BciHciEeg => "bci/hci/eeg",
            EnvironmentPlane::RealityOs => "reality.os",
            EnvironmentPlane::ConsciousnessNetwork => "consciousness-network",
            EnvironmentPlane::NeuralNetwork => "neural-network",
            EnvironmentPlane::Wetware => "wetware",
            EnvironmentPlane::Bioware => "bioware",
            EnvironmentPlane::SoftwareOnly => "software-only",
            EnvironmentPlane::Unknown => "unknown",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub struct PlaneClassifierConfig {
    pub prohibit_consciousness_mutation: bool,
    pub prohibit_brain_token_freeze: bool,
    pub log_all_decisions: bool,
    pub allowed_hardware_ids: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PlaneClassifier {
    pub config: PlaneClassifierConfig,
    decision_log: Vec<String>,
}

impl PlaneClassifier {
    pub fn new(config: PlaneClassifierConfig) -> Self {
        PlaneClassifier {
            config,
            decision_log: Vec::with_capacity(256),
        }
    }

    pub fn awareness_check(&mut self, involves_living_organism: bool) -> AwarenessFlag {
        let flag = if involves_living_organism {
            AwarenessFlag::LivingOrganismInvolved
        } else {
            AwarenessFlag::NoOrganism
        };
        if self.config.log_all_decisions {
            self.decision_log.push(format!(
                "[awareness-check] involves_living_organism={} -> {:?}",
                involves_living_organism, flag
            ));
        }
        flag
    }

    pub fn consciousness_state_check(
        &mut self,
        requested_state: ConsciousnessState,
    ) -> ConsciousnessState {
        match requested_state {
            ConsciousnessState::ActiveImmutable | ConsciousnessState::InactiveImmutable => {
                if self.config.prohibit_consciousness_mutation {
                    if self.config.log_all_decisions {
                        self.decision_log.push(format!(
                            "[consciousness-state] immutable-state requested {:?} -> preserved",
                            requested_state
                        ));
                    }
                    requested_state
                } else {
                    if self.config.log_all_decisions {
                        self.decision_log.push(format!(
                            "[consciousness-state] immutable-state allowed {:?}",
                            requested_state
                        ));
                    }
                    requested_state
                }
            }
            other => {
                if self.config.log_all_decisions {
                    self.decision_log.push(format!(
                        "[consciousness-state] non-immutable state {:?}",
                        other
                    ));
                }
                other
            }
        }
    }

    pub fn oculus_check(
        &mut self,
        has_oculus: bool,
        has_remote_feed: bool,
    ) -> (OculusFlag, FeedbackSensorFlag) {
        let oculus_flag = if has_oculus {
            if has_remote_feed {
                OculusFlag::OculusRemoteFeed
            } else {
                OculusFlag::OculusLocalOnly
            }
        } else {
            OculusFlag::NoOculus
        };

        let feedback_flag = if has_remote_feed {
            FeedbackSensorFlag::RemoteFeedback
        } else if has_oculus {
            FeedbackSensorFlag::LocalFeedback
        } else {
            FeedbackSensorFlag::NoFeedback
        };

        if self.config.log_all_decisions {
            self.decision_log.push(format!(
                "[oculus-check] has_oculus={}, has_remote_feed={} -> {:?}/{:?}",
                has_oculus, has_remote_feed, oculus_flag, feedback_flag
            ));
        }

        (oculus_flag, feedback_flag)
    }

    pub fn brain_token_check(
        &mut self,
        net_weight: f64,
        circulating_supply: f64,
        frozen: bool,
        controller_contract: &str,
        hardware_dependencies: Vec<String>,
    ) -> BrainTokenState {
        if frozen && self.config.prohibit_brain_token_freeze {
            let state = BrainTokenState::FrozenDisallowed {
                reason: String::from("Brain-tokens must not be frozen for any cybernetic hardware or augmented citizen."),
                last_holder_id: String::from("unknown-holder"),
            };
            if self.config.log_all_decisions {
                self.decision_log.push(format!(
                    "[brain-tokens] frozen=TRUE -> {:?}",
                    state
                ));
            }
            return state;
        }

        if net_weight <= 0.0 || circulating_supply <= 0.0 {
            if self.config.log_all_decisions {
                self.decision_log.push(String::from(
                    "[brain-tokens] no net_weight or circulating_supply -> None",
                ));
            }
            return BrainTokenState::None;
        }

        let state = BrainTokenState::Circulating {
            net_weight,
            circulating_supply,
            controller_contract: controller_contract.to_string(),
            hardware_dependencies,
        };

        if self.config.log_all_decisions {
            self.decision_log.push(format!(
                "[brain-tokens] circulating net_weight={}, circulating_supply={}, controller={}",
                net_weight, circulating_supply, controller_contract
            ));
        }

        state
    }

    pub fn cloning_policy_check(
        &mut self,
        involves_conscious_pattern: bool,
        contains_identity_descriptors: bool,
    ) -> CloningPolicy {
        let policy = if involves_conscious_pattern || contains_identity_descriptors {
            CloningPolicy::NonClonableContainsIdentity
        } else {
            CloningPolicy::ClonableNonConscious
        };

        if self.config.log_all_decisions {
            self.decision_log.push(format!(
                "[cloning-policy] conscious_pattern={}, identity_descriptors={} -> {:?}",
                involves_conscious_pattern, contains_identity_descriptors, policy
            ));
        }

        policy
    }

    pub fn classify_plane(
        &mut self,
        input_tags: &[String],
        hardware_profile: &[String],
    ) -> EnvironmentPlane {
        let mut score: HashMap<EnvironmentPlane, i32> = HashMap::new();

        fn inc(map: &mut HashMap<EnvironmentPlane, i32>, plane: EnvironmentPlane, v: i32) {
            let entry = map.entry(plane).or_insert(0);
            *entry += v;
        }

        for tag in input_tags {
            let t = tag.to_lowercase();
            if t.contains("bci") || t.contains("eeg") {
                inc(&mut score, EnvironmentPlane::BciHciEeg, 3);
                inc(&mut score, EnvironmentPlane::Biophysics, 1);
            }
            if t.contains("emg") || t.contains("ecg") {
                inc(&mut score, EnvironmentPlane::Biophysics, 2);
                inc(&mut score, EnvironmentPlane::Bioscale, 1);
            }
            if t.contains("avatar") || t.contains("ar/vr") || t.contains("oculus") {
                inc(&mut score, EnvironmentPlane::Augmented, 3);
                inc(&mut score, EnvironmentPlane::RealityOs, 1);
            }
            if t.contains("wetware") {
                inc(&mut score, EnvironmentPlane::Wetware, 4);
            }
            if t.contains("bioware") {
                inc(&mut score, EnvironmentPlane::Bioware, 4);
            }
            if t.contains("neural-net") || t.contains("ml-model") {
                inc(&mut score, EnvironmentPlane::NeuralNetwork, 3);
                inc(&mut score, EnvironmentPlane::SoftwareOnly, 1);
            }
            if t.contains("quantum") {
                inc(&mut score, EnvironmentPlane::Cybernetics, 1);
                inc(&mut score, EnvironmentPlane::NeuralNetwork, 1);
            }
            if t.contains("consciousness-net") || t.contains("soul-guard") {
                inc(&mut score, EnvironmentPlane::ConsciousnessNetwork, 4);
            }
        }

        for hw in hardware_profile {
            let h = hw.to_lowercase();
            if h.contains("openbci") || h.contains("eeg-headset") {
                inc(&mut score, EnvironmentPlane::BciHciEeg, 5);
                inc(&mut score, EnvironmentPlane::Biophysics, 2);
            }
            if h.contains("implant") || h.contains("neural-link") {
                inc(&mut score, EnvironmentPlane::Transhuman, 4);
                inc(&mut score, EnvironmentPlane::Cybernetics, 2);
            }
            if h.contains("robotic-limb") || h.contains("exoskeleton") {
                inc(&mut score, EnvironmentPlane::Cybernetics, 4);
                inc(&mut score, EnvironmentPlane::Augmented, 2);
            }
        }

        let mut best_plane = EnvironmentPlane::Unknown;
        let mut best_score = i32::MIN;
        for (plane, val) in score.iter() {
            if *val > best_score {
                best_score = *val;
                best_plane = plane.clone();
            }
        }

        if self.config.log_all_decisions {
            self.decision_log.push(format!(
                "[plane-classifier] tags={:?}, hardware={:?} -> {}",
                input_tags, hardware_profile, best_plane
            ));
        }

        best_plane
    }

    pub fn build_environment_metadata(
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
    ) -> EnvironmentMetadata {
        let awareness_flag = self.awareness_check(involves_living_organism);
        let consciousness_state = self.consciousness_state_check(requested_conscious_state);
        let (oculus_flag, feedback_flag) = self.oculus_check(has_oculus, has_remote_feed);
        let brain_token_state = self.brain_token_check(
            net_weight,
            circulating_supply,
            frozen,
            controller_contract,
            hardware_dependencies.clone(),
        );
        let cloning_policy =
            self.cloning_policy_check(involves_conscious_pattern, contains_identity_descriptors);
        let plane = self.classify_plane(&input_tags, &hardware_profile);

        EnvironmentMetadata {
            id: id.to_string(),
            environment_plane: plane,
            awareness_flag,
            consciousness_state,
            oculus_flag,
            feedback_flag,
            brain_token_state,
            cloning_policy,
            tags: input_tags,
            hardware_profile,
            regulatory_labels,
        }
    }

    pub fn decision_log(&self) -> &[String] {
        &self.decision_log
    }
}
