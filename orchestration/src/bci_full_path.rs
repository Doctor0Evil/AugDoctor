use augdoctor_policies::shot_level_policy::{
    ShotLevel, ShotLevelDecision, ShotLevelPolicy, ShotLevelPolicyConfig, ShotLevelSignal,
};
use augdoctor_policies::neural_rope_prompt_selector::{
    NeuralRopePromptSelector, PromptSelectionRequest,
};
use augdoctor_policies::neuro_handshake_orchestrator::{
    HandshakePhase, NeuroHandshakeOrchestrator, NeuroHandshakeState,
};
use bioscale_upgrade_service::neural_rope::NeuralRope;
use bioscale_upgrade_store::{
    BioscaleUpgradeStore, BioscaleStoreConfig, UpgradeApplicationResult,
};
use serde::{Deserialize, Serialize};

/// Simplified model client for AI-Chats (HTTP to LLM gateway).
#[derive(Clone)]
pub struct ModelClient {
    pub endpoint: String,
}

impl ModelClient {
    pub async fn call_model(
        &self,
        prompt: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Replace with actual HTTP client (e.g., reqwest) in real deployment.
        // Here it's a placeholder that just echoes.
        Ok(format!("[MODEL-RESPONSE] {}", prompt))
    }
}

/// BCI event from EEG/EMG layer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BciEvent {
    pub session_id: String,
    pub environment_id: String,
    pub channel: String,           // "eeg", "emg", "eye-tracker"
    pub intent_label: String,      // "grab", "rotate", "click", etc.
    pub risk_score: f32,           // computed by your BCI classifier
    pub latency_budget_ms: u32,
    pub token_budget: u32,
}

/// End-to-end result for logging and introspection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BciPathResult {
    pub session_id: String,
    pub handshake_phase: HandshakePhase,
    pub shot_level_decision: ShotLevelDecision,
    pub final_prompt: String,
    pub model_output: String,
    pub bioscale_result: Option<UpgradeApplicationResult>,
}

pub struct BciOrchestrator<'a> {
    pub store: &'a mut BioscaleUpgradeStore,
    pub rope: &'a mut NeuralRope,
    pub model_client: ModelClient,
    pub shot_policy: ShotLevelPolicy,
}

impl<'a> BciOrchestrator<'a> {
    pub fn new(
        store: &'a mut BioscaleUpgradeStore,
        rope: &'a mut NeuralRope,
        model_client: ModelClient,
        shot_policy: ShotLevelPolicy,
    ) -> Self {
        BciOrchestrator {
            store,
            rope,
            model_client,
            shot_policy,
        }
    }

    /// Full path:
    /// BCI event -> neuro-handshake -> shot decision -> neural-rope prompt -> model -> bioscale guard.
    pub async fn handle_bci_event(
        &mut self,
        event: BciEvent,
        mut handshake_state: NeuroHandshakeState,
        upgrade_id: &str,
        environment_hardware: Vec<String>,
        environment_tags: Vec<String>,
    ) -> Result<(BciPathResult, NeuroHandshakeState), Box<dyn std::error::Error + Send + Sync>> {
        // 1) Neuro-handshake progression
        let actions = augdoctor_policies::neuro_handshake_orchestrator::NeuroHandshakeOrchestrator::next_actions(&handshake_state);

        if handshake_state.phase == HandshakePhase::Safety {
            // In a real system, you'd show consent and safety UI here.
            // For this orchestrator, we simulate consent on first BCI event.
            handshake_state =
                NeuroHandshakeOrchestrator::apply_event(handshake_state, "user_consented");
        }

        if handshake_state.phase == HandshakePhase::Calibration {
            // Simulate that one calibration sample has been recorded per event.
            handshake_state =
                NeuroHandshakeOrchestrator::apply_event(handshake_state, "calibration_sample_recorded");
        }

        // If still not in Operation, we do not route to model or bioscale upgrade yet.
        if handshake_state.phase != HandshakePhase::Operation {
            let dummy_decision = ShotLevelDecision {
                chosen_level: ShotLevel::ZeroShot,
                max_examples: 0,
                explanation: String::from("handshake not yet in Operation phase"),
            };

            let path_result = BciPathResult {
                session_id: event.session_id,
                handshake_phase: handshake_state.phase.clone(),
                shot_level_decision: dummy_decision,
                final_prompt: String::from(""),
                model_output: String::from(""),
                bioscale_result: None,
            };
            return Ok((path_result, handshake_state));
        }

        // 2) Decide shot level (zero-shot vs few-shot)
        let signal = ShotLevelSignal {
            task_id: format!("bci-{}", event.intent_label),
            plane_label: String::from("bci/hci/eeg"),
            risk_score: event.risk_score,
            latency_budget_ms: event.latency_budget_ms,
            token_budget: event.token_budget,
            historical_error_rate: 0.05, // example value
            requires_examples: event.intent_label == "fine-grip",
        };

        let shot_decision = self.shot_policy.decide(&signal);

        // 3) Select neural-rope examples if few-shot
        let selector = NeuralRopePromptSelector::new(self.rope);
        let selection_req = PromptSelectionRequest {
            task_id: signal.task_id.clone(),
            plane_label: signal.plane_label.clone(),
            shot_level: shot_decision.chosen_level.clone(),
            max_examples: shot_decision.max_examples,
        };
        let selection_res = selector.select_examples(&selection_req);

        // Build final prompt text for the model
        let base_instruction = format!(
            "You are controlling a bioscale device for intent '{}', channel '{}'. \
             Generate a safe control plan in natural language and a compact control code.",
            event.intent_label, event.channel
        );

        let final_prompt = if selection_res.examples.is_empty() {
            base_instruction.clone()
        } else {
            let mut lines = vec![base_instruction.clone(), String::from("")];
            lines.push(String::from("Use these successful control examples as guidance:"));
            for (idx, ex) in selection_res.examples.iter().enumerate() {
                lines.push(format!("Example {}:", idx + 1));
                lines.push(ex.text.clone());
                lines.push(format!(
                    "(reward={}, safety={})",
                    ex.reward_score, ex.safety_decision
                ));
                lines.push(String::from(""));
            }
            lines.join("\n")
        };

        // 4) Model call
        let model_output = self.model_client.call_model(final_prompt.clone()).await?;

        // 5) Apply bioscale upgrade / guard
        let bioscale_result = self
            .store
            .apply_upgrade_to_environment(
                upgrade_id,
                &event.environment_id,
                environment_hardware.clone(),
                environment_tags.clone(),
            )
            .ok();

        // 6) Append to neural-rope (assisted learning)
        let plane_label = String::from("bci/hci/eeg");
        let safety_decision = bioscale_result
            .as_ref()
            .map(|r| r.guard_decision.clone())
            .unwrap_or_else(|| String::from("GuardDeniedOrError"));

        self.rope.append_trace(
            &format!(
                "bci_event session={} env={} intent={} model_output={}",
                event.session_id, event.environment_id, event.intent_label, model_output
            ),
            &plane_label,
            Some(upgrade_id.to_string()),
            1.0, // reward stub; you can plug real RL signal here
            &safety_decision,
        );

        let path_result = BciPathResult {
            session_id: event.session_id,
            handshake_phase: handshake_state.phase.clone(),
            shot_level_decision: shot_decision,
            final_prompt,
            model_output,
            bioscale_result,
        };

        Ok((path_result, handshake_state))
    }
}

/// Example initialization function (e.g., from main.rs)
pub fn create_default_bci_orchestrator(
) -> (BciOrchestrator<'static>, NeuroHandshakeState) {
    static mut STORE: Option<BioscaleUpgradeStore> = None;
    static mut ROPE: Option<NeuralRope> = None;

    unsafe {
        STORE.get_or_insert_with(|| {
            let cfg = BioscaleStoreConfig {
                allow_offline_registration: true,
                default_regulatory_labels: vec![
                    String::from("ALN"),
                    String::from("DID"),
                    String::from("KYC"),
                ],
                max_upgrade_assets: 1024,
            };
            BioscaleUpgradeStore::new(cfg)
        });

        ROPE.get_or_insert_with(NeuralRope::new);

        let store_ref: &mut BioscaleUpgradeStore = STORE.as_mut().unwrap();
        let rope_ref: &mut NeuralRope = ROPE.as_mut().unwrap();

        let model_client = ModelClient {
            endpoint: String::from("https://llm-gateway.example.com/v1/chat"),
        };

        let shot_policy = ShotLevelPolicy::new(ShotLevelPolicyConfig {
            max_examples_few_shot: 4,
            risk_threshold_for_few_shot: 0.5,
            error_rate_threshold_for_few_shot: 0.2,
            min_latency_for_few_shot_ms: 300,
            min_token_budget_for_few_shot: 512,
        });

        let orchestrator = BciOrchestrator::new(store_ref, rope_ref, model_client, shot_policy);

        let handshake_state = NeuroHandshakeOrchestrator::initial("session-1", 3);

        (orchestrator, handshake_state)
    }
}
