use crate::types::{EvolutionProposalKind, EvolutionProposal};
use crate::consent::{EmergencyOverride, verify_emergency_signature};
use crate::lifeforce::LifeforceError;

/// Return Err if proposal tries to reduce autonomy or evolution stage
/// without a valid, host-signed emergency override.
pub fn enforce_monotone_evolution(
    proposal: &EvolutionProposal,
    emergency: Option<&EmergencyOverride>,
) -> Result<(), LifeforceError> {
    if !proposal.requests_rollback_or_downgrade() {
        return Ok(());
    }

    let token = match emergency {
        Some(t) => t,
        None => {
            return Err(LifeforceError::Forbidden(
                "rollback/downgrade of system-autonomy or evolution stages \
                 is forbidden without an explicit EmergencyOverride from the host"
                    .into(),
            ));
        }
    };

    // Scope must match an evolution-emergency rollback profile you define in ALN.[file:39]
    if token.scope_id != "evolution-emergency-rollback-v1" {
        return Err(LifeforceError::Forbidden(
            "emergency override has wrong scope for evolution rollback".into(),
        ));
    }

    // Verify signature under your Bostrom DID key using your existing DID/consent stack.[file:47]
    verify_emergency_signature(token)
        .map_err(|e| LifeforceError::Forbidden(format!("invalid emergency signature: {e}")))?;

    Ok(())
}
