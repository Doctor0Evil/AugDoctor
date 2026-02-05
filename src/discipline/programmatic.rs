#![forbid(unsafe_code)]

//! Programmatic discipline envelopes for the biophysical-blockchain.
//!
//! This module defines a unifying `ProgrammaticDiscipline` trait and a concrete
//! `EvolutionTurnDiscipline` struct that binds:
//! - inner-ledger turn state (`DailyTurnState`),
//! - ALN-hosted policy (`EvolutionTurnPolicy2026v1`),
//! - and per-turn validation results,
//! into a single, typed constraint surface for evolution-related mutations.[file:39]
//!
//! It does not change lifeforce, eco, or consent semantics; it composes them by
//! calling into existing guards before deciding whether an evolution turn may
//! be consumed.[file:42][file:47]

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::biophysical_blockchain::turns::DailyTurnState;
use crate::biophysical_chain::neuro_automation_pipeline::EvolutionProposal;
use crate::organichain_consensus::EvolutionIntervalState;
use crate::biophysical_chain::constraints::BiophysicalConstraints;
use crate::governance::aln_loader::AlnShardLoader;
use crate::governance::evolution_turn_policy::EvolutionTurnPolicy2026v1; // ALN binding.[file:39]

/// Logical outcome of a discipline check for a specific operation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisciplineDecisionKind {
    /// Operation is allowed to proceed under this discipline.
    Allow,
    /// Operation is rejected; higher layers should not attempt to commit it.
    Deny,
    /// Operation is recorded only (e.g., for logs or telemetry), not executed.
    LogOnly,
}

/// Detailed result of applying a discipline envelope to some operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisciplineDecision {
    pub kind: DisciplineDecisionKind,
    pub reason: String,
}

impl DisciplineDecision {
    pub fn allow() -> Self {
        Self {
            kind: DisciplineDecisionKind::Allow,
            reason: "discipline: allow".into(),
        }
    }

    pub fn deny<S: Into<String>>(reason: S) -> Self {
        Self {
            kind: DisciplineDecisionKind::Deny,
            reason: reason.into(),
        }
    }

    pub fn log_only<S: Into<String>>(reason: S) -> Self {
        Self {
            kind: DisciplineDecisionKind::LogOnly,
            reason: reason.into(),
        }
    }
}

impl fmt::Display for DisciplineDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.reason)
    }
}

/// Unifying trait for all programmatic discipline envelopes.
///
/// Each discipline type is responsible for enforcing one axis of constraint
/// (e.g., evolution turns per day, deep-domain excavation rights, eco budgets)
/// in a purely non-financial, per-host, host-sovereign way.[file:47]
pub trait ProgrammaticDiscipline {
    /// Symbolic identifier for this discipline envelope (for logs and proofs).
    fn id(&self) -> &'static str;

    /// Short, machine-readable description of what this discipline governs.
    fn description(&self) -> &'static str;

    /// Apply this discipline to the given proposal and state snapshot,
    /// returning a structured decision.
    fn evaluate(
        &self,
        ctx: &mut DisciplineContext<'_>,
    ) -> DisciplineDecision;
}

/// Minimal context needed for discipline decisions on evolution turns.
///
/// This mirrors the evolution-related fields already present in your
/// per-turn validation context, but is kept narrow so it can be re-used
/// outside the full validator matrix if needed.[file:39]
#[derive(Debug)]
pub struct DisciplineContext<'a> {
    /// Current evolution proposal being considered (if any).
    pub proposal: Option<&'a EvolutionProposal>,

    /// Per-host daily turn state (inner-ledger, mutable).
    pub daily_turn_state: &'a mut DailyTurnState,

    /// Evolution interval state from OrganichainConsensus, if available.
    pub interval_state: Option<&'a EvolutionIntervalState>,

    /// Biophysical constraints resolved from ALN shards (pain, blood, fear, etc.).
    pub constraints: Option<&'a BiophysicalConstraints>,

    /// Host identifier (Bostrom/ALN DID).
    pub host_id: &'a str,
}

/// Concrete discipline: daily evolution-turn envelope.
///
/// This binds host-local `DailyTurnState` with the host-authored ALN
/// policy `EvolutionTurnPolicy2026v1` and an inner-ledger hard ceiling
/// on turns per day.[file:42]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionTurnDiscipline {
    /// Policy as authored by the host in `evolutionturnpolicy.aln`.
    pub policy: EvolutionTurnPolicy2026v1,

    /// Inner-ledger hard ceiling; policy cannot exceed this.
    pub compiled_max_daily_turns: u8,
}

impl EvolutionTurnDiscipline {
    /// Construct from ALN for a given host, clamping to the compiled ceiling.
    pub fn from_aln<S: AsRef<str>>(
        loader: &AlnShardLoader,
        host_id: S,
        compiled_max_daily_turns: u8,
    ) -> Result<Self, String> {
        let mut policy: EvolutionTurnPolicy2026v1 = loader
            .load_for_host("EvolutionTurnPolicy2026v1", host_id.as_ref())
            .map_err(|e| format!("failed to load EvolutionTurnPolicy2026v1: {e}"))?;

        let max_turns = policy
            .maxturnsperday
            .min(compiled_max_daily_turns as u32) as u8;

        policy.maxturnsperday = max_turns as u32;

        Ok(Self {
            policy,
            compiled_max_daily_turns,
        })
    }

    /// Internal evaluation logic, separated to keep the trait impl simple.
    fn evaluate_internal(
        &self,
        ctx: &mut DisciplineContext<'_>,
    ) -> DisciplineDecision {
        // 1. If there is no proposal, nothing to enforce; treat as LogOnly.
        if ctx.proposal.is_none() {
            return DisciplineDecision::log_only(
                "no evolution proposal present; nothing to turn-gate",
            );
        }

        // 2. Interval state gating (min spacing, max steps/day).
        if let Some(interval) = ctx.interval_state {
            if !interval.permits_new_step {
                return DisciplineDecision::log_only(
                    "Organichain evolution interval exhausted for today",
                );
            }
            if interval.steps_taken_today >= interval.max_steps_per_day {
                return DisciplineDecision::deny(
                    "Organichain max_steps_per_day reached",
                );
            }
        }

        // 3. ALN maxturnsperday vs current DailyTurnState.
        let policy_max = self.policy.maxturnsperday as u8;
        let ceiling = policy_max.min(self.compiled_max_daily_turns);

        // DailyTurnState handles date rollover and quota; if it refuses,
        // the turn limit is reached for today.[file:42]
        let can_consume = ctx.daily_turn_state.can_consume_turn(ceiling);
        if !can_consume {
            return DisciplineDecision::deny("daily evolution-turn limit reached");
        }

        // 4. Optional: reflect pain/blood/fear envelopes as guardrails,
        // but do not override existing BiophysicalConstraints; those
        // are enforced in the core validator pipeline.[file:39][file:42]
        if let Some(constraints) = ctx.constraints {
            if !constraints.within_daily_evolution_envelopes(&self.policy) {
                return DisciplineDecision::deny(
                    "biophysical envelopes (pain/blood/fear) exceeded for evolution-turn policy",
                );
            }
        }

        DisciplineDecision::allow()
    }
}

impl ProgrammaticDiscipline for EvolutionTurnDiscipline {
    fn id(&self) -> &'static str {
        "discipline.evolution-turn"
    }

    fn description(&self) -> &'static str {
        "Per-host, per-day evolution-turn discipline derived from EvolutionTurnPolicy2026v1 and clamped by inner-ledger ceilings."
    }

    fn evaluate(
        &self,
        ctx: &mut DisciplineContext<'_>,
    ) -> DisciplineDecision {
        self.evaluate_internal(ctx)
    }
}
