use bioscale_upgrade_service::neural_rope::NeuralRope;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct EegFeatureSummaryJson {
    pub session_id: String,
    pub environment_id: String,
    pub intent_label: String,
    pub channel_count: u8,
    pub fs_hz: u16,
    pub band_alpha_power: f32,
    pub band_beta_power: f32,
    pub band_gamma_power: Option<f32>,
    pub csp_component: f32,
    pub erp_latency_ms: u16,
    pub classifier_confidence: f32,
    pub reward_score: f32,
    pub safety_decision: String,
}

/// Convert a JSON EEGFeatureSummary into the canonical text token
/// used inside NeuralRope.
pub fn eeg_json_to_rope_text(f: &EegFeatureSummaryJson) -> String {
    let gamma = f.band_gamma_power.unwrap_or(0.0);
    format!(
        "EEG_FEATURE session={} env={} intent={} chan={} fs={}Hz alpha={:.4} beta={:.4} gamma={:.4} csp={:.4} erp_latency_ms={} conf={:.3}",
        f.session_id,
        f.environment_id,
        f.intent_label,
        f.channel_count,
        f.fs_hz,
        f.band_alpha_power,
        f.band_beta_power,
        gamma,
        f.csp_component,
        f.erp_latency_ms,
        f.classifier_confidence
    )
}

/// Ingest a JSON string, append to neural-rope with attributes.
pub fn append_eeg_json_to_neural_rope(
    rope: &mut NeuralRope,
    json: &str,
) -> Result<(), String> {
    let parsed: EegFeatureSummaryJson =
        serde_json::from_str(json).map_err(|e| format!("invalid_json: {}", e))?;

    let text = eeg_json_to_rope_text(&parsed);

    rope.append_trace(
        &text,
        "bci/hci/eeg",
        None,
        parsed.reward_score,
        &parsed.safety_decision,
    );

    Ok(())
}
