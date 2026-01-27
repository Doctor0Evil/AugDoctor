use augdoctor_policies::shot_level_policy::{ShotLevel, ShotLevelDecision, ShotLevelSignal};
use augdoctor_policies::neural_rope_prompt_selector::{
    NeuralRopePromptSelector, PromptSelectionRequest,
};
use bioscale_upgrade_service::neural_rope::NeuralRope;

/// Decide shot level and build a prompt from EEG traces for a target intent.
pub fn build_eeg_few_shot_prompt(
    rope: &NeuralRope,
    intent_label: &str,
    risk_score: f32,
    latency_budget_ms: u32,
    token_budget: u32,
) -> (ShotLevelDecision, String) {
    let policy = augdoctor_policies::shot_level_policy::ShotLevelPolicy::new(
        augdoctor_policies::shot_level_policy::ShotLevelPolicyConfig {
            max_examples_few_shot: 4,
            risk_threshold_for_few_shot: 0.4,
            error_rate_threshold_for_few_shot: 0.15,
            min_latency_for_few_shot_ms: 250,
            min_token_budget_for_few_shot: 512,
        },
    );

    let signal = ShotLevelSignal {
        task_id: format!("eeg-{}", intent_label),
        plane_label: String::from("bci/hci/eeg"),
        risk_score,
        latency_budget_ms,
        token_budget,
        historical_error_rate: 0.05,
        requires_examples: intent_label == "fine_grip",
    };

    let decision = policy.decide(&signal);

    let selector = NeuralRopePromptSelector::new(rope);
    let selection_req = PromptSelectionRequest {
        task_id: signal.task_id.clone(),
        plane_label: signal.plane_label.clone(),
        shot_level: decision.chosen_level.clone(),
        max_examples: decision.max_examples,
    };
    let selection_res = selector.select_examples(&selection_req);

    let base_instruction = format!(
        "You are an EEG-driven control assistant. \
         Given motor imagery features, map the user's intent '{}' into safe, \
         discrete control actions for a robotic gripper. \
         Use low force and minimize abrupt motion.",
        intent_label
    );

    let final_prompt = if selection_res.examples.is_empty() {
        base_instruction.clone()
    } else {
        let mut lines = vec![base_instruction.clone(), String::from("")];
        lines.push(String::from(
            "Use these successful EEG-to-action mappings as examples:",
        ));
        for (idx, ex) in selection_res.examples.iter().enumerate() {
            lines.push(format!("Example {}:", idx + 1));
            lines.push(ex.text.clone());
            lines.push(format!(
                "(reward={}, safety={})",
                ex.reward_score, ex.safety_decision
            ));
            lines.push(String::from(""));
        }
        lines.push(String::from(
            "Now infer the next control action and explain the reasoning briefly.",
        ));
        lines.join("\n")
    };

    (decision, final_prompt)
}
