use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShotLevel {
    ZeroShot,
    FewShot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShotLevelSignal {
    pub task_id: String,
    pub plane_label: String,
    pub risk_score: f32,
    pub latency_budget_ms: u32,
    pub token_budget: u32,
    pub historical_error_rate: f32,
    pub requires_examples: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShotLevelDecision {
    pub chosen_level: ShotLevel,
    pub max_examples: u8,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShotLevelPolicyConfig {
    pub max_examples_few_shot: u8,
    pub risk_threshold_for_few_shot: f32,
    pub error_rate_threshold_for_few_shot: f32,
    pub min_latency_for_few_shot_ms: u32,
    pub min_token_budget_for_few_shot: u32,
}

#[derive(Clone, Debug)]
pub struct ShotLevelPolicy {
    cfg: ShotLevelPolicyConfig,
}

impl ShotLevelPolicy {
    pub fn new(cfg: ShotLevelPolicyConfig) -> Self {
        ShotLevelPolicy { cfg }
    }

    pub fn decide(&self, signal: &ShotLevelSignal) -> ShotLevelDecision {
        let mut use_few_shot = false;
        let mut reasons: Vec<String> = Vec::new();

        if signal.requires_examples {
            use_few_shot = true;
            reasons.push(String::from("task explicitly requires examples"));
        }
        if signal.risk_score >= self.cfg.risk_threshold_for_few_shot {
            use_few_shot = true;
            reasons.push(format!(
                "risk_score {:.3} >= threshold {:.3}",
                signal.risk_score, self.cfg.risk_threshold_for_few_shot
            ));
        }
        if signal.historical_error_rate >= self.cfg.error_rate_threshold_for_few_shot {
            use_few_shot = true;
            reasons.push(format!(
                "historical_error_rate {:.3} >= threshold {:.3}",
                signal.historical_error_rate, self.cfg.error_rate_threshold_for_few_shot
            ));
        }
        if signal.latency_budget_ms < self.cfg.min_latency_for_few_shot_ms {
            use_few_shot = false;
            reasons.push(format!(
                "latency_budget_ms {} < min_latency_for_few_shot_ms {}",
                signal.latency_budget_ms, self.cfg.min_latency_for_few_shot_ms
            ));
        }
        if signal.token_budget < self.cfg.min_token_budget_for_few_shot {
            use_few_shot = false;
            reasons.push(format!(
                "token_budget {} < min_token_budget_for_few_shot {}",
                signal.token_budget, self.cfg.min_token_budget_for_few_shot
            ));
        }

        let chosen_level = if use_few_shot {
            ShotLevel::FewShot
        } else {
            ShotLevel::ZeroShot
        };

        let max_examples = match chosen_level {
            ShotLevel::ZeroShot => 0,
            ShotLevel::FewShot => self.cfg.max_examples_few_shot,
        };

        ShotLevelDecision {
            chosen_level,
            max_examples,
            explanation: reasons.join("; "),
        }
    }
}
