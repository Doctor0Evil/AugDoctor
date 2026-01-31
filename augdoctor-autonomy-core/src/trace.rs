use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyConstraint {
    pub highest_risk_score: f32,
    pub worst_lifeforce_scalar: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyTraceAttributes {
    pub schemaversion: String,
    pub host_id: String,
    pub session_id: String,
    pub environment_id: String,
    pub plane: String,

    pub stress: f32,
    pub fatigue: f32,
    pub reward: f32,
    pub safety: f32,
    pub lifeforce_scalar: f32,
    pub eco_energy_nj: f32,
    pub risk: f32,

    pub actions_last_minute: u32,
    pub identity_drift_today: f32,

    pub decision_autonomy_level: f32,
    pub decision_shot_level_label: String,

    pub constraint: AutonomyConstraint,
}
