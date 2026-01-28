use crate::lifeforce::{HostVitals, DecayState};
use crate::tokens::{RadsToken, BrainToken};
use log::{error, warn, info};

/// Defines immutable safety constants — not user-modifiable
pub const RADS_BIOPHYSICAL_MAX: f64 = 0.0021;  // sieverts per cycle
pub const RADS_SAFE_OPERATING_MARGIN: f64 = 0.0017;

/// Core safety structure enforcing !#mutation!rules
pub struct MutationGuard;

impl MutationGuard {
    /// Check and enforce protection limits against radiation mutation
    pub fn enforce(host: &mut HostVitals, rads: &mut RadsToken, decay: &DecayState) -> bool {
        let applied_rads = rads.value_after_decay(decay);

        if applied_rads > RADS_BIOPHYSICAL_MAX {
            error!(
                "[!] MutationRiskDetected: Exposure {:.6} exceeds biophysical-maximum ({:.6}) for host {}",
                applied_rads, RADS_BIOPHYSICAL_MAX, host.id
            );
            Self::trigger_termination(host, rads);
            return false;
        }

        if applied_rads > RADS_SAFE_OPERATING_MARGIN {
            warn!(
                "[!] Warning: radiation near upper-safe-limit {:.6}. Minimizing exposure...",
                applied_rads
            );
            Self::minimize_operation(host);
        }

        info!("[OK] Radiation exposure safe: {:.6}", applied_rads);
        true
    }

    /// Cancel or terminate procedure when mutation threshold breached
    fn trigger_termination(host: &mut HostVitals, rads: &mut RadsToken) {
        rads.revoke();
        host.set_evolution_state("CANCELLED_DUE_TO_RADIATION");
        host.push_audit("RADIATION_TERMINATION_TRIGGERED", "Auto-safe enforced");
    }

    /// Shorten or minimize high-exposure procedure
    fn minimize_operation(host: &mut HostVitals) {
        host.reduce_operational_cycle(0.4);
        host.rebalance_cognitive_load();
        host.flag("RADIATION_MINIMIZED");
    }
}
