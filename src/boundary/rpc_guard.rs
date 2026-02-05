use crate::types::IdentityHeader;
use crate::governance::host_rights_travel_us::HostRightsTravelUsProfile;
use crate::access::validate_identity_for_inner_ledger;

#[derive(Clone, Debug)]
pub struct RpcSecurityContext {
    pub id_header: IdentityHeader,
    pub rights_travel_us: HostRightsTravelUsProfile,
}

pub fn guard_rpc_for_host_rights(
    ctx: &RpcSecurityContext,
    required_k: f32,
) -> Result<(), String> {
    // Enforce standard ALN/Bostrom DID + role gating.
    validate_identity_for_inner_ledger(&ctx.id_header, required_k)
        .map_err(|e| format!("identity validation failed: {:?}", e))?;

    // Enforce propose-only semantics for AI-chats and external authorities.
    let issuer = ctx.id_header.issuerdid.as_str();
    let role = &ctx.id_header.subjectrole;

    let is_ai_chat = issuer.contains("perplexity")
        || issuer.contains("gemini")
        || issuer.contains("copilot")
        || issuer.contains("xai")
        || issuer.contains("vondy");

    let is_external_authority = issuer.contains("court")
        || issuer.contains("maricopa")
        || issuer.contains("us-federal")
        || issuer.contains("hospital")
        || issuer.contains("law-enforcement");

    if (is_ai_chat || is_external_authority)
        && ctx.rights_travel_us.ai_platforms_may_execute
    {
        // Invariant should already forbid this, but we double-guard.
        return Err("execution from AI-chats / external authorities is forbidden; propose-only".into());
    }

    // You can add domain-specific checks here to ensure that any request
    // coming from these issuers is tagged as a proposal, not a mutation.

    Ok(())
}
