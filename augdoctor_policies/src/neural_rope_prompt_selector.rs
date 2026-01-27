use crate::shot_level_policy::ShotLevel;
use bioscale_upgrade_service::neural_rope::{NeuralRope, NeuralRopeSegmentSnapshot};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptExample {
    pub text: String,
    pub reward_score: f32,
    pub safety_decision: String,
    pub plane_label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptSelectionRequest {
    pub task_id: String,
    pub plane_label: String,
    pub shot_level: ShotLevel,
    pub max_examples: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptSelectionResult {
    pub task_id: String,
    pub plane_label: String,
    pub shot_level: ShotLevel,
    pub examples: Vec<PromptExample>,
}

pub struct NeuralRopePromptSelector<'a> {
    rope: &'a NeuralRope,
}

impl<'a> NeuralRopePromptSelector<'a> {
    pub fn new(rope: &'a NeuralRope) -> Self {
        NeuralRopePromptSelector { rope }
    }

    pub fn select_examples(
        &self,
        req: &PromptSelectionRequest,
    ) -> PromptSelectionResult {
        let snapshot = self.rope.export_snapshot(256);

        if req.shot_level == ShotLevel::ZeroShot || req.max_examples == 0 {
            return PromptSelectionResult {
                task_id: req.task_id.clone(),
                plane_label: req.plane_label.clone(),
                shot_level: ShotLevel::ZeroShot,
                examples: Vec::new(),
            };
        }

        let mut candidates: Vec<PromptExample> = snapshot
            .into_iter()
            .filter_map(|seg: NeuralRopeSegmentSnapshot| {
                if let Some(attrs) = seg.attributes {
                    if attrs.plane_label == req.plane_label
                        && attrs.reward_score > 0.0
                        && attrs.safety_decision.starts_with("Allow")
                    {
                        Some(PromptExample {
                            text: seg.text,
                            reward_score: attrs.reward_score,
                            safety_decision: attrs.safety_decision,
                            plane_label: attrs.plane_label,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by(|a, b| b.reward_score.partial_cmp(&a.reward_score).unwrap());

        let max = req.max_examples as usize;
        let examples = if candidates.len() > max {
            candidates.into_iter().take(max).collect()
        } else {
            candidates
        };

        PromptSelectionResult {
            task_id: req.task_id.clone(),
            plane_label: req.plane_label.clone(),
            shot_level: ShotLevel::FewShot,
            examples,
        }
    }
}
