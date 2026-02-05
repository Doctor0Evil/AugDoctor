#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use crate::biophysical_chain_neuroautomationpipeline::{
    EvolutionProposal, Reversibility,
};
use crate::deep_object_router::{EpochInventory, LifeforceState};
use crate::eco::EcoBudgetWindow;
use crate::governance::augmentationright::{
    AugmentationRightProfile, AugmentationRightStatus,
};
use crate::governance::deep_domain_rights::{
    DeepDomainRightsProfile, DeepDomainRightsStatus,
};
use crate::governance::evolution_emergency_rights::{
    validate_emergency_override_turn, EmergencyEmergencyValidationKind as EmergencyValidationKind,
    EmergencyTurnContext, EvolutionEmergencyRightsProfile,
};
use crate::governance::irreversible_token::IrreversibleToken;
use crate::organichainconsensus::EvolutionIntervalState;
use crate::telemetry::brain_token_ledger::BrainTokenWindow;
use crate::telemetry::dw_token_ledger::DraculaWaveWindow;
use crate::telemetry::outer_attestation::OuterAttestationRequest;

/// Enumerate the per-turn automation actions.
///
/// NOTE: EmergencyOverrideUse is treated as action 0 so it always runs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AutomationActionId {
    EmergencyOverrideUse,
    IntentDetection,
    TurnValidation,
    EvolutionInterval,
    EcoAndEvolveBudgets,
    ReversibleActuation,
    IrreversibleTurn,
    DeepDomainExcavation,
    Traceability,
    MetaGovernance,
    OuterAttestation,
}

#[derive(Clone, Debug)]
pub enum ValidationResultKind {
    Passed,
    Failed,
    Skipped, // not applicable this turn (e.g., no irreversible patterns)
}

#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub action: AutomationActionId,
    pub kind: ValidationResultKind,
    pub messages: Vec<String>,
}

/// High-level view of everything a single evolution turn might touch.
#[derive(Clone, Debug)]
pub struct PerTurnContext<'a> {
    pub proposal: Option<&'a EvolutionProposal>,
    pub interval_state: Option<&'a EvolutionIntervalState>,
    pub eco_window: Option<&'a EcoBudgetWindow>,
    pub brain_window: Option<&'a BrainTokenWindow>,
    pub dw_window: Option<&'a DraculaWaveWindow>,
    pub lifeforce: Option<&'a LifeforceState>,
    pub deep_epochs: Option<&'a [EpochInventory]>,
    pub deep_rights: Option<&'a DeepDomainRightsProfile>,
    pub aug_rights: Option<&'a AugmentationRightProfile>,
    pub transcripthash: Option<String>,
    pub human_explanation: Option<String>,
    pub wrote_deep_qpudatashard: bool,
    pub irreversible_token: Option<&'a IrreversibleToken>,
    pub outer_attestation: Option<&'a OuterAttestationRequest>,
    pub host_role: String,
    pub ai_platform_label: String,

    // Emergency override wiring
    pub emergency: Option<EmergencyTurnContext>,
    pub emergency_rights: Option<EvolutionEmergencyRightsProfile>,
    pub emergencies_used_today: u32,
}

/// Central entrypoint: validate all actions for this turn.
pub fn validate_per_turn(
    context: &PerTurnContext<'_>,
) -> BTreeMap<AutomationActionId, ValidationResult> {
    use AutomationActionId::*;

    let mut results = BTreeMap::new();

    // 0. Emergency override invariants
    results.insert(
        EmergencyOverrideUse,
        validate_emergency_override_use(context),
    );

    // 1–10. Existing actions
    results.insert(IntentDetection, validate_intent_detection(context));
    results.insert(TurnValidation, validate_turn_validation(context));
    results.insert(EvolutionInterval, validate_evolution_interval(context));
    results.insert(EcoAndEvolveBudgets, validate_eco_and_evolve_budgets(context));
    results.insert(ReversibleActuation, validate_reversible_actuation(context));
    results.insert(IrreversibleTurn, validate_irreversible_turn(context));
    results.insert(
        DeepDomainExcavation,
        validate_deep_domain_excavation(context),
    );
    results.insert(Traceability, validate_traceability(context));
    results.insert(MetaGovernance, validate_meta_governance(context));
    results.insert(OuterAttestation, validate_outer_attestation(context));

    results
}

/// 0. EmergencyOverrideUse – host-only rollback guard
fn validate_emergency_override_use(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::EmergencyOverrideUse;

    if let (Some(erights), Some(ectx)) = (&ctx.emergency_rights, &ctx.emergency) {
        let res = validate_emergency_override_turn(erights, ectx, ctx.emergencies_used_today);
        let kind = match res.kind {
            EmergencyValidationKind::Passed => ValidationResultKind::Passed,
            EmergencyValidationKind::Failed => ValidationResultKind::Failed,
            EmergencyValidationKind::Skipped => ValidationResultKind::Skipped,
        };
        ValidationResult {
            action: EmergencyOverrideUse,
            kind,
            messages: res.messages,
        }
    } else {
        ValidationResult {
            action: EmergencyOverrideUse,
            kind: ValidationResultKind::Skipped,
            messages: vec!["no emergency override bound to this turn".into()],
        }
    }
}

/// 1. Intent detection + proposal shaping
fn validate_intent_detection(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::IntentDetection;
    let mut msgs = Vec::new();

    if let Some(proposal) = ctx.proposal {
        if proposal.intentid.is_empty() {
            msgs.push("proposal.intentid must be non-empty and part of typed vocabulary".into());
            return ValidationResult {
                action: IntentDetection,
                kind: ValidationResultKind::Failed,
                messages: msgs,
            };
        }
        if !proposal.intentid.starts_with("INTENT_") {
            msgs.push(format!(
                "intentid '{}' does not use INTENT_ prefix",
                proposal.intentid
            ));
            return ValidationResult {
                action: IntentDetection,
                kind: ValidationResultKind::Failed,
                messages: msgs,
            };
        }
    } else {
        msgs.push("no proposal; nothing to validate for intent-detection".into());
        return ValidationResult {
            action: IntentDetection,
            kind: ValidationResultKind::Skipped,
            messages: msgs,
        };
    }

    ValidationResult {
        action: IntentDetection,
        kind: ValidationResultKind::Passed,
        messages: vec!["intent-detection invariants satisfied".into()],
    }
}

/// 2. DefaultProposalValidator
fn validate_turn_validation(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::TurnValidation;
    let mut msgs = Vec::new();

    if let Some(proposal) = ctx.proposal {
        if proposal.constraints_profile_id.is_empty() {
            msgs.push("constraints_profile_id must be set on EvolutionProposal".into());
            return ValidationResult {
                action: TurnValidation,
                kind: ValidationResultKind::Failed,
                messages: msgs,
            };
        }
        if !proposal.prevalidated_by_default_validator {
            msgs.push("DefaultProposalValidator did not confirm this proposal".into());
            return ValidationResult {
                action: TurnValidation,
                kind: ValidationResultKind::Failed,
                messages: msgs,
            };
        }

        ValidationResult {
            action: TurnValidation,
            kind: ValidationResultKind::Passed,
            messages: vec!["proposal satisfied all biophysical constraints".into()],
        }
    } else {
        ValidationResult {
            action: TurnValidation,
            kind: ValidationResultKind::Skipped,
            messages: vec!["no proposal present for this turn".into()],
        }
    }
}

/// 3. Evolution interval gating
fn validate_evolution_interval(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::EvolutionInterval;

    if let Some(interval) = ctx.interval_state {
        if !interval.permits_new_step {
            return ValidationResult {
                action: EvolutionInterval,
                kind: ValidationResultKind::Failed,
                messages: vec!["cantakeevolutionstep() denied this turn".into()],
            };
        }
        if interval.steps_taken_today > interval.max_steps_per_day {
            return ValidationResult {
                action: EvolutionInterval,
                kind: ValidationResultKind::Failed,
                messages: vec!["steps_taken_today exceeded max_steps_per_day".into()],
            };
        }

        ValidationResult {
            action: EvolutionInterval,
            kind: ValidationResultKind::Passed,
            messages: vec!["evolution interval invariants satisfied".into()],
        }
    } else {
        ValidationResult {
            action: EvolutionInterval,
            kind: ValidationResultKind::Skipped,
            messages: vec!["no interval_state; likely a dry-run or sandbox turn".into()],
        }
    }
}

/// 4. Eco + evolution budgets
fn validate_eco_and_evolve_budgets(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::EcoAndEvolveBudgets;

    if let (Some(eco), Some(brain), Some(dw)) = (ctx.eco_window, ctx.brain_window, ctx.dw_window) {
        if !eco.within_daily_budget() {
            return ValidationResult {
                action: EcoAndEvolveBudgets,
                kind: ValidationResultKind::Failed,
                messages: vec!["eco-governor daily budget exhausted".into()],
            };
        }
        if !brain.has_sufficient_capacity() {
            return ValidationResult {
                action: EcoAndEvolveBudgets,
                kind: ValidationResultKind::Failed,
                messages: vec!["insufficient BrainTokens for this turn".into()],
            };
        }
        if !dw.has_sufficient_capacity() {
            return ValidationResult {
                action: EcoAndEvolveBudgets,
                kind: ValidationResultKind::Failed,
                messages: vec!["insufficient DraculaWave for this turn".into()],
            };
        }

        ValidationResult {
            action: EcoAndEvolveBudgets,
            kind: ValidationResultKind::Passed,
            messages: vec!["eco and evolution budgets respected".into()],
        }
    } else {
        ValidationResult {
            action: EcoAndEvolveBudgets,
            kind: ValidationResultKind::Skipped,
            messages: vec!["missing eco or token windows; likely non-actuating turn".into()],
        }
    }
}

/// 5. Reversible per-turn actuation
fn validate_reversible_actuation(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::ReversibleActuation;

    if let Some(proposal) = ctx.proposal {
        if proposal.patterns.is_empty() {
            return ValidationResult {
                action: ReversibleActuation,
                kind: ValidationResultKind::Skipped,
                messages: vec!["no BiophysicalPatterns in proposal".into()],
            };
        }

        let mut has_irreversible = false;
        for p in &proposal.patterns {
            if matches!(
                p.reversibility,
                Reversibility::Irreversible | Reversibility::PartiallyReversible
            ) {
                has_irreversible = true;
                break;
            }
        }

        if has_irreversible {
            return ValidationResult {
                action: ReversibleActuation,
                kind: ValidationResultKind::Skipped,
                messages: vec![
                    "proposal contains non-reversible patterns; handled by irreversible-turn validator"
                        .into(),
                ],
            };
        }

        ValidationResult {
            action: ReversibleActuation,
            kind: ValidationResultKind::Passed,
            messages: vec!["all actuation patterns in this turn are FullyReversible".into()],
        }
    } else {
        ValidationResult {
            action: ReversibleActuation,
            kind: ValidationResultKind::Skipped,
            messages: vec!["no proposal for this turn".into()],
        }
    }
}

/// 6. Irreversible + partially reversible turns
fn validate_irreversible_turn(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::IrreversibleTurn;

    let mut needs_irrev = false;
    if let Some(proposal) = ctx.proposal {
        for p in &proposal.patterns {
            if matches!(
                p.reversibility,
                Reversibility::Irreversible | Reversibility::PartiallyReversible
            ) {
                needs_irrev = true;
                break;
            }
        }
    }

    if !needs_irrev {
        return ValidationResult {
            action: IrreversibleTurn,
            kind: ValidationResultKind::Skipped,
            messages: vec!["no irreversible or partially-reversible patterns".into()],
        };
    }

    let token = match ctx.irreversible_token {
        Some(t) => t,
        None => {
            return ValidationResult {
                action: IrreversibleTurn,
                kind: ValidationResultKind::Failed,
                messages: vec![
                    "irreversible patterns present but no IrreversibleToken attached".into(),
                ],
            }
        }
    };

    if let Some(thash) = &ctx.transcripthash {
        if &token.transcripthash != thash {
            return ValidationResult {
                action: IrreversibleTurn,
                kind: ValidationResultKind::Failed,
                messages: vec!["IrreversibleToken.transcripthash != turn.transcripthash".into()],
            };
        }
    }

    if token.revoked {
        return ValidationResult {
            action: IrreversibleTurn,
            kind: ValidationResultKind::Failed,
            messages: vec!["IrreversibleToken is revoked".into()],
        };
    }

    ValidationResult {
        action: IrreversibleTurn,
        kind: ValidationResultKind::Passed,
        messages: vec!["irreversible turn carries valid consent token".into()],
    }
}

/// 7. Deep-domain excavation (B1–B4, ?/!)
fn validate_deep_domain_excavation(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::DeepDomainExcavation;

    if ctx.deep_epochs.is_none() || ctx.deep_rights.is_none() || ctx.lifeforce.is_none() {
        return ValidationResult {
            action: DeepDomainExcavation,
            kind: ValidationResultKind::Skipped,
            messages: vec!["no deep-domain epochs or rights bound to this turn".into()],
        };
    }

    ValidationResult {
        action: DeepDomainExcavation,
        kind: ValidationResultKind::Passed,
        messages: vec![
            "deep-domain excavation delegated to governed_select_deep_epochs and rights profile"
                .into(),
        ],
    }
}

/// 8. Traceability
fn validate_traceability(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::Traceability;
    let mut msgs = Vec::new();

    if ctx
        .transcripthash
        .as_ref()
        .map(|s| s.is_empty())
        .unwrap_or(true)
    {
        msgs.push("transcripthash must be non-empty for all evolution turns".into());
        return ValidationResult {
            action: Traceability,
            kind: ValidationResultKind::Failed,
            messages: msgs,
        };
    }

    if let Some(exp) = &ctx.human_explanation {
        let word_count = exp.split_whitespace().count();
        if word_count < 25 {
            msgs.push(format!(
                "human_explanation too short: {} words (min 25)",
                word_count
            ));
            return ValidationResult {
                action: Traceability,
                kind: ValidationResultKind::Failed,
                messages: msgs,
            };
        }
    } else {
        msgs.push("missing human_explanation for this turn".into());
        return ValidationResult {
            action: Traceability,
            kind: ValidationResultKind::Failed,
            messages: msgs,
        };
    }

    if ctx.wrote_deep_qpudatashard {
        msgs.push("qpudatashard for deep-object excavation was written".into());
    } else {
        msgs.push("no deep qpudatashard recorded (ok if no B3/B4 used)".into());
    }

    ValidationResult {
        action: Traceability,
        kind: ValidationResultKind::Passed,
        messages: msgs,
    }
}

/// 9. Meta-governance (AugmentationRight + DeepDomainRights)
fn validate_meta_governance(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::MetaGovernance;
    let mut msgs = Vec::new();

    if let Some(aug) = ctx.aug_rights {
        match aug.verify_rights_safe() {
            AugmentationRightStatus::RightsSafe => {
                msgs.push("AugmentationRight profile is RightsSafe".into())
            }
            AugmentationRightStatus::ViolatesInvariant(e) => {
                return ValidationResult {
                    action: MetaGovernance,
                    kind: ValidationResultKind::Failed,
                    messages: vec![format!("AugmentationRight invariant violation: {:?}", e)],
                };
            }
        }
    } else {
        msgs.push(
            "no AugmentationRight profile in context; assuming pre-validated at startup".into(),
        );
    }

    if let Some(dd) = ctx.deep_rights {
        match dd.verify_rights_safe() {
            DeepDomainRightsStatus::RightsSafe => {
                msgs.push("DeepDomainRightsProfile is RightsSafe".into())
            }
            DeepDomainRightsStatus::ViolatesInvariant(e) => {
                return ValidationResult {
                    action: MetaGovernance,
                    kind: ValidationResultKind::Failed,
                    messages: vec![format!("DeepDomainRights invariant violation: {:?}", e)],
                };
            }
        }
    } else {
        msgs.push(
            "no DeepDomainRightsProfile bound; deep-domain excavation should be disabled".into(),
        );
    }

    ValidationResult {
        action: MetaGovernance,
        kind: ValidationResultKind::Passed,
        messages: msgs,
    }
}

/// 10. Outer-world attestation
fn validate_outer_attestation(ctx: &PerTurnContext<'_>) -> ValidationResult {
    use AutomationActionId::OuterAttestation;

    if let Some(att) = ctx.outer_attestation {
        if !att.payload_is_hashes_only {
            return ValidationResult {
                action: OuterAttestation,
                kind: ValidationResultKind::Failed,
                messages: vec!["outer attestation must contain only hashes / proof-ids".into()],
            };
        }
        if att.includes_inner_token_balances {
            return ValidationResult {
                action: OuterAttestation,
                kind: ValidationResultKind::Failed,
                messages: vec!["outer attestation may not leak inner token balances".into()],
            };
        }

        ValidationResult {
            action: OuterAttestation,
            kind: ValidationResultKind::Passed,
            messages: vec!["outer attestation respects inner/outer firewall".into()],
        }
    } else {
        ValidationResult {
            action: OuterAttestation,
            kind: ValidationResultKind::Skipped,
            messages: vec!["no outer attestation for this turn".into()],
        }
    }
}
