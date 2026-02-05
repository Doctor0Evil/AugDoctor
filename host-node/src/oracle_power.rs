use serde::Serialize;
use crate::power::PowerCorridor;

/// Redacted, read-only POWER view for UIs and AI-chats.
#[derive(Clone, Debug, Serialize)]
pub struct PowerOracleView {
    pub host_id: String,
    pub remaining_documents: u32,
    pub remaining_external_calls: u32,
    pub remaining_policy_surface: u32,
    pub max_novelty: f32,
    pub irreversible_allowed: bool,
}

impl From<PowerCorridor> for PowerOracleView {
    fn from(c: PowerCorridor) -> Self {
        PowerOracleView {
            host_id: c.host_id,
            remaining_documents: c.remaining_documents,
            remaining_external_calls: c.remaining_external_calls,
            remaining_policy_surface: c.remaining_policy_surface,
            max_novelty: c.max_novelty,
            irreversible_allowed: c.irreversible_allowed,
        }
    }
}
