use crate::governance::host_rights_travel_us::{HostRightsTravelUsProfile, HostRightsStatus};
use crate::telemetry::shard_loader::load_host_rights_travel_us; // your existing shard loader

pub fn enforce_host_rights_at_startup(expected_host_id: &str) {
    let profile: HostRightsTravelUsProfile =
        load_host_rights_travel_us(expected_host_id)
            .expect("host-rights-travel-us shard must be present");

    match profile.verify_rights_safe(expected_host_id) {
        HostRightsStatus::RightsSafe => {
            // OK: node can continue to boot.
        }
        HostRightsStatus::ViolatesInvariant(errs) => {
            panic!(
                "Host rights profile violates invariants, refusing to start:\n{}",
                errs.join("\n")
            );
        }
    }
}
