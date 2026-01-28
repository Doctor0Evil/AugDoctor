//! Biophysical aura + karma-governed decay/evolve modulation.
//!
//! This module plugs into the existing lifeforce-guarded inner-ledger path
//! (`apply_lifeforce_guarded_adjustment` + `InnerLedger::system_apply`) and
//! provides a *non-financial*, per-host, per-domain multiplier that:
//! - Rewards sustained decay reduction / eco-healthy behavior.
//! - Caps per-day EVOLVE for sensitive “teeth-claws” defensive mutations.
//! - Never bypasses lifeforce, eco, SCALE, or consent invariants.
//!
//! It is intentionally narrow: it only shapes *how much* mutation from a
//! given `SystemAdjustment` may be realized, never *who* owns anything.
use crate::lifeforce::LifeforceError;
use crate::types::{
    BioTokenState,
    HostEnvelope,
    SystemAdjustment,
};
use crate::mutation::LifeforceMutator;
/// High-level “karma class” for the host’s current pattern.
///
/// This is not an asset and has **no** balances; it is a derived label from
/// civic / eco / decay patterns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KarmaClass {
    /// Consistently safe, eco-neutral, low-decay, high-consent patterns.
    Benevolent,
    /// Mixed patterns, mostly safe but without strong positive history.
    Balanced,
    /// Repeated risky patterns, frequent HardStop, or eco abuse.
    Reckless,
}
/// Biophysical aura snapshot used to shape decay and evolution reward.
/// All fields are normalized scalars or small bounded counters, *per host*.
#[derive(Clone, Debug)]
pub struct BiophysicalAura {
    /// Normalized karma score in [0.0, 1.0], derived from CivicRewardProfile
    /// and decay reduction history (e.g., sleep hygiene, eco neutrality).
    pub karma_score: f32,
    /// How often the host hits HardStop lifeforce bands (0.0–1.0),
    /// 0.0 = never, 1.0 = constantly.
    pub hard_stop_ratio: f32,
    /// How often lifeforce remains in Safe band under load (0.0–1.0).
    pub safe_band_ratio: f32,
    /// Normalized eco-usage score in [0.0, 1.0], where lower is better.
    /// 0.0 = always low eco-cost, 1.0 = frequently near ecoflopslimit.
    pub eco_usage_score: f32,
    /// Total EVOLVE usage in the current UTC day for this host and domain,
    /// already accumulated from prior adjustments (0.0–1.0 normalized).
    /// 0.0 = unused, 1.0 = at daily cap.
    pub daily_evolve_usage: f32,
    /// Optional SCALE-derived upper bound for this day (0.0–1.0),
    /// representing maximum safe upgrade span for the host.
    pub daily_scale_budget: f32,
}
/// Mutation / upgrade domain used by the aura logic.
/// This is *not* a token; it is a routing label for safety policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvolutionDomain {
    /// Default / generic evolution (cognitive, sensory tuning, etc.).
    Generic,
    /// Defensive, somatic-only traits such as teeth/claws hardening,
    /// subject to much tighter per-day EVOLVE caps.
    TeethClawsDefense,
    /// Purely software / tooling upgrades, no somatic mutation.
    SoftwareOnly,
}
/// Safe multiplier output from the aura logic.
#[derive(Clone, Debug)]
pub struct DecayMultiplier {
    /// Scalar in [0.0, 1.0] that *scales down* the pending `SystemAdjustment`.
    /// 1.0 = no change, 0.5 = half of the proposed EVOLVE delta applied,
    /// 0.0 = blocked (lifeforce or consent should already gate in that case).
    pub factor: f32,
    /// Whether daily EVOLVE cap for this domain was reached or exceeded.
    pub daily_cap_hit: bool,
}
/// Classify a numeric karma score into a coarse class.
pub fn classify_karma(score: f32) -> KarmaClass {
    let s = score.clamp(0.0, 1.0);
    if s >= 0.75 {
        KarmaClass::Benevolent
    } else if s >= 0.35 {
        KarmaClass::Balanced
    } else {
        KarmaClass::Reckless
    }
}
/// Core policy: compute a safe multiplier for a prospective mutation.
///
/// - Pure function: depends only on `aura` and `domain`.
/// - Never returns > 1.0.
/// - Adds *extra* caps for `TeethClawsDefense`, so defensive traits
/// cannot outpace EVOLVE capacity in a single day.
///
/// This is where “good karma” helps: higher `karma_score` and better
/// eco / lifeforce patterns allow **using more** of the already safe
/// EVOLVE/SCALE budget, not minting anything new.
pub fn safe_decay_multiplier(
    aura: &BiophysicalAura,
    domain: EvolutionDomain,
) -> DecayMultiplier {
    // 1. Base factor from karma class.
    let karma = aura.karma_score.clamp(0.0, 1.0);
    let hard_stop = aura.hard_stop_ratio.clamp(0.0, 1.0);
    let safe_band = aura.safe_band_ratio.clamp(0.0, 1.0);
    let eco = aura.eco_usage_score.clamp(0.0, 1.0);
    let used_today = aura.daily_evolve_usage.clamp(0.0, 1.0);
    let scale_budget = aura.daily_scale_budget.clamp(0.0, 1.0);
    let class = classify_karma(karma);
    // Start from a conservative base; karma then shapes adjustments.
    let mut factor: f32 = match class {
        KarmaClass::Benevolent => 0.90,
        KarmaClass::Balanced => 0.70,
        KarmaClass::Reckless => 0.40,
    };
    // 2. Penalize frequent HardStop, reward sustained Safe band.
    // More HardStop => lower factor; more Safe => higher factor.
    // Clamp within [0.25, 1.0] to never fully bypass other guards.
    let hard_penalty = 0.30 * hard_stop;
    let safe_bonus = 0.20 * safe_band;
    factor = (factor - hard_penalty + safe_bonus).clamp(0.25, 1.0);
    // 3. Eco usage: if eco_usage_score is high, throttle additional mutation.
    // This is a gentle, non-financial eco karma effect.
    let eco_penalty = 0.35 * eco;
    factor = (factor - eco_penalty).clamp(0.20, 1.0);
    // 4. SCALE / per-day EVOLVE budget: the host’s SCALE has already
    // derived a daily budget; the aura should *never* exceed it.
    //
    // We treat `daily_evolve_usage` as 0.0–1.0 of `daily_scale_budget`.
    // If usage already >= budget, multiplier for “growth” becomes ~0.
    let mut daily_cap_hit = false;
    if scale_budget > 0.0 {
        // effective remaining fraction in [0.0, 1.0]
        let remaining = (scale_budget - used_today).max(0.0) / scale_budget;
        if remaining <= 0.0 {
            factor = 0.0;
            daily_cap_hit = true;
        } else {
            // As we approach the budget, squeeze factor harder.
            factor *= remaining.clamp(0.0, 1.0);
        }
    }
    // 5. Domain-specific caps, especially for Teeth/Claws defense.
    //
    // TeethClawsDefense:
    // - Hard per-day “evolve” soft cap: at most 30% of daily SCALE budget
    // may be used by this domain.
    // - When used_today already exceeds 0.30*scale_budget, factor is forced low.
    //
    // Generic:
    // - Slightly looser, but still bounded by SCALE + eco + lifeforce.
    //
    // SoftwareOnly:
    // - Most permissive (within SCALE), as it does not encode somatic mutation.
    match domain {
        EvolutionDomain::TeethClawsDefense => {
            // Maximum fraction of daily SCALE this domain may consume.
            let domain_fraction_cap: f32 = 0.30;
            if scale_budget > 0.0 {
                let domain_cap = scale_budget * domain_fraction_cap;
                if used_today >= domain_cap {
                    // Already used the defensive mutation budget for today.
                    factor = 0.0;
                    daily_cap_hit = true;
                } else {
                    // As we approach the defensive-cap, squeeze factor harder.
                    let remaining_domain =
                        ((domain_cap - used_today) / domain_cap).clamp(0.0, 1.0);
                    factor *= remaining_domain * 0.75; // extra tightening
                }
            } else {
                // Without SCALE, still allow only very small evolution steps.
                factor = factor.min(0.40);
            }
            // Defensive domain should never be able to use 100% of a proposed
            // adjustment, even in Benevolent karma; keep a global ceiling.
            factor = factor.min(0.80);
        }
        EvolutionDomain::Generic => {
            // Generic evolution: may use more of the budget when karma is high,
            // but still subordinate to SCALE and eco.
            factor = factor.min(0.95);
        }
        EvolutionDomain::SoftwareOnly => {
            // Software-only upgrades: rely mostly on eco/SCALE; allow closer
            // to full effect, but never >1.0.
            factor = factor.min(1.0);
        }
    }
    // Final clamp for absolute safety.
    DecayMultiplier {
        factor: factor.clamp(0.0, 1.0),
        daily_cap_hit,
    }
}
/// Apply aura-governed decay/evolution to a pending adjustment,
/// then pass through the existing lifeforce guards.
///
/// This is the intended integration hook from `InnerLedger::system_apply`
/// or from an evolution-specific orchestration path.
///
/// Usage pattern (inside crate):
/// - Derive `BiophysicalAura` + `EvolutionDomain` from host analytics.
/// - Call this function instead of calling `LifeforceMutator::apply_guarded`
/// directly for evolution-related `SystemAdjustment`s.
pub fn apply_aura_shaped_adjustment<M>(
    state: &mut BioTokenState,
    env: &HostEnvelope,
    mut adj: SystemAdjustment,
    aura: &BiophysicalAura,
    domain: EvolutionDomain,
    lifeforce_mut: &M,
) -> Result<(), LifeforceError>
where
    M: LifeforceMutator,
{
    let mult = safe_decay_multiplier(aura, domain);
    // If multiplier is zero, we simply do not apply the mutation.
    if mult.factor <= 0.0 {
        // From the ledger’s perspective this is a no-op; callers can interpret
        // `daily_cap_hit` to surface UI feedback (“daily defensive evolution
        // quota reached, try again tomorrow”).
        return Ok(());
    }
    let f = mult.factor as f64;
    // Scale only the *evolutionary* components of the adjustment.
    // This assumes that deltabrain / deltawave / deltanano / deltasmart
    // are used as evolution/upgrade carriers; BLOOD/OXYGEN should already
    // be gated by lifeforce and usually remain neutral here.
    adj.deltabrain *= f;
    adj.deltawave *= f;
    adj.deltanano *= f;
    adj.deltasmart *= f;
    // Eco-cost is also scaled, to keep accounting consistent.
    adj.ecocost *= f;
    // Delegate to the canonical lifeforce guard.
    lifeforce_mut.apply_guarded(state, env.clone(), adj)
}
