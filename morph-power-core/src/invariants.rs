use serde::{Deserialize, Serialize};

use crate::types::{MorphVector, MorphBudgetCorridorSpec, MorphUsage, MorphEvidenceBundle};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MorphSafetyError {
    ExceedsEvolveScalar { l1_norm: f32, evolve_scalar: f32 },
    ExceedsPerDimensionCap { dim: &'static str, value: f32, cap: f32 },
    MissingEvidence,
    InsufficientEvidenceTags { have: usize, required: usize },
}

/// Internal helper to compute ∥M∥₁.
fn l1_norm(m: &MorphVector) -> f32 {
    (m.eco.abs() + m.cyber.abs() + m.neuro.abs() + m.smart.abs())
}

/// Sealed corridor object, not directly constructible by external crates.
#[derive(Clone, Debug)]
pub struct MorphBudgetCorridor {
    spec: MorphBudgetCorridorSpec,
}

impl MorphBudgetCorridor {
    pub fn new(spec: MorphBudgetCorridorSpec) -> Self {
        Self { spec }
    }

    /// Check that a proposed MORPH vector lies inside the host's EVOLVE corridor.
    pub fn check_vector(&self, m: &MorphVector) -> Result<(), MorphSafetyError> {
        let n = l1_norm(m);
        if n > self.spec.evolve_scalar {
            return Err(MorphSafetyError::ExceedsEvolveScalar {
                l1_norm: n,
                evolve_scalar: self.spec.evolve_scalar,
            });
        }
        if m.eco > self.spec.max_eco {
            return Err(MorphSafetyError::ExceedsPerDimensionCap {
                dim: "eco",
                value: m.eco,
                cap: self.spec.max_eco,
            });
        }
        if m.cyber > self.spec.max_cyber {
            return Err(MorphSafetyError::ExceedsPerDimensionCap {
                dim: "cyber",
                value: m.cyber,
                cap: self.spec.max_cyber,
            });
        }
        if m.neuro > self.spec.max_neuro {
            return Err(MorphSafetyError::ExceedsPerDimensionCap {
                dim: "neuro",
                value: m.neuro,
                cap: self.spec.max_neuro,
            });
        }
        if m.smart > self.spec.max_smart {
            return Err(MorphSafetyError::ExceedsPerDimensionCap {
                dim: "smart",
                value: m.smart,
                cap: self.spec.max_smart,
            });
        }
        Ok(())
    }

    /// Enforce MORPH ≤ EVOLVE for a proposed upgrade consumption.
    ///
    /// `host_morph` is the current MORPH state for the host.
    pub fn check_upgrade(
        &self,
        host_morph: &MorphVector,
        usage: &MorphUsage,
        evidence: &MorphEvidenceBundle,
    ) -> Result<MorphVector, MorphSafetyError> {
        // 1. Evidence gate: enforce 10-tag EvidenceBundle.
        if evidence.tags.is_empty() || evidence.values.is_empty() {
            return Err(MorphSafetyError::MissingEvidence);
        }
        if evidence.tags.len().min(evidence.values.len()) < 10 {
            return Err(MorphSafetyError::InsufficientEvidenceTags {
                have: evidence.tags.len().min(evidence.values.len()),
                required: 10,
            });
        }

        // 2. Compute new MORPH vector after delta.
        let next = MorphVector {
            eco: host_morph.eco + usage.delta_morph.eco,
            cyber: host_morph.cyber + usage.delta_morph.cyber,
            neuro: host_morph.neuro + usage.delta_morph.neuro,
            smart: host_morph.smart + usage.delta_morph.smart,
        };

        // 3. Enforce MORPH ≤ EVOLVE and per-dimension caps.
        self.check_vector(&next)?;
        Ok(next)
    }
}
