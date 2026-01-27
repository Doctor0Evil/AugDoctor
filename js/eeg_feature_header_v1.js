const SUPPORTED_SCHEMA = "eeg_feature_summary.v1";

/**
 * Validate header + version before sending EEGFeatureSummary to any
 * neural-rope or biophysical-chain endpoint.
 */
function validateHeaderAndVersion(payload) {
  if (payload.schema_version !== SUPPORTED_SCHEMA) {
    throw new Error(`unsupported schema_version: ${payload.schema_version}`);
  }
  const h = payload.header || {};
  if (!h.issuer_did || typeof h.issuer_did !== "string") {
    throw new Error("missing or invalid issuer_did");
  }
  if (!h.subject_role || !["augmented_citizen", "authorized_researcher", "system_daemon"].includes(h.subject_role)) {
    throw new Error("unauthorized subject_role");
  }
  if (!h.network_tier || !["core", "edge", "sandbox"].includes(h.network_tier)) {
    throw new Error("invalid network_tier");
  }
  if (h.network_tier === "sandbox" && h.biophysical_chain_allowed === true) {
    throw new Error("sandbox tier cannot set biophysical_chain_allowed=true");
  }
  if (!h.issuer_did.startsWith("bostrom") && !h.issuer_did.startsWith("did:")) {
    throw new Error("issuer_did must be ALN/DID/Bostrom namespace");
  }
  return true;
}

module.exports = {
  SUPPORTED_SCHEMA,
  validateHeaderAndVersion,
};
