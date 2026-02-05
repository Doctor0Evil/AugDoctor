use serde::{Deserialize, Serialize};
use crate::sealed::inner::Sealed;

/// Scalar EVOLVE budget per host (already present conceptually).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolveBudget {
    pub evolve_total: f64,   // Eevolve ≥ 0, dimensionless corridor
    pub evolve_used: f64,    // consumed this UTC day
}

/// MORPH vector: constrained neuromorph/cybernetic/SMART slice of EVOLVE.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MorphBudget {
    pub m_eco:   f64,  // eco upgrades
    pub m_cyber: f64,  // cybernetic influence
    pub m_neuro: f64,  // neuromorph kernels
    pub m_smart: f64,  // SMART autonomy complexity
}

impl Sealed for MorphBudget {}

impl MorphBudget {
    /// L1 norm of MORPH vector ‖M‖₁.
    pub fn l1_norm(&self) -> f64 {
        self.m_eco + self.m_cyber + self.m_neuro + self.m_smart
    }

    /// Component‑wise add a proposed delta (can be negative for tightening).
    pub fn plus(&self, d: &MorphDelta) -> Self {
        Self {
            m_eco:   (self.m_eco   + d.d_eco  ).max(0.0),
            m_cyber: (self.m_cyber + d.d_cyber).max(0.0),
            m_neuro: (self.m_neuro + d.d_neuro).max(0.0),
            m_smart: (self.m_smart + d.d_smart).max(0.0),
        }
    }
}

/// Proposed MORPH change attached to upgrades / mutations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MorphDelta {
    pub d_eco:   f64,
    pub d_cyber: f64,
    pub d_neuro: f64,
    pub d_smart: f64,
}

/// Risk directions: fields that may only tighten without new evidence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MorphRiskBands {
    pub max_cyber: f64,  // upper bound on M_cyber
    pub max_neuro: f64,  // upper bound on M_neuro
    pub max_smart: f64,  // upper bound on M_smart
}
