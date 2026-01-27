use bioscale_upgrade_service::neural_rope::NeuralRope;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct EegHeaderV1 {
    pub issuer_did: String,
    pub subject_role: String, // "augmented_citizen" | "authorized_researcher" | "system_daemon"
    pub biophysical_chain_allowed: bool,
    pub network_tier: String, // "core" | "edge" | "sandbox"
}

#[derive(Clone, Debug, Deserialize)]
pub struct EegFeatureSummaryV1 {
    pub schema_version: String,
    pub header: EegHeaderV1,

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

/// Hard policy: only augmented-citizen and authorized identities on trusted tiers
/// may feed the biophysical chain and neural-rope for control.
fn validate_header_security(header: &EegHeaderV1) -> Result<(), String> {
    if header.subject_role != "augmented_citizen"
        && header.subject_role != "authorized_researcher"
        && header.subject_role != "system_daemon"
    {
        return Err("unauthorized subject_role".to_string());
    }

    if header.network_tier == "sandbox" && header.biophysical_chain_allowed {
        return Err("sandbox tier cannot anchor to biophysical-chain".to_string());
    }

    if !header.issuer_did.starts_with("bostrom") && !header.issuer_did.starts_with("did:") {
        return Err("issuer_did not in ALN/DID/Bostrom namespace".to_string());
    }

    Ok(())
}

pub fn eeg_json_to_rope_text_v1(f: &EegFeatureSummaryV1) -> String {
    let gamma = f.band_gamma_power.unwrap_or(0.0);
    format!(
        "EEG_FEATURE v={} issuer={} role={} tier={} session={} env={} intent={} chan={} fs={}Hz alpha={:.4} beta={:.4} gamma={:.4} csp={:.4} erp_latency_ms={} conf={:.3}",
        f.schema_version,
        f.header.issuer_did,
        f.header.subject_role,
        f.header.network_tier,
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

/// Append only if schema_version and header security pass.
pub fn append_eeg_json_v1_to_neural_rope(
    rope: &mut NeuralRope,
    json: &str,
) -> Result<(), String> {
    let parsed: EegFeatureSummaryV1 =
        serde_json::from_str(json).map_err(|e| format!("invalid_json: {}", e))?;

    if parsed.schema_version != "eeg_feature_summary.v1" {
        return Err("unsupported schema_version".to_string());
    }

    validate_header_security(&parsed.header)?;

    let text = eeg_json_to_rope_text_v1(&parsed);

    rope.append_trace(
        &text,
        "bci/hci/eeg",
        None,
        parsed.reward_score,
        &parsed.safety_decision,
    );

    Ok(())
}
