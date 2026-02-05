import { buildRightsEnvelope } from "./rightsEnvelope.js";

export async function submitEvolutionProposal({
  rpcUrl,
  evolutionProposal,
  chatContext
}) {
  const envelope = buildRightsEnvelope({
    queryId: chatContext.queryId,
    platformLabel: chatContext.platformLabel
  });

  const rpcRequest = {
    method: "SubmitEvent",
    params: {
      security: {
        issuerdid: envelope.did,
        subjectrole: "AugmentedCitizen",
        networktier: "trusted-edge",
        biophysicalchainallowed: true,
        rights_profile_id: envelope.rights_profile_id,
        mode: envelope.mode
      },
      proposal: evolutionProposal,
      chat_context: chatContext
    }
  };

  const res = await fetch(rpcUrl, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(rpcRequest)
  });

  if (!res.ok) {
    throw new Error(`AugDoctor RPC failed: ${res.status}`);
  }
  return res.json();
}
