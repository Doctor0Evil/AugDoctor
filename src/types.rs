#[derive(Clone, Debug)]
pub struct SystemAdjustment {
    pub delta_brain: f64,
    pub delta_wave: f64,
    pub delta_blood: f64,
    pub delta_oxygen: f64,
    pub delta_nano: f64,
    pub delta_smart: f64,
    pub eco_cost: f64,
    // deep‑brain metadata
    pub kl_step: f64,
    pub risk_increment: f64,
}
