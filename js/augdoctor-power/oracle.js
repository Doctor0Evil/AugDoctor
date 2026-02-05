class PowerOracleView {
  constructor(payload) {
    this.hostId = payload.host_id;
    this.remainingDocuments = payload.remaining_documents;
    this.remainingExternalCalls = payload.remaining_external_calls;
    this.remainingPolicySurface = payload.remaining_policy_surface;
    this.maxNovelty = payload.max_novelty;
    this.irreversibleAllowed = payload.irreversible_allowed;
  }

  canProposeIrreversible() {
    return this.irreversibleAllowed;
  }

  hasBudgetFor(stepKind) {
    switch (stepKind) {
      case "document":
        return this.remainingDocuments > 0;
      case "external":
        return this.remainingExternalCalls > 0;
      case "policy":
        return this.remainingPolicySurface > 0;
      default:
        return false;
    }
  }
}

async function fetchPowerOracle(baseUrl, hostId, authToken) {
  const res = await fetch(`${baseUrl}/oracle/power?host=${encodeURIComponent(hostId)}`, {
    method: "GET",
    headers: {
      "Accept": "application/json",
      "Authorization": `Bearer ${authToken}`,
    },
  });
  if (!res.ok) {
    throw new Error(`POWER oracle HTTP ${res.status}`);
  }
  const json = await res.json();
  return new PowerOracleView(json);
}

module.exports = {
  PowerOracleView,
  fetchPowerOracle,
};
