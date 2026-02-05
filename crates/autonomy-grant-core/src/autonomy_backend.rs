use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HardwareBackendKind {
    OrganicCpu,
    NeuromorphicTile,
    Gpu,
    Loihi,
    BciCapableDevice,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyBackendProfile {
    pub backend: HardwareBackendKind,
    pub preferred: bool,
    pub degraded_safe_mode_available: bool,
}
