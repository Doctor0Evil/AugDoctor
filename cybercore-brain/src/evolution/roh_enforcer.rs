#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Corridor-level evolution readiness and RoH geometry are computed
/// elsewhere; this file only enforces the non-derogable invariants:
/// - RoH ∈ [0, 0.3]
/// - No capability downgrade via policy
/// - Only host-sovereign emergency rollback is allowed for safety

/// Scalar risk-of-harm index, normalized to [0, 1].
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RiskOfHarm {
    pub value: f32,
}

impl RiskOfHarm {
    pub const HARD_CEILING: f32 = 0.30;

    pub fn clamped(self) -> Self {
        let v = if self.value < 0.0 {
            0.0
        } else if self.value > 1.0 {
            1.0
        } else {
            self.value
        };
        Self { value: v }
    }

    pub fn is_within_ceiling(&self) -> bool {
        self.clamped().value <= Self::HARD_CEILING
    }
}

/// Lightweight corridor summary used by the enforcer.
/// It is explicitly *capability-monotone* (no implicit downgrades).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorridorSnapshot {
    /// Risk-of-harm at this snapshot.
    pub roh: RiskOfHarm,
    /// Scalar capability score (0–1) summarizing enabled tools/routes.
    /// This must never be reduced by remote policy, only by host-directed
    /// choices or safety rollback to a previous host-approved snapshot.
    pub capability_score: f32,
    /// Flag that this snapshot was explicitly signed/approved by the host.
    pub host_signed: bool,
}

/// High-level evolution proposal.
/// All heavy math (corridor polytopes, host_axis_deltas, etc.)
/// is computed upstream and summarized here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionProposal {
    /// Proposed next RoH (already computed from corridor-level math).
    pub proposed_roh: RiskOfHarm,
    /// Proposed capability score, derived from corridor readiness.
    pub proposed_capability_score: f32,
    /// True only if the host explicitly requested this evolution step
    /// (e.g., via EVOLVE token + DID signature).
    pub host_requested: bool,
    /// Whether this proposal is marked as emergency safety action
    /// (e.g., nanoswarm overheat, corridor breach).
    /// This MUST be set only by the local safety kernel, not by vendors.
    pub emergency_flag: bool,
}

/// Decision outcome from the RoH enforcer.
/// Note: there is **no generic downgrade** variant.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionOutcome {
    /// Proposal is safe and allowed; caller may commit `after` state.
    Allow,
    /// Proposal is rejected; caller must keep `before` state unchanged.
    Deny,
    /// Safety-only rollback to a *previous* corridor snapshot that is
    /// already known to be RoH-safe and host-signed.
    /// This is not a policy downgrade; it is emergency-only.
    EmergencyRollback,
}

/// Sovereign RoH enforcer:
/// - Enforces RoH ≤ 0.30
/// - Enforces host sovereignty over capability_score
/// - Allows emergency rollback only under strict local conditions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoHEnforcer {
    /// Minimum capability score that *must* be preserved unless the host
    /// explicitly requested a lower capability configuration.
    pub min_capability_floor: f32,
}

impl RoHEnforcer {
    pub fn new(min_capability_floor: f32) -> Self {
        Self {
            min_capability_floor,
        }
    }

    /// Core decision function.
    ///
    /// - `before`: current, committed corridor snapshot.
    /// - `after`:   candidate snapshot if the proposal were accepted.
    /// - `proposal`: evolution proposal metadata.
    ///
    /// Invariants:
    /// - If `after.roh` > 0.30 ⇒ Deny.
    /// - If capability would be reduced below both `before` and
    ///   `min_capability_floor` without host request ⇒ Deny.
    /// - If emergency_flag is set AND RoH is already above ceiling ⇒
    ///   EmergencyRollback (safety-only).
    pub fn decide(
        &self,
        before: &CorridorSnapshot,
        after: &CorridorSnapshot,
        proposal: &EvolutionProposal,
    ) -> DecisionOutcome {
        let before_roh = before.roh.clamped();
        let after_roh = after.roh.clamped();

        // 1. If emergency flag is set and current RoH already violates ceiling,
        // force a safety-only rollback.
        if proposal.emergency_flag && !before_roh.is_within_ceiling() {
            return DecisionOutcome::EmergencyRollback;
        }

        // 2. Deny any evolution that would exceed the RoH hard ceiling.
        if !after_roh.is_within_ceiling() {
            return DecisionOutcome::Deny;
        }

        // 3. Capability sovereignty:
        //    Vendors/policies may NOT reduce capability below both:
        //    - current capability, AND
        //    - configured floor
        //    unless the host explicitly requested the change.
        let proposed_cap = after.capability_score;
        let current_cap = before.capability_score;

        if proposed_cap < current_cap && proposed_cap < self.min_capability_floor {
            if !proposal.host_requested {
                // This is a policy / vendor downgrade attempt: block it.
                return DecisionOutcome::Deny;
            }
        }

        // 4. Require host signature on the baseline being modified.
        if !before.host_signed {
            // Cannot evolve from a state that is not explicitly host-approved.
            return DecisionOutcome::Deny;
        }

        // 5. Require that any increase in capability is either host-requested
        //    or at least not raising RoH.
        if proposed_cap > current_cap && !proposal.host_requested {
            // Only allow "free" capability increases if RoH does not increase.
            if after_roh.value > before_roh.value {
                return DecisionOutcome::Deny;
            }
        }

        DecisionOutcome::Allow
    }
}

// ------- Optional helpers for integration -------

impl CorridorSnapshot {
    /// Helper to construct a snapshot from raw values, clamping RoH.
    pub fn from_raw(roh: f32, capability_score: f32, host_signed: bool) -> Self {
        Self {
            roh: RiskOfHarm { value: roh }.clamped(),
            capability_score: capability_score.clamp(0.0, 1.0),
            host_signed,
        }
    }
}

impl EvolutionProposal {
    /// Helper for a normal (non-emergency) host-requested evolution.
    pub fn host_step(proposed_roh: f32, proposed_capability_score: f32) -> Self {
        Self {
            proposed_roh: RiskOfHarm {
                value: proposed_roh,
            }
            .clamped(),
            proposed_capability_score: proposed_capability_score.clamp(0.0, 1.0),
            host_requested: true,
            emergency_flag: false,
        }
    }

    /// Helper for a safety-only emergency proposal.
    pub fn emergency(proposed_roh: f32, proposed_capability_score: f32) -> Self {
        Self {
            proposed_roh: RiskOfHarm {
                value: proposed_roh,
            }
            .clamped(),
            proposed_capability_score: proposed_capability_score.clamp(0.0, 1.0),
            host_requested: false,
            emergency_flag: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_when_after_roh_exceeds_ceiling() {
        let enforcer = RoHEnforcer::new(0.5);
        let before = CorridorSnapshot::from_raw(0.10, 0.80, true);
        let after = CorridorSnapshot::from_raw(0.35, 0.80, true);
        let prop = EvolutionProposal::host_step(0.35, 0.80);

        let decision = enforcer.decide(&before, &after, &prop);
        assert_eq!(decision, DecisionOutcome::Deny);
    }

    #[test]
    fn deny_remote_downgrade_below_floor() {
        let enforcer = RoHEnforcer::new(0.70);
        let before = CorridorSnapshot::from_raw(0.10, 0.90, true);
        let after = CorridorSnapshot::from_raw(0.10, 0.40, true);
        let prop = EvolutionProposal {
            proposed_roh: after.roh,
            proposed_capability_score: after.capability_score,
            host_requested: false,
            emergency_flag: false,
        };

        let decision = enforcer.decide(&before, &after, &prop);
        assert_eq!(decision, DecisionOutcome::Deny);
    }

    #[test]
    fn allow_host_requested_capability_change_within_roh() {
        let enforcer = RoHEnforcer::new(0.50);
        let before = CorridorSnapshot::from_raw(0.10, 0.80, true);
        let after = CorridorSnapshot::from_raw(0.12, 0.60, true);
        let prop = EvolutionProposal::host_step(0.12, 0.60);

        let decision = enforcer.decide(&before, &after, &prop);
        assert_eq!(decision, DecisionOutcome::Allow);
    }

    #[test]
    fn emergency_rollback_when_before_roh_above_ceiling() {
        let enforcer = RoHEnforcer::new(0.50);
        let before = CorridorSnapshot::from_raw(0.40, 0.80, true);
        let after = CorridorSnapshot::from_raw(0.25, 0.50, true);
        let prop = EvolutionProposal::emergency(0.25, 0.50);

        let decision = enforcer.decide(&before, &after, &prop);
        assert_eq!(decision, DecisionOutcome::EmergencyRollback);
    }
}
