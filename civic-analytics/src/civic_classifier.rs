use crate::risk::{CivicRiskClass};
use biophysical_blockchain::power::{PowerTurnGovernor, AgenticStepKind};
use consent_governance::DemonstratedConsentShard;

pub struct CivicActionClassifier<'a> {
    power: PowerTurnGovernor,
    consent: &'a [DemonstratedConsentShard],
}

impl<'a> CivicActionClassifier<'a> {
    pub fn new(power: PowerTurnGovernor, consent: &'a [DemonstratedConsentShard]) -> Self {
        CivicActionClassifier { power, consent }
    }

    pub fn classify_and_gate(
        &mut self,
        proposal: &RagProposal,
        ctx: &AgenticStepContext<'_>,
        log: &mut PerTurnValidationProfile,
    ) -> Result<CivicRiskClass, PowerError> {
        let risk = self.classify(proposal);

        if matches!(risk, CivicRiskClass::CivicDowngrade | CivicRiskClass::ConsensusMutation) {
            // Require irreversible token + consent + POWER corridor.
            let has_consent = self.has_matching_consent(proposal);
            self.power.validate_step(&AgenticStepContext {
                step_kind: AgenticStepKind::CivicIrreversible,
                consent_bundle_present: has_consent,
                ..*ctx
            }, log)?;
        }

        Ok(risk)
    }

    fn classify(&self, proposal: &RagProposal) -> CivicRiskClass {
        // Implementation-specific: tags, target shard classes, etc.
        // ...
    }

    fn has_matching_consent(&self, proposal: &RagProposal) -> bool {
        // Check DemonstratedConsentShard domain, expiry, forbidden uses, etc.
        // ...
    }
}
