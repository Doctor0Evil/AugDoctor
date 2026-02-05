use serde::{Deserialize, Serialize};
use crate::rpc_types::{RpcSecurityHeader, RpcResponse};
use organichain_runtime::{EvolutionProposal, EmergencyOverride, submit_evolution};

#[derive(Debug, Deserialize)]
pub struct RightsAugmentedProposal {
    pub proposal: EvolutionProposal,
    pub host_id: String,
    pub rights_profile_id: String,
    pub emergency_token: Option<EmergencyToken>,
}

#[derive(Debug, Deserialize)]
pub struct EmergencyToken {
    pub scope_id: String,
    pub transcript_hash: String,
    pub issued_at_utc: String,
    pub signature_hex: String,
}

pub fn handle_submit_evolution_with_rights(
    sec: RpcSecurityHeader,
    wrapped: RightsAugmentedProposal,
) -> RpcResponse {
    // Normal RpcSecurityHeader role/tier checks already run elsewhere.[file:47]

    if wrapped.host_id != "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7" {
        return RpcResponse::error("invalid host_id for rights-enforced evolution");
    }
    if wrapped.rights_profile_id != "host-rights-travel-us-maricopa.v1" {
        return RpcResponse::error("missing or wrong rights_profile_id");
    }

    let emergency = wrapped.emergency_token.map(|t| EmergencyOverride {
        scope_id: t.scope_id,
        transcript_hash: t.transcript_hash,
        issued_at_utc: t.issued_at_utc,
        signature_hex: t.signature_hex,
    });

    submit_evolution(sec.identity_header, wrapped.proposal, emergency)
}
