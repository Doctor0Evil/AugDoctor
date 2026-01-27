#![cfg(feature = "wasm")]

use crate::security::{classify_civic, civic_reward_multiplier, AuthEnvelope, CivicClass};
use bci_bioledger_bridge::{BciEvent, BciLedgerOrchestrator};
use biophysical_blockchain::{BioTokenState, HostEnvelope, InnerLedger};
use bioscaleupgradeservice::neuralrope::NeuralRope;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

static LEDGER: once_cell::sync::OnceCell<Mutex<InnerLedger>> = once_cell::sync::OnceCell::new();
static ROPE: once_cell::sync::OnceCell<Mutex<NeuralRope>> = once_cell::sync::OnceCell::new();

fn init_globals() {
    LEDGER.get_or_init(|| Mutex::new(super::init_ledger()));
    ROPE.get_or_init(|| Mutex::new(NeuralRope::new()));
}

#[derive(Debug, Deserialize)]
struct WasmApplyRequest {
    pub auth: AuthEnvelope,
    pub bci_event: BciEvent,
    pub civic_tags: Vec<String>,
    pub timestamp_utc: String,
}

#[derive(Debug, Serialize)]
struct WasmApplyResponse {
    pub result: Option<bci_bioledger_bridge::BciLedgerResult>,
    pub civic_class: CivicClass,
    pub reward_multiplier: f64,
    pub error: Option<String>,
}

#[wasm_bindgen]
pub fn wasm_bci_ledger_apply(json_req: &str) -> String {
    init_globals();
    let parsed: Result<WasmApplyRequest, _> = serde_json::from_str(json_req);
    if let Err(e) = parsed {
        return format!(r#"{{"result":null,"civic_class":"Neutral","reward_multiplier":0.0,"error":"invalid-json:{e}"}}"#);
    }
    let req = parsed.unwrap();

    let civic_class = classify_civic(&req.civic_tags);
    if civic_class == CivicClass::Disallowed {
        return r#"{"result":null,"civic_class":"Disallowed","reward_multiplier":0.0,"error":"disallowed-civic-class"}"#.to_string();
    }
    let multiplier = civic_reward_multiplier(civic_class.clone());

    let id_header = req.auth.to_identity_header();

    let ledger_mutex = LEDGER.get().unwrap();
    let rope_mutex = ROPE.get().unwrap();
    let mut ledger = ledger_mutex.lock().unwrap();
    let mut rope = rope_mutex.lock().unwrap();
    let mut orchestrator = BciLedgerOrchestrator::new(&mut ledger, &mut rope);

    let mut event = req.bci_event.clone();
    event.eco_cost_estimate =
        (event.eco_cost_estimate * multiplier).min(ledger.env.eco_flops_limit);

    let session_id = Uuid::new_v4().to_string();
    let handshake = augdoctorpolicies::neurohandshakeorchestrator::NeuroHandshakeOrchestrator::initial(
        &session_id,
        3,
    );

    let res = orchestrator.handle_bci_event(
        &event,
        handshake,
        &id_header,
        0.6,
        &req.timestamp_utc,
    );

    match res {
        Ok((ledger_res, _handshake, _ledger_event, _shot_decision)) => {
            let resp = WasmApplyResponse {
                result: Some(ledger_res),
                civic_class,
                reward_multiplier: multiplier,
                error: None,
            };
            serde_json::to_string(&resp).unwrap_or_else(|e| {
                format!(r#"{{"result":null,"civic_class":"Neutral","reward_multiplier":0.0,"error":"serialization:{e}"}}"#)
            })
        }
        Err(e) => {
            let resp = WasmApplyResponse {
                result: None,
                civic_class,
                reward_multiplier: multiplier,
                error: Some(format!("guard-or-handshake-error:{e}")),
            };
            serde_json::to_string(&resp).unwrap_or_else(|_| {
                r#"{"result":null,"civic_class":"Neutral","reward_multiplier":0.0,"error":"serialization"}"#.to_string()
            })
        }
    }
}
