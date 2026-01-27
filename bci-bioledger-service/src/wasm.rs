#![cfg(feature = "wasm")]

use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use bci_bioledger_bridge::{BciEvent, BciLedgerOrchestrator};
use biophysical_blockchain::{
    BioTokenState, HostEnvelope, IdentityHeader, InnerLedger,
};
use bioscaleupgradeservice::neuralrope::NeuralRope;
use augdoctorpolicies::neurohandshakeorchestrator::NeuroHandshakeState;

static LEDGER: OnceLock<Mutex<InnerLedger>> = OnceLock::new();
static ROPE: OnceLock<Mutex<NeuralRope>> = OnceLock::new();

fn init_globals() {
    LEDGER.get_or_init(|| {
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
        Mutex::new(InnerLedger::new(env, state))
    });
    ROPE.get_or_init(|| Mutex::new(NeuralRope::new("bci-hci-eeg".to_string())));
}

#[derive(Debug, Deserialize)]
struct WasmBciRequest {
    pub identity: IdentityHeader,
    pub event: BciEvent,
    pub required_knowledge_factor: f32,
    pub timestamp_utc: String,
}

#[derive(Debug, Serialize)]
struct WasmBciResponse {
    pub applied: bool,
    pub reason: String,
    pub prev_state_hash: Option<String>,
    pub new_state_hash: Option<String>,
}

#[wasm_bindgen]
pub fn bci_apply_json(req: &str) -> String {
    init_globals();

    let parsed: Result<WasmBciRequest, _> = serde_json::from_str(req);
    let req = match parsed {
        Ok(v) => v,
        Err(e) => {
            return format!(r#"{{"error":"invalid_json","detail":"{}"}}"#, e);
        }
    };

    let ledger_mutex = LEDGER.get().unwrap();
    let rope_mutex = ROPE.get().unwrap();

    let mut ledger = ledger_mutex.lock().unwrap();
    let mut rope = rope_mutex.lock().unwrap();

    let mut orchestrator = BciLedgerOrchestrator::new(&mut *ledger, &mut *rope);
    let handshake = NeuroHandshakeState::new(req.event.session_id.clone());

    let result = orchestrator.handle_bci_event(
        &req.event,
        handshake,
        &req.identity,
        req.required_knowledge_factor,
        &req.timestamp_utc,
    );

    match result {
        Ok((ledger_result, _hs, _event, _shot)) => {
            let resp = WasmBciResponse {
                applied: ledger_result.applied,
                reason: ledger_result.reason,
                prev_state_hash: ledger_result.prev_state_hash,
                new_state_hash: ledger_result.new_state_hash,
            };
            serde_json::to_string(&resp)
                .unwrap_or_else(|e| format!(r#"{{"error":"serialization","detail":"{}"}}"#, e))
        }
        Err(e) => format!(
            r#"{{"applied":false,"error":"{}"}}"#,
            format!("{:?}", e)
        ),
    }
}
