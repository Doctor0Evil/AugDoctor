export function buildRightsEnvelope({ queryId, platform }) {
  return {
    query_id: queryId,
    aichat_platform: platform,
    did: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
    rights_profile_id: "host-rights-travel-us-maricopa.v1",
    mode: "propose-only",
  };
}
