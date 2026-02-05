const RIGHTS_ENVELOPE = Object.freeze({
  hostId: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
  rightsProfileId: "host-rights-travel-us-maricopa.v1",
});

/**
 * Emergency token must be created explicitly by you.
 */
class EmergencyToken {
  /**
   * @param {Object} payload
   * @param {string} payload.scopeId
   * @param {string} payload.transcriptHash
   * @param {string} payload.issuedAtUtc
   * @param {string} payload.signatureHex
   */
  constructor(payload) {
    this.scopeId = payload.scopeId;
    this.transcriptHash = payload.transcriptHash;
    this.issuedAtUtc = payload.issuedAtUtc;
    this.signatureHex = payload.signatureHex;
  }
}

module.exports.RIGHTS_ENVELOPE = RIGHTS_ENVELOPE;
module.exports.EmergencyToken = EmergencyToken;
