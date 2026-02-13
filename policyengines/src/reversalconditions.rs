use crate::alnroles::{RoleSet, canrevertcapability};
use crate::alncore::{
    CapabilityState,
    CapabilityTransitionRequest,
    Decision,
    DecisionReason,
    PolicyStack,
};
use crate::rohmodel::RoHScore;
use crate::envelope::EnvelopeContextView;
use crate::policyreversal::ReversalPolicyFlags;

/// Pure, side-effect-free context for evaluating neuromorph evolution reversals.
/// This is the minimal state tuple, aligned with SECTION,REVERSAL-POLICY,
/// SECTION,ROLES, SECTION,ROLE-COMPOSITION, and the BiophysicalEnvelope / RoH model.
#[derive(Debug, Clone)]
pub struct ReversalContext {
    /// Original capability transition request (from -> to, consent, jurisdiction, etc).
    pub base: CapabilityTransitionRequest,

    /// Risk-of-Harm score before the proposed transition.
    pub roh_before: RoHScore,

    /// Risk-of-Harm score after the proposed transition (counterfactual / predicted).
    pub roh_after: RoHScore,

    /// Shard-level reversal policy flags, frozen in ALN.
    pub reversal_flags: ReversalPolicyFlags,

    /// Active role set, including HOST, ORGANICCPUOWNER, REGULATOR, SOVEREIGNKERNEL, etc.
    pub roles: RoleSet,

    /// Pre-conjoined policy result: BASEMEDICAL ∧ BASEENGINEERING ∧ JURISLOCAL ∧ QUANTUMAISAFETY.
    pub policy_stack: PolicyStack,

    /// View over biophysical envelope outputs for this subject/session.
    pub envelope_ctx: EnvelopeContextView,

    /// Conservative boolean: true only if all softer mitigations (tighten, pause, rest)
    /// have been exhausted without restoring safety.
    pub no_safer_alternative: bool,

    /// Diagnostic-only step flag (Tree-of-Life, AutoChurch, NEUROPRINT, etc.).
    /// If true, capability must not change in this step.
    pub diag_event: bool,
}

/// Evaluate a capability transition with respect to neuromorph evolution reversal.
///
/// Invariants:
/// - MODELONLY / LABBENCH research is never blocked by this kernel.
/// - RoH is monotone and ≤ 0.30 in CapControlledHuman, except when a downgrade reduces RoH.
/// - Neuromorph evolution downgrades are forbidden by default:
///   only allowed if allow_neuromorph_reversal && canrevertcapability && policy_stack.all_pass().
/// - This function is total and side-effect-free: it only reads ctx and returns a Decision.
pub fn evaluate_reversal(ctx: ReversalContext) -> Decision {
    // 1. If this is not a neuromorph evolution downgrade, delegate to the base evaluator.
    if !is_neuromorph_downgrade(ctx.base.from(), ctx.base.to()) {
        // This kernel does not decide upgrades or non-evolution downgrades.
        // Let the normal capability evaluation path handle it.
        return Decision {
            allowed: true,
            reason: DecisionReason::Allowed,
        };
    }

    // 2. Diagnostic isolation: diagnostics may never drive capability changes.
    if ctx.diag_event {
        if ctx.base.to() != ctx.base.from() {
            return Decision {
                allowed: false,
                reason: DecisionReason::DeniedDiagnosticOnlyStep,
            };
        }
        // If there is no actual tier change, diagnostics are harmless from this kernel's view.
        return Decision {
            allowed: true,
            reason: DecisionReason::Allowed,
        };
    }

    // 3. RoH monotonicity and ceiling at CapControlledHuman.
    if matches!(ctx.base.from(), CapabilityState::CapControlledHuman) {
        let before = ctx.roh_before.value();
        let after = ctx.roh_after.value();

        // For general transitions, RoH must not increase and must stay under 0.30.
        // A downgrade that reduces RoH is treated as safety-increasing and allowed past monotone.
        if !reduces_capability_and_roh(&ctx) {
            if after > before || after > 0.30 {
                return Decision {
                    allowed: false,
                    reason: DecisionReason::DeniedRoHViolation,
                };
            }
        } else {
            // Even for safety-increasing downgrades, respect the global ceiling.
            if after > 0.30 {
                return Decision {
                    allowed: false,
                    reason: DecisionReason::DeniedRoHViolation,
                };
            }
        }
    }

    // 4. Default forbid neuromorph evolution downgrades at Tier-1.
    if !ctx.reversal_flags.allow_neuromorph_reversal {
        // You can treat this as the hard, non-waivable default for your own DIDs
        // by never shipping shards that flip this flag.
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedReversalNotAllowedInTier,
        };
    }

    // 5. Sovereignty + explicit order + no safer alternative.
    // canrevertcapability = neuromorphgodsatisfied ∧ explicit_reversal_order ∧ no_safer_alternative
    let required_reg_quorum = ctx.reversal_flags.required_regulator_quorum();
    let can_revert = canrevertcapability(
        &ctx.roles,
        required_reg_quorum,
        ctx.reversal_flags.explicit_reversal_order,
        ctx.no_safer_alternative,
    );

    if !can_revert {
        // Distinguish sovereignty/roles vs. missing order/evidence where possible.
        if !ctx.roles.neuromorph_god_satisfied(required_reg_quorum) {
            return Decision {
                allowed: false,
                reason: DecisionReason::DeniedIllegalDowngradeByNonRegulator,
            };
        }

        if !ctx.reversal_flags.explicit_reversal_order || !ctx.no_safer_alternative {
            return Decision {
                allowed: false,
                reason: DecisionReason::DeniedNoSaferAlternativeNotProved,
            };
        }

        // Fallback: generic consent/roles failure.
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedConsentOrRoles,
        };
    }

    // 6. PolicyStack gate: BASEMEDICAL ∧ BASEENGINEERING ∧ JURISLOCAL ∧ QUANTUMAISAFETY.
    if !ctx.policy_stack.all_pass() {
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedPolicyStackFailure,
        };
    }

    // 7. Envelope advisory context:
    // envelopes may recommend downgrade, but can never bypass sovereignty or policy.
    // If a downgrade is being granted, require that the envelope at least requested it.
    if !ctx.envelope_ctx.request_capability_downgrade() {
        return Decision {
            allowed: false,
            reason: DecisionReason::DeniedIllegalDowngradeByNonRegulator,
        };
    }

    // 8. All guards passed: allow as a last-resort, sovereign, policy-checked reversal.
    Decision {
        allowed: true,
        reason: DecisionReason::Allowed,
    }
}

/// True if this transition is a neuromorph evolution downgrade that reduces
/// rights-bearing capability tier.
fn is_neuromorph_downgrade(from: CapabilityState, to: CapabilityState) -> bool {
    use CapabilityState::*;

    match (from, to) {
        // High-stakes neuromorph evolution downgrades:
        (CapControlledHuman, CapLabBench) |
        (CapControlledHuman, CapModelOnly) |
        (CapGeneralUse, CapControlledHuman) |
        (CapGeneralUse, CapLabBench) |
        (CapGeneralUse, CapModelOnly) => true,

        _ => false,
    }
}

/// True when this downgrade reduces both capability tier and RoH,
/// permitting an exception to strict RoH monotonicity for safety-increasing reversals.
fn reduces_capability_and_roh(ctx: &ReversalContext) -> bool {
    let from = ctx.base.from();
    let to = ctx.base.to();
    let before = ctx.roh_before.value();
    let after = ctx.roh_after.value();

    is_neuromorph_downgrade(from, to) && after < before
}
