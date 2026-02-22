use sovereign_id::NeuroSubjectId;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicrospaceLivenessProfile {
    pub subject_id: NeuroSubjectId,
    pub microspace_id: String,
    pub min_probes_per_window: u32,
    pub max_offline_gap: Duration,
    pub continuity_mode: ContinuityMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContinuityMode {
    Bounded,
    Paused,
    Revoked,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LivenessWindow {
    pub microspace_id: String,
    pub last_probe_ts: u64, // unix seconds
    pub probe_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LivenessDecision {
    Allowed,
    LogOnly(String),
    Blocked(String),
}

pub fn check_swarm_liveness(
    profile: &MicrospaceLivenessProfile,
    window: &LivenessWindow,
    now_ts: u64,
) -> LivenessDecision {
    if profile.continuity_mode == ContinuityMode::Revoked {
        return LivenessDecision::Blocked("continuity revoked".into());
    }
    if now_ts.saturating_sub(window.last_probe_ts) > profile.max_offline_gap.as_secs() {
        return LivenessDecision::Blocked("offline gap exceeded".into());
    }
    if window.probe_count < profile.min_probes_per_window {
        return LivenessDecision::LogOnly("insufficient liveness probes".into());
    }
    LivenessDecision::Allowed
}
