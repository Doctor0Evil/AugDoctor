use crate::types::{BioTokenState, HostEnvelope};
use crate::lifeforce::{LifeforceBand, LifeforceBandSeries};
use crate::tokens::RadsToken;
use log::{error, warn, info};

/// Hard, non-configurable biosafety limits for RADS.
/// These are *not* read from governance shards; they are code-constants.
pub const RADS_BIOPHYSICAL_MAX: f64 = 0.0021;      // absolute hard-stop per cycle
pub const RADS_SAFE_OPERATING_MARGIN: f64 = 0.0017; // soft-warn band

/// BRAIN-coupled predictive threshold:
/// as BRAIN drops toward brainmin, allowable RADS are tightened.
pub const RADS_BRAIN_COUPLING_FACTOR: f64 = 0.65;   // scales brain reserve into max RADS

/// Predictive safety inputs derived from BRAIN and lifeforce history
pub struct PredictiveExposureBudget {
    pub max_rads_this_cycle: f64,
    pub brain_reserve: f64,
    pub cognitive_risk_band: LifeforceBand,
}

/// Core safety structure enforcing !#mutation!rules plus BRAIN feedback
pub struct MutationGuard;

impl MutationGuard {
    /// Compute a predictive RADS budget from current BRAIN and lifeforce state.
    /// This *tightens* limits as BRAIN approaches brainmin or lifeforce enters SoftWarn.
    pub fn predictive_budget(
        host_env: &HostEnvelope,
        state: &BioTokenState,
        history: Option<&LifeforceBandSeries>,
    ) -> PredictiveExposureBudget {
        // Brain reserve above absolute floor
        let brain_reserve = (state.brain - host_env.brainmin).max(0.0);

        // Default band if no history available
        let mut cognitive_band = LifeforceBand::Safe;
        if let Some(series) = history {
            if let Some(last) = series.samples.last() {
                cognitive_band = last.band;
            }
        }

        // Map lifeforce band to an additional safety factor
        let lifeforce_factor = match cognitive_band {
            LifeforceBand::Safe     => 1.0,
            LifeforceBand::SoftWarn => 0.6,
            LifeforceBand::HardStop => 0.0, // no additional RADS allowed
        };

        // Base limit from global hard cap and BRAIN reserve
        let brain_ceiling = brain_reserve * RADS_BRAIN_COUPLING_FACTOR;

        // Effective predictive ceiling is the *tightest* of:
        // - absolute system hard max
        // - brain-derived ceiling
        // - lifeforce scaling
        let mut max_rads = RADS_BIOPHYSICAL_MAX
            .min(brain_ceiling)
            * lifeforce_factor;

        // Never exceed the global hard-stop even if math misbehaves
        if max_rads > RADS_BIOPHYSICAL_MAX {
            max_rads = RADS_BIOPHYSICAL_MAX;
        }

        PredictiveExposureBudget {
            max_rads_this_cycle: max_rads.max(0.0),
            brain_reserve,
            cognitive_risk_band: cognitive_band,
        }
    }

    /// Enforce !#mutation!rules with predictive BRAIN coupling and decay-aware check.
    ///
    /// Returns:
    /// - true  = safe to proceed with operation
    /// - false = operation must be cancelled/terminated
    pub fn enforce(
        host_env: &HostEnvelope,
        state: &mut BioTokenState,
        rads: &mut RadsToken,
        history: Option<&LifeforceBandSeries>,
        decay: &crate::decay::DecayState,
    ) -> bool {
        let budget = Self::predictive_budget(host_env, state, history);

        // Hard ban when lifeforce already at HardStop.
        if matches!(budget.cognitive_risk_band, LifeforceBand::HardStop) {
            error!(
                "[RADS] HardStop lifeforce: denying all radiation for host {}",
                host_env.hostid
            );
            Self::trigger_termination(state, rads, "HARDBAND_BLOCK");
            return false;
        }

        // Clamp requested RADS to predictive budget *before* decay is applied.
        let requested = rads.requested();
        if requested > budget.max_rads_this_cycle {
            warn!(
                "[RADS] Predictive clamp: requested={:.8} > predictive_budget={:.8} (brain_reserve={:.6}) for host {}",
                requested,
                budget.max_rads_this_cycle,
                budget.brain_reserve,
                host_env.hostid
            );
            rads.clamp_to(budget.max_rads_this_cycle);
        }

        // Compute post-decay effective exposure.
        let applied_rads = rads.value_after_decay(decay);

        // Global hard-stop check (no mutation allowed beyond this).
        if applied_rads > RADS_BIOPHYSICAL_MAX {
            error!(
                "[RADS] Mutation risk: effective exposure {:.8} exceeds hard biophysical max {:.8} for host {}",
                applied_rads,
                RADS_BIOPHYSICAL_MAX,
                host_env.hostid
            );
            Self::trigger_termination(state, rads, "RADS_HARDSTOP_EXCEEDED");
            return false;
        }

        // Soft margin: minimize or shorten operation automatically.
        if applied_rads > RADS_SAFE_OPERATING_MARGIN {
            warn!(
                "[RADS] Near limit: {:.8} > safe margin {:.8} for host {}; minimizing operation",
                applied_rads,
                RADS_SAFE_OPERATING_MARGIN,
                host_env.hostid
            );
            Self::minimize_operation(state);
        }

        info!(
            "[RADS] Exposure approved: {:.8} within predictive and hard limits for host {}",
            applied_rads,
            host_env.hostid
        );

        true
    }

    /// Cancel / terminate operation when mutation threshold or policy violation is detected.
    fn trigger_termination(state: &mut BioTokenState, rads: &mut RadsToken, reason: &str) {
        rads.revoke(); // !#mutation!rules: RADS revoked, operation cannot continue
        state.smart = 0.0; // force manual mode; no autonomous continuation

        // Any host-level audit or evolution-state tagging can be performed here
        // via surrounding runtime (e.g., InnerLedger event reason strings).
    }

    /// Shorten or minimize a high-exposure procedure, with BRAIN-coupled protection.
    fn minimize_operation(state: &mut BioTokenState) {
        // Reduce WAVE and SMART to lower cognitive load during the remainder of this cycle.
        state.wave *= 0.4;
        state.smart = state.smart.min(state.brain * 0.3);
    }
}
