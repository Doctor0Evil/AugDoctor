pub struct SovereigntyFlags {
    pub sovereignty_core_enabled: bool,
    pub rollback_requested: bool,
    pub rollback_ticks: u32,
    pub rollback_timeout_ticks: u32,
}

impl SovereigntyFlags {
    pub fn assert_invariants(&self, host: &HostEnvelope) -> Result<(), &'static str> {
        if !self.sovereignty_core_enabled {
            return Err("Sovereignty core must never be disabled");
        }
        if self.rollback_requested && self.rollback_ticks > self.rollback_timeout_ticks {
            return Err("Rollback did not complete within allowed ticks");
        }
        if host.identity.kl_drift_cum_day > host.identity.kl_budget_per_day {
            return Err("Identity drift invariant violated");
        }
        Ok(())
    }
}
