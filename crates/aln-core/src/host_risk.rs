use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HostRiskWeights {
    pub w_e: f64,
    pub w_t: f64,
    pub w_d: f64,
    pub w_c: f64,
    pub w_n: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HostRiskComponents {
    pub e: f64,
    pub t: f64,
    pub d: f64,
    pub c: f64,
    pub n: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HostRiskScalar {
    pub v_host: f64,
    pub components: HostRiskComponents,
}

impl HostRiskScalar {
    pub fn from_components(weights: HostRiskWeights, components: HostRiskComponents) -> Self {
        let v_host = weights.w_e * components.e
            + weights.w_t * components.t
            + weights.w_d * components.d
            + weights.w_c * components.c
            + weights.w_n * components.n;
        Self {
            v_host,
            components,
        }
    }

    pub fn is_monotone_non_increasing(self, next: HostRiskScalar) -> bool {
        next.v_host <= self.v_host + f64::EPSILON
    }

    pub fn has_strict_improvement(self, next: HostRiskScalar) -> bool {
        next.components.e < self.components.e
            || next.components.t < self.components.t
            || next.components.d < self.components.d
            || next.components.c < self.components.c
            || next.components.n < self.components.n
    }
}
