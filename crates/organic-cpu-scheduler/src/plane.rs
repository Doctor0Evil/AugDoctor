use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HostPlane {
    OrganicCpuSoftwareOnly,
    OrganicCpuWithWearables,
    BciHciEeg,          // optional
    NanoswarmAttached,  // optional
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostPlaneConfig {
    pub plane: HostPlane,
    // Whether this plane MAY influence evolution; never required.
    pub may_contribute_telemetry: bool,
    // Whether this plane is required for a given upgrade; default false.
    pub required_for_upgrade: bool,
}
