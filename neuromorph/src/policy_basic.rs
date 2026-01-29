use super::orchestrator::{ReflexMicroPolicy, ReflexProposal, ReflexActionDomain};
use super::observation::{NeuromorphEvent, NeuromorphEventKind};
use biophysical_blockchain::{BioTokenState, SystemAdjustment};

pub struct BasicReflexPolicy;

impl BasicReflexPolicy {
    fn mk_adjustment(
        &self,
        deltabrain: f64,
        deltawave: f64,
        deltanano: f64,
        deltasmart: f64,
        reason: &str,
    ) -> SystemAdjustment {
        SystemAdjustment {
            deltabrain,
            deltawave,
            deltablood: 0.0,
            deltaoxygen: 0.0,
            deltanano,
            deltasmart,
            ecocost: 0.0001,
            reason: reason.to_string(),
        }
    }
}

impl ReflexMicroPolicy for BasicReflexPolicy {
    fn decide(&self, state: &BioTokenState, event: &NeuromorphEvent) -> Option<ReflexProposal> {
        match &event.kind {

            // Reflex Safety: overload → clamp WAVE/SMART slightly.
            NeuromorphEventKind::SystemOverload { error_rate } if *error_rate > 0.2 => {
                let deltawave  = -0.01_f64.max(-0.05 * (*error_rate as f64));
                let deltasmart = -0.01_f64;
                let adj = self.mk_adjustment(0.0, deltawave, 0.0, deltasmart,
                                             "neuromorph-reflex-safety-overload");
                Some(ReflexProposal {
                    domain: ReflexActionDomain::ReflexScheduling,
                    adjustment: adj,
                })
            }

            // Sensory Clarity: stable posture + clean EMG/EEG → tiny BRAIN reward.
            NeuromorphEventKind::SystemRecovery { pattern_id } => {
                let adj = self.mk_adjustment(0.001, 0.0, 0.0, 0.0,
                                             &format!("neuromorph-sense-clarity-{}", pattern_id));
                Some(ReflexProposal {
                    domain: ReflexActionDomain::SensoryClarity,
                    adjustment: adj,
                })
            }

            // Attention / Load Balance: repeated motion bursts + rising error → rebalance WAVE.
            NeuromorphEventKind::MotionBurst { magnitude, .. } if *magnitude > 0.7 => {
                // Very small negative WAVE to non-critical tasks can be encoded
                // indirectly as small positive BRAIN when stability improves later.
                let deltawave = -0.005_f64;
                let adj = self.mk_adjustment(0.0005, deltawave, 0.0, 0.0,
                                             "neuromorph-attention-balance-motion");
                Some(ReflexProposal {
                    domain: ReflexActionDomain::AttentionBalance,
                    adjustment: adj,
                })
            }

            _ => None,
        }
    }
}
