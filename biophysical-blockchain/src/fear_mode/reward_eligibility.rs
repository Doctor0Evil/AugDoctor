#[derive(Clone, Debug)]
pub struct FearSessionSummary {
    pub host_id: String,
    pub session_id: String,
    pub plane: String,            // e.g. "fear-mode.softwareonly"
    pub started_at_utc: i64,
    pub ended_at_utc: i64,
    pub knowledge_factor_delta: f32, // clarified, compliant bits/energy unit
    pub kernel_passed: bool,
    pub roh_invariant_passed: bool,
    pub cyberrank_invariant_passed: bool,
}

#[derive(Clone, Debug)]
pub struct RewardDecision {
    pub session_id: String,
    pub eligible: bool,
    pub reason_codes: Vec<String>,  // e.g. ["KERNEL_VIOLATION", "ROH_SPIKE"]
}

pub fn evaluate_fear_session(summary: &FearSessionSummary) -> RewardDecision {
    let mut reasons = Vec::new();

    if !summary.kernel_passed {
        reasons.push("KERNEL_VIOLATION".to_string());
    }
    if !summary.roh_invariant_passed {
        reasons.push("ROH_INVARIANT_BROKEN".to_string());
    }
    if !summary.cyberrank_invariant_passed {
        reasons.push("CYBERRANK_DEGRADED".to_string());
    }
    if summary.knowledge_factor_delta <= 0.0 {
        reasons.push("NO_KNOWLEDGE_GAIN".to_string());
    }

    RewardDecision {
        session_id: summary.session_id.clone(),
        eligible: reasons.is_empty(),
        reason_codes: reasons,
    }
}
