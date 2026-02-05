use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostBudget {
    // Energy in Joules for this evolution window
    pub energy_joules_remaining: f32,
    // Available amino acid / protein budget as normalized fraction 0..1
    pub protein_budget_fraction: f32,
    // Cognitive duty cycle ceilings 0..1
    pub max_chat_duty_fraction: f32,
    pub max_neuromorph_duty_fraction: f32,
}
