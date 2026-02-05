pub fn evaluate_upgrade(
    host_state:   &BioTokenState,
    descriptor:   &UpgradeDescriptor,
    host_budget:  &HostBudget,
    thermo_env:   &ThermodynamicEnvelope,
    morph_risk:   &MorphRiskBands,
) -> UpgradeDecision {
    // 1. HostBudget / thermodynamic envelope
    if !host_budget.allows(descriptor, thermo_env) {
        return UpgradeDecision::Denied("host budget or thermodynamic envelope".into());
    }

    // 2. EVOLVE scalar requirement
    let evolve_remaining =
        host_state.evolve.evolve_total - host_state.evolve.evolve_used;
    if descriptor.delta_morph.l1_norm() > evolve_remaining {
        return UpgradeDecision::Denied("not enough EVOLVE to cover MORPH slice".into());
    }

    // 3. MORPH corridor requirement
    let m  = &host_state.morph;
    let rm = &descriptor.required_morph;
    if m.m_eco   < rm.m_eco
        || m.m_cyber < rm.m_cyber
        || m.m_neuro < rm.m_neuro
        || m.m_smart < rm.m_smart
    {
        return UpgradeDecision::Denied("MORPH corridor insufficient for this upgrade".into());
    }

    // 4. Risk monotonicity (cyber, neuromorph, SMART)
    let proposed_after = m.plus(&descriptor.delta_morph);
    if proposed_after.m_cyber > morph_risk.max_cyber
        || proposed_after.m_neuro > morph_risk.max_neuro
        || proposed_after.m_smart > morph_risk.max_smart
    {
        return UpgradeDecision::Denied("MORPH risk bands exceeded".into());
    }

    UpgradeDecision::Allowed
}
