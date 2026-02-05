const { RIGHTS_ENVELOPE, EmergencyToken } = require("./index");

/**
 * @param {object} proposal EvolutionProposal JSON for your host RPC.
 * It MUST have a field "flags" or similar where rollback/downgrade is visible.
 */
function proposalRequestsRollbackOrDowngrade(proposal) {
  if (!proposal || typeof proposal !== "object") return false;
  const flags = proposal.flags || {};
  return Boolean(
    flags.requestRollbackAutonomy ||
      flags.requestReverseEvolutionStage ||
      flags.requestCapabilityDowngrade
  );
}

/**
 * Attach rights envelope and optional emergency token, enforcing:
 * - host-rights-travel-us-maricopa.v1 is always present
 * - reversals are only allowed when an EmergencyToken is provided.
 *
 * @param {object} identityHeader IdentityHeader JSON for RpcSecurityHeader.
 * @param {object} proposal EvolutionProposal JSON.
 * @param {EmergencyToken|null} emergencyToken
 * @returns {{securityHeader: object, payload: object}}
 */
function buildRightsAugmentedSubmit(identityHeader, proposal, emergencyToken) {
  if (proposalRequestsRollbackOrDowngrade(proposal) && !emergencyToken) {
    throw new Error(
      "Rollback/downgrade of system-autonomy or evolution stages is " +
        "forbidden without an explicit EmergencyToken from the host."
    );
  }

  const rightsAugmentedProposal = {
    proposal,
    hostId: RIGHTS_ENVELOPE.hostId,
    rightsProfileId: RIGHTS_ENVELOPE.rightsProfileId,
    emergencyToken: emergencyToken || null,
  };

  const securityHeader = {
    issuerdid: identityHeader.issuerdid,
    subjectrole: identityHeader.subjectrole,
    networktier: identityHeader.networktier,
    biophysicalchainallowed: true,
  };

  return { securityHeader, payload: rightsAugmentedProposal };
}

module.exports.buildRightsAugmentedSubmit = buildRightsAugmentedSubmit;
