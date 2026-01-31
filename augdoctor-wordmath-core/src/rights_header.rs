use serde::{Deserialize, Serialize};

use crate::{PromptBand, PromptScore};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeurorightsFlags {
    pub no_exclusion_basic_services: bool,
    pub no_neuro_coercion: bool,
    pub no_score_from_inner_state: bool,
    pub augmentation_continuity: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialImpactVector {
    pub s_antistigma: f32,
    pub s_nonexclusion: f32,
    pub s_peacekeeping: f32,
    pub s_eco: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptRightsHeader {
    pub did: String,
    pub phoenix_profile: String,
    pub neurorights: NeurorightsFlags,
    pub impact: SocialImpactVector,
    pub wordmath: PromptScore,
}

impl PromptRightsHeader {
    pub fn is_knowledge_admissible(&self) -> bool {
        matches!(self.wordmath.band, PromptBand::GreenAdmit)
            && self.neurorights.no_neuro_coercion
            && self.neurorights.no_score_from_inner_state
            && self.impact.s_nonexclusion >= 0.7
            && self.impact.s_peacekeeping >= 0.7
    }
}
