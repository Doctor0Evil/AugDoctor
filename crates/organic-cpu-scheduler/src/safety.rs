use crate::snapshot::OrganicCpuSnapshot;
use crate::budget::HostBudget;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobDecision {
    Permit,
    Defer(String),
    Deny(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThermodynamicEnvelope {
    pub max_delta_core_c: f32,
    pub max_core_c: f32,
    pub max_neuromorph_duty: f32,
    pub max_chat_duty: f32,
    pub max_energy_joules: f32,
    pub max_inflammation_index: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphJobCost {
    pub estimated_energy_joules: f32,
    pub estimated_duty_neuromorph: f32,
    pub estimated_delta_core_c: f32,
}

pub fn decide_organic_job(
    snap: &OrganicCpuSnapshot,
    budget: &HostBudget,
    envelope: &ThermodynamicEnvelope,
    cost: &NeuromorphJobCost,
) -> JobDecision {
    // Thermal hard stops
    if snap.core_temp_c + cost.estimated_delta_core_c > envelope.max_core_c {
        return JobDecision::Deny("core_temp_ceiling".into());
    }
    if cost.estimated_delta_core_c > envelope.max_delta_core_c {
        return JobDecision::Deny("delta_core_c_ceiling".into());
    }

    // Energy budget
    if cost.estimated_energy_joules > budget.energy_joules_remaining {
        return JobDecision::Defer("insufficient_energy_budget".into());
    }
    if cost.estimated_energy_joules > envelope.max_energy_joules {
        return JobDecision::Deny("thermo_envelope_energy_violation".into());
    }

    // Duty cycles
    if snap.duty_fraction_neuromorph + cost.estimated_duty_neuromorph
        > budget.max_neuromorph_duty_fraction
    {
        return JobDecision::Defer("neuromorph_duty_ceiling".into());
    }

    if snap.duty_fraction_chat > budget.max_chat_duty_fraction {
        return JobDecision::Defer("chat_duty_ceiling".into());
    }

    // Inflammation / recovery
    if snap.inflammation_index > envelope.max_inflammation_index {
        return JobDecision::Defer("inflammation_high".into());
    }

    JobDecision::Permit
}
