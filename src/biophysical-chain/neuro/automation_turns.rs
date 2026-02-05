use crate::power::{PowerCorridor, PowerError};
use crate::types::{HostEnvelope, LifeforceBand, EcoBandProfile};
use crate::governance::perturnvalidationprofilev1::PerTurnValidationProfile;

#[derive(Clone, Debug)]
pub enum AgenticStepKind {
    ReadDocument,
    ExternalCall,
    PolicyTouch,
    CivicIrreversible,
}

#[derive(Clone, Debug)]
pub struct AgenticStepContext<'a> {
    pub host: &'a HostEnvelope,
    pub lifeforce_band: LifeforceBand,
    pub eco_profile: &'a EcoBandProfile,
    pub irreversible_token_present: bool,
    pub consent_bundle_present: bool,
    pub step_kind: AgenticStepKind,
    pub novelty_score_01: f32,
}

pub struct PowerTurnGovernor {
    corridor: PowerCorridor,
}

impl PowerTurnGovernor {
    pub fn new(host: &HostEnvelope, lf: LifeforceBand, eco: &EcoBandProfile) -> Self {
        let corridor = PowerCorridor::from_state(host, lf, eco);
        PowerTurnGovernor { corridor }
    }

    /// Called by the central 10-action validator before each agentic step.
    pub fn validate_step(
        &mut self,
        ctx: &AgenticStepContext<'_>,
        log: &mut PerTurnValidationProfile,
    ) -> Result<(), PowerError> {
        // Novelty corridor: reject excessive novelty outright.
        if ctx.novelty_score_01 > self.corridor.max_novelty {
            log.log_power_denial(
                "novelty",
                ctx.novelty_score_01,
                self.corridor.max_novelty,
                "POWER novelty corridor exceeded",
                "0xPWR01",
            );
            return Err(PowerError::PolicySurfaceBudgetExhausted);
        }

        // Charge counters by step kind.
        match ctx.step_kind {
            AgenticStepKind::ReadDocument => {
                if let Err(e) = self.corridor.charge_document() {
                    log.log_power_denial(
                        "documents",
                        self.corridor.max_documents as f32,
                        0.0,
                        &e.to_string(),
                        "0xPWR01",
                    );
                    return Err(e);
                }
            }
            AgenticStepKind::ExternalCall => {
                if let Err(e) = self.corridor.charge_external_call() {
                    log.log_power_denial(
                        "external_calls",
                        self.corridor.max_external_calls as f32,
                        0.0,
                        &e.to_string(),
                        "0xPWR01",
                    );
                    return Err(e);
                }
            }
            AgenticStepKind::PolicyTouch => {
                if let Err(e) = self.corridor.charge_policy_touch() {
                    log.log_power_denial(
                        "policy_surface",
                        self.corridor.max_policy_surface as f32,
                        0.0,
                        &e.to_string(),
                        "0xPWR01",
                    );
                    return Err(e);
                }
            }
            AgenticStepKind::CivicIrreversible => {
                // POWER can only *tighten*: it never skips consent or consensus checks.
                if let Err(e) = self.corridor.require_irreversible_capacity(
                    ctx.irreversible_token_present,
                    ctx.consent_bundle_present,
                ) {
                    log.log_power_denial(
                        "irreversible",
                        1.0,
                        0.0,
                        &e.to_string(),
                        "0xPWR04",
                    );
                    return Err(e);
                }
            }
        }

        // Log success path for auditability.
        log.log_power_usage(&self.corridor);
        Ok(())
    }

    pub fn snapshot(&self) -> PowerCorridor {
        self.corridor.clone()
    }
}
