use crate::domain::{NeuromorphPlane, NeuromorphDomain, NeuromorphPlaneTag};
use crate::observation::{NeuromorphEvent, NeuromorphEventKind};
use biophysical_blockchain::{SystemAdjustment, BioTokenState, HostEnvelope, applylifeforceguardedadjustment};

#[derive(Clone, Debug)]
pub enum ReflexActionDomain {
    ReflexScheduling,   // maps to neuromorph-reflex-micro
    SensoryClarity,     // neuromorph-sense-micro
    AttentionBalance,   // neuromorph-attention-micro
}

#[derive(Clone, Debug)]
pub struct ReflexProposal {
    pub domain: ReflexActionDomain,
    pub adjustment: SystemAdjustment,
}

pub trait ReflexMicroPolicy {
    fn decide(&self, state: &BioTokenState, event: &NeuromorphEvent) -> Option<ReflexProposal>;
}

pub struct ReflexMicroOrchestrator<P: ReflexMicroPolicy> {
    pub policy: P,
}

impl<P: ReflexMicroPolicy> ReflexMicroOrchestrator<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }

    /// Shadow-mode execution: compute but do not apply.
    pub fn shadow_step(&self, state: &BioTokenState, event: &NeuromorphEvent)
        -> Option<ReflexProposal>
    {
        self.policy.decide(state, event)
    }

    /// Live execution: propose tiny adjustments through existing lifeforce guards.
    pub fn live_step(
        &self,
        state: &mut BioTokenState,
        env: &HostEnvelope,
        event: &NeuromorphEvent,
    ) -> Option<Result<(), LifeforceError>> {
        let proposal = self.policy.decide(state, event)?;
        let mut adj = proposal.adjustment.clone();
        // Ensure adjustments are very small and bounded.
        adj.deltabrain *= 0.25;
        adj.deltawave  *= 0.25;
        adj.deltanano  *= 0.25;
        adj.deltasmart *= 0.25;

        Some(applylifeforceguardedadjustment(state, env, adj))
    }
}
