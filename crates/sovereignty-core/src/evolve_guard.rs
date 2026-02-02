pub fn approve_evolve_step(
    proposal: EvolveProposal,
    donut: &mut DonutloopSink,
    biostate: &BioStateSnapshot,
    envelopes: &PersonalEnvelopes,
) -> Decision {
    // 1. Check global RoH, universal adult floor, eco bands
    if !roh_within_ceiling(&proposal, biostate) { 
        return donut.reject(proposal, "RoH-ceiling"); 
    }
    if !universal_adult_floor_ok(biostate) {
        return donut.reject(proposal, "UniversalAdultFloor");
    }
    if !eco_budget_ok(&proposal) {
        return donut.reject(proposal, "EcoBudget");
    }

    // 2. Evaluate personal envelopes and INSTINCT veto
    let envelope_event = evaluate_personal_envelopes(&proposal, biostate, envelopes);
    if envelope_event.instinct_vetoed {
        return donut.log_with_envelope(proposal, "REJECTED", envelope_event);
    }

    // 3. Require EVOLVE multisig when explicit_spend or high band
    if envelope_event.explicit_spend || envelope_event.approaching_threshold {
        if !has_multisig_evolve(&proposal) {
            return donut.log_with_envelope(proposal, "DEFERRED", envelope_event);
        }
    }

    // 4. All guards satisfied: apply and log ALLOWED
    apply_to_inner_ledger(&proposal);
    donut.log_with_envelope(proposal, "ALLOWED", envelope_event)
}
