use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganicCpuSnapshot {
    pub host_id: String,
    pub captured_at: SystemTime,

    // Cardiovascular / stress
    pub hrv_ms: f32,           // heart rate variability (ms)
    pub resting_hr_bpm: f32,

    // Thermal
    pub core_temp_c: f32,
    pub skin_temp_c: f32,

    // Inflammation / recovery proxies (normalized 0..1 from lab reports / self-report)
    pub inflammation_index: f32,
    pub protein_availability_index: f32,

    // Cognitive workload proxies (0..1)
    pub perceived_fatigue: f32,
    pub perceived_cognitive_load: f32,

    // Duty and evolution pacing
    pub duty_fraction_chat: f32,      // fraction of time in AI-assisted tasks
    pub duty_fraction_neuromorph: f32,
}
