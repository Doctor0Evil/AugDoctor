use crate::governance::host_rights_travel_us::{HostRightsTravelUsProfile, HostRightsStatus};
use crate::telemetry::shard_loader::load_host_rights_travel_us;

pub fn enforce_host_rights_travel_us(expected_host_id: &str) {
    let profile: HostRightsTravelUsProfile =
        load_host_rights_travel_us(expected_host_id)
            .expect("host-rights-travel-us-maricopa.aln must be present");

    match profile.verify_rights_safe(expected_host_id) {
        HostRightsStatus::RightsSafe => {}
        HostRightsStatus::ViolatesInvariant(errs) => {
            panic!(
                "Host rights travel profile violates invariants; refusing to start:\n{}",
                errs.join("\n")
            );
        }
    }
}
