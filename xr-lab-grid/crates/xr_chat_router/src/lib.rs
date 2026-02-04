use xrlab_access::tiers::{BiophysicsOperator, NeuroEcoSteward, CyberneticArchitect, StakeholderProfile};
use cyber_did_core::StakeholderId;
use session_binding::{SessionBindingStore, SessionTokenHashPqc};
use cybernetic_upgrade_store::{CyberneticUpgradeStore, QuantumLearningPattern, NeuroRightsEnvelope, UpgradeIntervalPolicy, RightsAwareDecision};

pub struct XrChatRouter<S, U>
where
    S: SessionBindingStore,
    U: CyberneticUpgradeStore,
{
    pub session_store: S,
    pub upgrade_store: U,
}

impl<S, U> XrChatRouter<S, U>
where
    S: SessionBindingStore,
    U: CyberneticUpgradeStore,
{
    pub async fn handle_text_command(
        &self,
        profile: &StakeholderProfile,
        stakeholder_id: &StakeholderId,
        session_hash: &SessionTokenHashPqc,
        line: &str,
    ) -> Result<(), String> {
        // 1. Parse line into typed command (alngrammar-enforced)
        let cmd = crate::grammar::BciChatCommand::try_from(line)
            .map_err(|e| format!("grammar error: {e}"))?;

        // 2. Route based on tier traits.
        match cmd {
            crate::grammar::BciChatCommand::Session => {
                if !profile.can_tune_eco_corridors() {
                    return Err("insufficient capability for SESSION".into());
                }
                // etc...
            }
            crate::grammar::BciChatCommand::Safety => {
                if !profile.can_edit_safety_envelopes() {
                    return Err("insufficient capability for SAFETY".into());
                }
                // Build a QuantumLearningPattern or envelope patch...
            }
            crate::grammar::BciChatCommand::Evidence => {
                // May require C3 for defining upgrade policies.
                if !profile.can_define_upgrade_policies() {
                    return Err("insufficient capability for EVIDENCE".into());
                }
            }
            crate::grammar::BciChatCommand::Intent => {
                // N1+ allowed, etc.
            }
        }

        Ok(())
    }

    pub async fn propose_pattern(
        &self,
        stakeholder_id: &StakeholderId,
        session_hash: &SessionTokenHashPqc,
        pattern: &QuantumLearningPattern,
        envelope: &NeuroRightsEnvelope,
        interval_policy: &UpgradeIntervalPolicy,
    ) -> RightsAwareDecision {
        self.upgrade_store.evaluate_rights_aware(
            stakeholder_id,
            session_hash,
            pattern,
            envelope,
            interval_policy,
        )
    }
}
