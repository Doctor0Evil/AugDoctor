use bioscale_upgrade_service::neural_rope::NeuralRope;

/// Minimal EEG feature summary attached to each trace (non-identity-bearing).
#[derive(Clone, Debug)]
pub struct EegFeatureSummary {
    pub channel_count: u8,
    pub fs_hz: u16,
    pub band_alpha_power: f32,
    pub band_beta_power: f32,
    pub csp_component: f32,
    pub erp_latency_ms: u16,
}

pub struct EegTraceRecorder<'a> {
    rope: &'a mut NeuralRope,
}

impl<'a> EegTraceRecorder<'a> {
    pub fn new(rope: &'a mut NeuralRope) -> Self {
        EegTraceRecorder { rope }
    }

    /// Append a successful EEG intent detection as a textual trace with attributes usable
    /// later as few-shot examples.
    pub fn record_successful_intent(
        &mut self,
        environment_id: &str,
        upgrade_id: &str,
        intent_label: &str,
        feature: &EegFeatureSummary,
        reward_score: f32,
        safety_decision: &str,
    ) {
        let trace_text = format!(
            "EEG_INTENT env={} upgrade={} intent={} alpha={:.3} beta={:.3} csp={:.3} erp_latency_ms={}",
            environment_id,
            upgrade_id,
            intent_label,
            feature.band_alpha_power,
            feature.band_beta_power,
            feature.csp_component,
            feature.erp_latency_ms
        );

        self.rope.append_trace(
            &trace_text,
            "bci/hci/eeg",
            Some(upgrade_id.to_string()),
            reward_score,
            safety_decision,
        );
    }
}
