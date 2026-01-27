/**
 * Normalize an EEGFeatureSummary JSON object to the canonical rope text.
 * This mirrors the Rust eeg_json_to_rope_text function.
 */
function eegFeatureToRopeText(f) {
  const gamma = typeof f.band_gamma_power === "number" ? f.band_gamma_power : 0.0;

  return [
    "EEG_FEATURE",
    `session=${f.session_id}`,
    `env=${f.environment_id}`,
    `intent=${f.intent_label}`,
    `chan=${f.channel_count}`,
    `fs=${f.fs_hz}Hz`,
    `alpha=${f.band_alpha_power.toFixed(4)}`,
    `beta=${f.band_beta_power.toFixed(4)}`,
    `gamma=${gamma.toFixed(4)}`,
    `csp=${f.csp_component.toFixed(4)}`,
    `erp_latency_ms=${f.erp_latency_ms}`,
    `conf=${f.classifier_confidence.toFixed(3)}`,
  ].join(" ");
}

/**
 * Minimal runtime validation compatible with the JSON schema.
 */
function validateEegFeatureSummary(obj) {
  const required = [
    "session_id",
    "environment_id",
    "intent_label",
    "channel_count",
    "fs_hz",
    "band_alpha_power",
    "band_beta_power",
    "csp_component",
    "erp_latency_ms",
    "classifier_confidence",
    "reward_score",
    "safety_decision",
  ];
  for (const key of required) {
    if (!(key in obj)) {
      throw new Error(`missing field: ${key}`);
    }
  }
  return true;
}

module.exports = {
  eegFeatureToRopeText,
  validateEegFeatureSummary,
};
