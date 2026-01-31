use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNodeEnvelope {
    pub max_node_duty: f64,
    pub max_traffic_rate: f64,
    pub max_blind_window: f64,
    pub eco_score_min: f64,
}

impl SwarmNodeEnvelope {
    pub fn default_for_city() -> Self {
        Self {
            max_node_duty: 0.7,
            max_traffic_rate: 1000.0,
            max_blind_window: 5.0,
            eco_score_min: 0.6,
        }
    }
}
