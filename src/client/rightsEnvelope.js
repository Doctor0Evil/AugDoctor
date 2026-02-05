const HOST_DID = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
const RIGHTS_PROFILE_ID = "host-rights-travel-us-maricopa.v1";

export function buildRightsEnvelope({ queryId, platformLabel }) {
  return {
    query_id: queryId,
    aichat_platform: platformLabel,
    did: HOST_DID,
    rights_profile_id: RIGHTS_PROFILE_ID,
    mode: "propose-only"
  };
}
