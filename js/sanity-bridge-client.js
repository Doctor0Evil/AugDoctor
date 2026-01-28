// Assumes wasm_bindgen and wasm module are already loaded.

function buildSanityRequest(identity, bciEvent, requiredKnowledge, timestampUtc) {
  return JSON.stringify({
    identity,
    event: bciEvent,
    required_knowledge: requiredKnowledge,
    timestamputc: timestampUtc
  });
}

/**
 * JS helper: apply SANITY/DECAY update via WASM bridge.
 * AI-Chats must supply a valid IdentityHeader (ALN/DID/Bostrom),
 * but never touch inner ledger mechanics directly.
 */
async function applySanityDecay(identityHeader, bciEvent, requiredKnowledge, timestampUtc) {
  const payload = buildSanityRequest(identityHeader, bciEvent, requiredKnowledge, timestampUtc);
  const raw = wasm_bindgen.wasm_apply_sanity_decay(payload);
  return JSON.parse(raw);
}

module.exports = {
  applySanityDecay,
  buildSanityRequest
};
