#[derive(Clone, Debug)]
pub struct IdentityDriftState {
    pub z_latent: [f64; 4],        // slow “who I am” vector
    pub kl_step: f64,             // KL(old_policy || new_policy) for last tick
    pub kl_drift_cum_day: f64,    // accumulated identity drift for current day
    pub kl_budget_per_day: f64,   // neurorights floor
    pub safety_risk_cum: f64,     // Hoeffding-bound cumulative risk
    pub safety_risk_ceiling: f64, // neurorights risk ceiling
}

#[derive(Clone, Debug)]
pub struct HostEnvelope {
    pub hostid: String,
    pub brain_min: f64,
    pub blood_min: f64,
    pub oxygen_min: f64,
    pub nano_max_fraction: f64,
    pub smart_max: f64,
    pub eco_flops_limit: f64,
    pub identity: IdentityDriftState, // deep‑brain guard
}
