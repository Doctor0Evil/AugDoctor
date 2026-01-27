use crate::mapper::map_bci_to_adjustment;
use crate::types::{BciEvent, BciLedgerResult};
use augdoctorpolicies::neurohandshakeorchestrator::{
    HandshakePhase, NeuroHandshakeOrchestrator, NeuroHandshakeState,
};
use augdoctorpolicies::shotlevelpolicy::{ShotLevel, ShotLevelDecision, ShotLevelPolicy,
    ShotLevelPolicyConfig, ShotLevelSignal};
use bioscaleupgradeservice::neuralrope::NeuralRope;
use biophysical_blockchain::{
    IdentityHeader, InnerLedger, InnerLedgerError, LedgerEvent,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("inner-ledger error: {0}")]
    Inner(#[from] InnerLedgerError),
    #[error("handshake not yet in operation phase")]
    NotReady,
}

/// BCI → inner-ledger orchestrator.
/// One instance per host / process; NeuralRope lives alongside for learning traces.[file:1]
pub struct BciLedgerOrchestrator<'a> {
    pub ledger: &'a mut InnerLedger,
    pub rope: &'a mut NeuralRope,
    pub shot_policy: ShotLevelPolicy,
}

impl<'a> BciLedgerOrchestrator<'a> {
    pub fn new(ledger: &'a mut InnerLedger, rope: &'a mut NeuralRope) -> Self {
        let cfg = ShotLevelPolicyConfig {
            maxexamplesfewshot: 4,
            riskthresholdforfewshot: 0.5,
            errorratethresholdforfewshot: 0.2,
            minlatencyforfewshotms: 250,
            mintokenbudgetforfewshot: 512,
        };
        let shot_policy = ShotLevelPolicy::new(cfg);
        Self {
            ledger,
            rope,
            shot_policy,
        }
    }

    /// Handle one BCI event: handshake → shot decision (for your LLM call) →
    /// map to SystemAdjustment → apply to inner ledger under lifeforce guards.[file:1]
    pub fn handle_bci_event(
        &mut self,
        event: &BciEvent,
        mut handshake: NeuroHandshakeState,
        id_header: &IdentityHeader,
        required_k: f32,
        timestamp_utc: &str,
    ) -> Result<(BciLedgerResult, NeuroHandshakeState, Option<LedgerEvent>, ShotLevelDecision), BridgeError> {
        // 1. Neuro-handshake progression.
        let actions = NeuroHandshakeOrchestrator::next_actions(handshake.clone());
        if handshake.phase == HandshakePhase::Safety {
            // In real system: UI already collected consent.
            if !handshake.safety_confirmed {
                handshake = NeuroHandshakeOrchestrator::apply_event(handshake, "user-consented");
            }
        }
        if handshake.phase == HandshakePhase::Calibration {
            // In real system: calibration samples collected per event.
            handshake =
                NeuroHandshakeOrchestrator::apply_event(handshake, "calibration-sample-recorded");
        }
        if handshake.phase != HandshakePhase::Operation {
            // Not ready yet; we return NotReady with no ledger mutation.
            let res = BciLedgerResult {
                session_id: event.session_id.clone(),
                host_id: event.host_id.clone(),
                intent_label: event.intent_label.clone(),
                applied: false,
                reason: format!("handshake-phase:{:?}-actions:{:?}", handshake.phase, actions),
                prev_state_hash: Some(self.ledger.last_state_hash.clone()),
                new_state_hash: None,
            };
            return Err(BridgeError::NotReady);
        }

        // 2. Decide zero-shot vs few-shot for LLM side (if you wire it).
        let signal = ShotLevelSignal {
            taskid: format!("bci-{}", event.intent_label),
            planelabel: "bci-hci-eeg".to_string(),
            riskscore: event.risk_score,
            latencybudgetms: event.latency_budget_ms,
            tokenbudget: event.token_budget,
            historicalerrorrate: 0.05,
            requiresexamples: event.intent_label.contains("fine-grip"),
        };
        let shot_decision = self.shot_policy.decide(signal);

        // 3. Map BCI event to SystemAdjustment.
        let adj = map_bci_to_adjustment(event);

        // 4. Apply to inner ledger under access + lifeforce guards.
        let prev_hash = self.ledger.last_state_hash.clone();
        let ledger_event = self
            .ledger
            .system_apply(id_header, required_k, &adj, timestamp_utc)?;

        // 5. Log into neural-rope for future few-shot mining.
        let planelabel = "bci-hci-eeg".to_string();
        let safetydecision = "Allow".to_string();
        let trace_text = format!(
            "BCI-LEDGER env={} intent={} delta_brain={:.6} delta_wave={:.6} delta_nano={:.6} eco_cost={:.3}",
            event.environment_id, event.intent_label, adj.delta_brain, adj.delta_wave, adj.delta_nano, adj.eco_cost
        );
        self.rope.append_trace(
            trace_text,
            planelabel.clone(),
            Some(event.intent_label.clone()),
            1.0,
            safetydecision.clone(),
        );

        // 6. Build result.
        let res = BciLedgerResult {
            session_id: event.session_id.clone(),
            host_id: event.host_id.clone(),
            intent_label: event.intent_label.clone(),
            applied: true,
            reason: "lifeforce-guarded-adjustment-applied".to_string(),
            prev_state_hash: Some(prev_hash),
            new_state_hash: Some(ledger_event.new_state_hash.clone()),
        };

        Ok((res, handshake, Some(ledger_event), shot_decision))
    }
}
