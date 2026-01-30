impl InnerLedger {
    pub fn system_apply(
        &mut self,
        id_header: IdentityHeader,
        required_k: f32,
        adj: SystemAdjustment,
        policy: PolicySnapshot,
        confidence: f64,
        timestamp_utc: &str,
    ) -> Result<LedgerEvent, InnerLedgerError> {
        validate_identity_for_inner_ledger(id_header, required_k)?;

        // self‑only envelope; hostid is not taken from adj
        let step = self.env.apply_identity_drift(&policy, confidence, adj.risk_increment)
            .map_err(|e| InnerLedgerError::DeepBrainViolation(e.to_string()))?;

        // scale deltas by neurorights‑bounded evolution step
        let scaled = SystemAdjustment {
            delta_brain: adj.delta_brain * step,
            delta_wave: adj.delta_wave * step,
            delta_blood: adj.delta_blood * step,
            delta_oxygen: adj.delta_oxygen * step,
            delta_nano: adj.delta_nano * step,
            delta_smart: adj.delta_smart * step,
            eco_cost: adj.eco_cost,
            kl_step: adj.kl_step,
            risk_increment: adj.risk_increment,
        };

        apply_lifeforce_guarded_adjustment(&mut self.state, &self.env, scaled)?;
        // hash + event construction as in your existing InnerLedger
        // ...
        Ok(event)
    }
}
