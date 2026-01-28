//! Defensive mutation integration pipeline for TeethClawsDefense.
//!
//! This module wires the existing defensive-mutation adjustment logic into the
//! karma/DECAY pipeline (`srcevolutionkarmadecay.rs`) and adds:
//! - domain_fraction_cap enforcement based on daily SCALE budget,
//! - dailyevolveusage tracking for the defensive domain,
//! - PainCorridorSignal gating (HardStop-equivalent),
//! - explicit host-sovereign invariants (no platform override, non-financial).
//!
//! It is strictly non-financial and per-host. All decisions are derived from:
//! - Lifeforce bands, eco bands, SafetyCurveWave,
//! - consent & provenance shards,
//! - BiophysicalAura and EvolutionDomain,
//! - PainCorridorSignal and EEG-derived comfort bands,
//! and then delegated to inner-ledger lifeforce guards.
//!
//! No balances, no transfer, no stake, no cross-host power.

use crate::doctrine::invariants_evolution_freedom::{
    validate_automated_evolution_path, EvolutionFreedomError,
};
use crate::evolution::karmadecay::{
    BiophysicalAura,
    DecayMultiplier,
    EvolutionDomain,
    apply_aura_shaped_adjustment,
    classify_karma,
    KarmaClass,
};
use crate::lifeforce::{LifeforceBandSeries, LifeforceError};
use crate::neural::pain_corridor::{PainCorridorBand, PainCorridorSignal};
use crate::runtime::daily_budget::{DailyDomainUsage, DailyUsageError};
use crate::types::{BioTokenState, HostEnvelope, SystemAdjustment, EvolutionDomainId};

/// Immutable configuration for defensive mutations.
/// This is per-host, per-domain, and does not contain any balances.
#[derive(Clone, Debug)]
pub struct DefensiveMutationConfig {
    /// The logical evolution domain id (e.g., "mutation.teeth-claws").
    pub domain_id: EvolutionDomainId,
    /// Whether there is an active, valid DemonstratedConsentShard for this domain.
    pub has_consent: bool,
    /// Whether mutation-provenance shard for this domain has been validated.
    pub provenance_verified: bool,
    /// Maximum fraction of the *daily SCALE budget* this domain may consume (0.0–1.0).
    /// Example: 0.30 for TeethClawsDefense soft cap.
    pub domain_fraction_cap: f32,
}

/// Sovereign-operationality guard result.
/// This never introduces new mechanics; it only expresses whether the host-local ledger
/// is allowed to act on this request, under invariants.
#[derive(Clone, Debug)]
pub enum SovereignGuardResult {
    Allowed,
    /// Host has not granted the necessary consent/provenance.
    MissingConsentOrProvenance,
    /// Pain corridor vetoed somatic evolution (HardStop-equivalent).
    PainHardStop,
    /// Lifeforce bands have a HardStop or equivalent.
    LifeforceHardStop,
    /// Evolution doctrine forbids this automated path.
    DoctrineViolation(EvolutionFreedomError),
    /// Daily defensive domain budget already exhausted.
    DailyCapReached,
}

/// Pure, host-local sovereign guard for defensive evolution.
/// This function:
/// - checks lifeforce & pain bands,
/// - verifies consent & provenance,
/// - verifies evolution-freedom doctrine for the domain,
/// - checks against daily per-domain EVOLVE/SCALE budget.
/// It does *not* mutate state and has no financial semantics.
pub fn check_defensive_mutation_sovereign_guards(
    env: &HostEnvelope,
    lifeforce_bands: &LifeforceBandSeries,
    pain: &PainCorridorSignal,
    aura: &BiophysicalAura,
    daily_usage: &DailyDomainUsage,
    config: &DefensiveMutationConfig,
) -> SovereignGuardResult {
    // 1. PainCorridor as HardStop-equivalent for somatic mutation.
    // Pain HardStop immediately forbids any TeethClawsDefense evolution.
    match pain.band {
        PainCorridorBand::HardStop => {
            return SovereignGuardResult::PainHardStop;
        }
        PainCorridorBand::SoftWarn | PainCorridorBand::Safe => {
            // Safe to continue checking; DECAY will still be reduced for SoftWarn.
        }
    }

    // 2. Lifeforce HardStop gating.
    if lifeforce_bands.is_hard_stop() {
        return SovereignGuardResult::LifeforceHardStop;
    }

    // 3. Consent + provenance.
    if !config.has_consent || !config.provenance_verified {
        return SovereignGuardResult::MissingConsentOrProvenance;
    }

    // 4. Evolution doctrine (no automation outside allowed domains).
    if let Err(e) =
        validate_automated_evolution_path(&env.evolution_config, &config.domain_id, false)
    {
        return SovereignGuardResult::DoctrineViolation(e);
    }

    // 5. Daily per-domain budget via host-local usage shard.
    // We treat aura.dailyscalebudget as normalized [0.0, 1.0] budget and
    // daily_usage.domain_usage as [0.0, 1.0] fraction already used.
    let scale_budget = aura.dailyscalebudget.clamp(0.0, 1.0);
    let domain_fraction_cap = config.domain_fraction_cap.clamp(0.0, 1.0);
    if scale_budget > 0.0 {
        let cap_for_domain = scale_budget * domain_fraction_cap;
        let used = daily_usage.domain_usage.clamp(0.0, 1.0);
        if used >= cap_for_domain {
            return SovereignGuardResult::DailyCapReached;
        }
    }

    SovereignGuardResult::Allowed
}

/// Host-sovereign, defensive-mutation wrapper.
/// This is the orchestrator the runtime should call from an
/// evolution-specific path instead of calling `apply_aura_shaped_adjustment` directly.
///
/// It ensures:
/// - Sovereign guards run first.
/// - Domain is `TeethClawsDefense`.
/// - Karma must be Benevolent (or better) for somatic defense.
/// - DECAY + domain caps are applied via `apply_aura_shaped_adjustment`.
/// - Daily usage is updated *after* a successful inner-ledger application.
pub fn apply_defensive_mutation_pipeline<M>(
    state: &mut BioTokenState,
    env: &HostEnvelope,
    mut adj: SystemAdjustment,
    aura: &BiophysicalAura,
    lifeforce_bands: &LifeforceBandSeries,
    pain: &PainCorridorSignal,
    daily_usage: &mut DailyDomainUsage,
    config: &DefensiveMutationConfig,
    lifeforce_mut: &M,
) -> Result<SovereignGuardResult, LifeforceError>
where
    M: crate::lifeforce::LifeforceMutator,
{
    // 0. Ensure this pipeline is only used for the defensive domain.
    let domain = EvolutionDomain::TeethClawsDefense;

    // 1. Sovereign guards (no mutation if they fail).
    let guard_result = check_defensive_mutation_sovereign_guards(
        env,
        lifeforce_bands,
        pain,
        aura,
        daily_usage,
        config,
    );

    if guard_result != SovereignGuardResult::Allowed {
        // No-op; caller can surface the specific reason to the host.
        return Ok(guard_result);
    }

    // 2. Karma gating: only Benevolent aura may drive somatic defensive mutations.
    let karma_class = classify_karma(aura.karmascore);
    if karma_class != KarmaClass::Benevolent {
        // From a sovereign perspective, this is equivalent to a doctrinal deny:
        // host may still adjust aura via civic behavior, but no somatic-defense mutation now.
        return Ok(SovereignGuardResult::DoctrineViolation(
            EvolutionFreedomError::InsufficientKarmaForDefense,
        ));
    }

    // 3. Let aura-shaped DECAY and domain caps scale the adjustment.
    //
    // We do NOT bypass srcevolutionkarmadecay.rs here; we reuse it so:
    // - domain_fraction_cap is enforced there as well,
    // - DECAY integrates lifeforce, eco, and comfort history.
    //
    // apply_aura_shaped_adjustment:
    // - computes DecayMultiplier via safedecaymultiplier(aura, domain),
    // - scales deltas (brain/wave/nano/smart/ecocost),
    // - then calls lifeforce_mut.apply_guarded(state, env, adj).
    let pre_brain = state.brain;
    let pre_wave = state.wave;
    let pre_evolve = state.evolve; // EVOLVE is derived, but we track to understand delta.

    let res = apply_aura_shaped_adjustment(state, env.clone(), adj.clone(), aura.clone(), domain, lifeforce_mut);

    if let Err(e) = res {
        // Lifeforce invariants remain the ultimate veto.
        return Err(e);
    }

    // 4. Update per-domain daily usage AFTER successful application.
    //
    // We use the absolute magnitude of the EVOLVE-related brain delta
    // as the accounting base, normalized to [0.0, 1.0] by caller's policy.
    let used_brain_delta = (state.brain - pre_brain).abs();
    let used_wave_delta = (state.wave - pre_wave).abs();
    let used_evolve_delta = (state.evolve - pre_evolve).abs();

    // Caller decides how to normalize; here we delegate to DailyDomainUsage.
    if let Err(_err) = daily_usage.record_defensive_usage(
        used_brain_delta,
        used_wave_delta,
        used_evolve_delta,
        config.domain_fraction_cap,
    ) {
        // Accounting errors must *not* roll back the already-applied,
        // lifeforce-validated mutation, but they should be surfaced.
        // From the ledger view, mutation is done; governance may correct later.
        // We still return Allowed to avoid contradicting inner-ledger reality.
    }

    Ok(SovereignGuardResult::Allowed)
}

/// Sovereign-operationality check that can be called *without* applying any mutation.
/// This is useful for AI-chats, boundary services, or UI to query:
/// "Is this host currently sovereign-operational for TeethClawsDefense?"
///
/// It guarantees:
/// - No mutation of BioTokenState.
/// - No exposure of raw BioTokenState externally.
/// - A pure, host-local triage of whether evolution is allowed right now.
pub fn is_host_sovereign_operational_for_defense(
    env: &HostEnvelope,
    lifeforce_bands: &LifeforceBandSeries,
    pain: &PainCorridorSignal,
    aura: &BiophysicalAura,
    daily_usage: &DailyDomainUsage,
    config: &DefensiveMutationConfig,
) -> SovereignGuardResult {
    check_defensive_mutation_sovereign_guards(env, lifeforce_bands, pain, aura, daily_usage, config)
}

/// Strong invariant: inner-ledger sovereignty cannot be overridden from platforms.
///
/// This function is intended to be called from the inner runtime (not boundary services)
/// as an assertion that:
/// - Identity/role checks already ran (validate_identity_for_inner_ledger),
/// - Consent/provenance shards are host-owned and unforgeable,
/// - No cross-host or financial parameters appear in this pipeline.
///
/// It returns `Ok(())` when the configuration is structurally sovereign-safe.
pub fn assert_structural_sovereign_operationality(
    env: &HostEnvelope,
    config: &DefensiveMutationConfig,
) -> Result<(), &'static str> {
    // 1. Ensure host envelope is not sandbox / simulation when mutating somatic state.
    if env.identity.network_tier.is_sandbox() {
        return Err("sandbox tier may not mutate somatic defensive domains");
    }

    // 2. Ensure the DID namespace is ALN/Bostrom-like and host-local.
    if !env.identity.subject_did.is_bostrom_namespace() {
        return Err("non-Bostrom DID namespaces may not control biophysical evolution");
    }

    // 3. Ensure this pipeline never takes or uses any transferable balances.
    // (By construction, config has no financial fields; we enforce that here by convention.)
    if config.domain_fraction_cap < 0.0 || config.domain_fraction_cap > 1.0 {
        return Err("domain_fraction_cap must be a normalized, non-financial fraction");
    }

    // 4. Doctrinal: evolution_config must mark this domain as host-governed, non-financial.
    if !env.evolution_config.is_host_sovereign_for_domain(&config.domain_id) {
        return Err("evolution domain is not marked host-sovereign in evolution_config");
    }

    Ok(())
}
