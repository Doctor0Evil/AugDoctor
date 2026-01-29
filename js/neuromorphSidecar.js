/**
 * Neuromorph telemetry sidecar for AI-Chat agents.
 * - Sends neuromorph-tagged events to the host's neuromorph-reflex endpoint.
 * - Requires host-provided ALNDID headers.
 * - Cannot see balances or force rollbacks; only receives redacted hashes + flags.
 */

/**
 * Construct a security header that mirrors your RPC doctrine.
 * The host (or a trusted local daemon) must fill this; AI-Chat tools must NOT invent it.
 */
export function buildRpcSecurityHeader({
  issuerDid,
  subjectRole = "system-daemon",      // or "augmented-citizen" for direct host clients
  networkTier = "edge",               // "edge" for AI-Chat, never "core"
  biophysicalChainAllowed = false,    // AI-Chat is never allowed to anchor chain
}) {
  return {
    issuerdid: issuerDid,
    subjectrole: subjectRole,
    networktier: networkTier,
    biophysicalchainallowed: biophysicalChainAllowed,
  };
}

/**
 * Shape a neuromorph event payload for the host.
 */
export function makeNeuromorphEvent({
  hostId,
  sessionId,
  environmentId,
  timestampMsUtc,
  kind,
  riskscore = 0.0,
}) {
  return {
    hostid: hostId,
    sessionid: sessionId,
    environmentid: environmentId,
    timestamp_ms_utc: timestampMsUtc,
    riskscore,
    // The inner Rust side maps this into NeuromorphEventKind.
    kind,
  };
}

/**
 * Send neuromorph telemetry to the host node.
 * hostUrl is the local node, e.g. http://127.0.0.1:8082/neuromorph-reflex-apply
 */
export async function sendNeuromorphReflexEvent(hostUrl, {
  securityHeader,
  neuromorphEvent,
  requiredKnowledgeFactor = 0.6,
  timestampUtc,
}) {
  const body = {
    header: securityHeader,
    event: neuromorphEvent,
    requiredknowledgefactor: requiredKnowledgeFactor,
    timestamputc: timestampUtc,
  };

  const res = await fetch(hostUrl, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });

  if (!res.ok) {
    const txt = await res.text().catch(() => "");
    throw new Error(`neuromorph-reflex-apply HTTP ${res.status}: ${txt}`);
  }

  const json = await res.json();
  // json should mirror NeuromorphReflexResult (applied, reason, plane, domain, hashes).
  return json;
}

/**
 * Example usage from an AI-Chat agent:
 *
 * const header = buildRpcSecurityHeader({
 *   issuerDid: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
 *   subjectRole: "system-daemon",
 *   networkTier: "edge",
 *   biophysicalChainAllowed: false,
 * });
 *
 * const ev = makeNeuromorphEvent({
 *   hostId: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
 *   sessionId: "session-omega",
 *   environmentId: "phx-lab",
 *   timestampMsUtc: Date.now(),
 *   riskscore: 0.1,
 *   kind: {
 *     type: "SystemOverload",
 *     error_rate: 0.35,
 *   },
 * });
 *
 * const result = await sendNeuromorphReflexEvent(
 *   "http://127.0.0.1:8082/neuromorph-reflex-apply",
 *   {
 *     securityHeader: header,
 *     neuromorphEvent: ev,
 *     requiredKnowledgeFactor: 0.6,
 *     timestampUtc: new Date().toISOString(),
 *   }
 * );
 *
 * // result.applied === true/false, but AI-Chat never sees BRAIN/WAVE balances.
 */
