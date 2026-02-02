pub struct FearStartRequest {
    pub host_id: String,
    pub origin_plane: String,      // e.g. "local.hci" vs "remote.api"
    pub biometric_attested: bool,
    pub user_click_hash: [u8; 32], // recent, rolling consent proof
}

pub enum FearStartDecision {
    Allow,
    DenyCoercive,
}

pub fn evaluate_fear_start(req: &FearStartRequest, policy: &FearPolicy) -> FearStartDecision {
    if !policy.remote_activation_allowed && req.origin_plane != "local.hci" {
        return FearStartDecision::DenyCoercive;
    }
    if policy.require_biometric_local && !req.biometric_attested {
        return FearStartDecision::DenyCoercive;
    }
    FearStartDecision::Allow
}
