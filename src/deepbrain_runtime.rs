pub struct PolicySnapshot {
    pub probs_old: Vec<f64>,
    pub probs_new: Vec<f64>,
}

fn safe_kl(p_old: &[f64], p_new: &[f64]) -> f64 {
    p_old.iter().zip(p_new.iter())
        .map(|(o, n)| {
            let o = o.max(1e-12);
            let n = n.max(1e-12);
            o * (o / n).ln()
        })
        .sum()
}

/// Evolution step size governor f(D_KL, c, B).
pub fn compute_evolution_step(d_kl: f64, confidence: f64, budget: f64) -> f64 {
    let x = (budget - d_kl).max(-4.0).min(4.0);
    let sigma = 1.0 / (1.0 + (-x).exp());
    let c = confidence.max(0.0).min(1.0);
    sigma * c
}
