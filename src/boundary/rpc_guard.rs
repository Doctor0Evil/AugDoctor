use crate::types::IdentityHeader;
use crate::access::validate_identity_for_inner_ledger;
use crate::governance::host_rights_travel_us::HostRightsTravelUsProfile;

#[derive(Clone, Debug)]
pub struct RpcSecurityContext {
    pub id_header: IdentityHeader,
    pub rights_travel_us: HostRightsTravelUsProfile,
}

pub fn guard_rpc_for_augdoctor(
    ctx: &RpcSecurityContext,
    required_k: f32,
) -> Result<(), String> {
    // Standard DID/role/tier checks.
    validate_identity_for_inner_ledger(&ctx.id_header, required_k)
        .map_err(|e| format!("identity validation failed: {:?}", e))?;

    let issuer = ctx.id_header.issuerdid.as_str();

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

    // Hard rule: these actors are *always* propose-only.
    if is_ai_chat || is_external_authority {
        return Err(
            "execution from AI-chats or external authorities is forbidden (propose-only)".into(),
        );
    }

    // Local processes (your own SystemDaemon, etc.) continue past this point.
    Ok(())
}
