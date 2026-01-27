use biophysical_blockchain::{
    BioTokenState, HostEnvelope, IdentityHeader, InnerLedger,
};
use bci_bioledger_bridge::{
    BciEvent, BciLedgerOrchestrator,
};
use bioscaleupgradeservice::neuralrope::NeuralRope;
use augdoctorpolicies::neurohandshakeorchestrator::NeuroHandshakeState;

fn main() -> anyhow::Result<()> {
    // Host envelope and initial state (per-host, non-financial).
    let env = HostEnvelope {
        host_id: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
        brain_min: 0.0,
        blood_min: 0.2,
        oxygen_min: 0.9,
        nano_max_fraction: 0.25,
        smart_max: 1.0,
        eco_flops_limit: 10_000.0,
    };

    let state = BioTokenState {
        brain: 0.5,
        wave: 0.0,
        blood: 0.8,
        oxygen: 0.97,
        nano: 0.0,
        smart: 0.0,
    };

    let mut ledger = InnerLedger::new(env, state);
    let mut rope = NeuralRope::new("bci-hci-eeg".to_string());

    let mut orchestrator = BciLedgerOrchestrator::new(&mut ledger, &mut rope);

    // Example BCI event.
    let event = BciEvent {
        session_id: "session-001".to_string(),
        host_id: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
        environment_id: "phx-lab".to_string(),
        channel: "eeg".to_string(),
        intent_label: "cursor-move".to_string(),
        risk_score: 0.1,
        latency_budget_ms: 200,
        token_budget: 512,
        eco_cost_estimate: 50.0,
    };

    // ALN/DID identity header for the host daemon.
    let id_header = IdentityHeader {
        issuer_did: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
        subject_role: "system-daemon".to_string(),
        network_tier: "inner-core".to_string(),
        knowledge_factor: 0.9,
    };

    // Start handshake in Safety phase.
    let handshake = NeuroHandshakeState::new("session-001".to_string());

    // Required knowledge threshold, and timestamp.
    let required_k = 0.6;
    let timestamp_utc = "2026-01-27T19:00:00Z";

    let (result, _new_handshake, _ledger_event, _shot_decision) =
        orchestrator
            .handle_bci_event(&event, handshake, &id_header, required_k, timestamp_utc)
            .expect("BCI → ledger application failed");

    println!("BCI ledger result: {:?}", result);
    println!("New state hash: {}", orchestrator.ledger.last_state_hash);

    Ok(())
}
