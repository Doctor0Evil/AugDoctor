use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphFeature {
    pub session_id: String,
    pub environment_id: String,
    pub intent_label: String,
    pub channel_count: u16,
    pub fs_hz: u16,
    pub band_alpha_power: f32,
    pub band_beta_power: f32,
    pub band_gamma_power: f32,
    pub event_latency_ms: u16,
    pub classifier_confidence: f32,
    pub eco_energy_nj: f32,
    pub reward_score: f32,
    pub safety_decision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NeuromorphRoute {
    Safe,
    Defer,
    Deny,
}

pub trait NeuromorphRouter {
    fn route(&self, f: &NeuromorphFeature) -> NeuromorphRoute;
}

#[derive(Clone, Debug)]
pub struct DefaultNeuromorphRouter {
    pub min_confidence: f32,
    pub max_latency_ms: u16,
    pub max_eco_energy_nj: f32,
}

impl NeuromorphRouter for DefaultNeuromorphRouter {
    fn route(&self, f: &NeuromorphFeature) -> NeuromorphRoute {
        if f.safety_decision.starts_with("Deny") {
            return NeuromorphRoute::Deny;
        }
        if f.classifier_confidence < self.min_confidence
            || f.event_latency_ms > self.max_latency_ms
            || f.eco_energy_nj > self.max_eco_energy_nj
        {
            return NeuromorphRoute::Defer;
        }
        NeuromorphRoute::Safe
    }
}
