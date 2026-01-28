#![cfg(feature = "wasm")]

use std::sync::OnceLock;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use bci_bioledger_bridge::{BciEvent, BciLedgerOrchestrator};
use biophysical_blockchain::{BioTokenState, HostEnvelope, IdentityHeader, InnerLedger};
use bioscale_upgradeservice::neuralrope::NeuralRope;
use crate::mapper::{PsychoEnvSnapshot, map_bci_to_sanity_decay, sanity_decay_to_adjustment};

static LEDGER: OnceLock<Mutex<InnerLedger>> = OnceLock::new();
static ROPE:   OnceLock<Mutex<NeuralRope>>   = OnceLock::new();
static SANITY: OnceLock<Mutex<PsychoEnvSnapshot>> = OnceLock::new();

fn init_globals() {
    LEDGER.get_or_init(|| {
        let env = HostEnvelope {
            hostid: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            brainmin: 0.0,
            bloodmin: 0.2,
            oxygenmin: 0.9,
            nanomaxfraction: 0.25,
            smartmax: 1.0,
            ecoflopslimit: 10_000.0,
        };
        let state = BioTokenState {
            brain: 0.5, wave: 0.0, blood: 0.8, oxygen: 0.97, nano: 0.0, smart: 0.0,
        };
        Mutex::new(InnerLedger::new(env, state))
    });

    ROPE.get_or_init(|| Mutex::new(NeuralRope::new("bci-hci-eeg".to_string())));

    SANITY.get_or_init(|| {
        Mutex::new(PsychoEnvSnapshot {
            sanity: 0.7,
            decay:  0.1,
            indoor_bias: 0.7,
            social_score: 0.3,
        })
    });
}

#[derive(Debug, Deserialize)]
pub struct WasmSanityRequest {
    pub identity: IdentityHeader,
    pub event: BciEvent,
    pub required_knowledge: f32,
    pub timestamputc: String,
}

#[derive(Debug, Serialize)]
pub struct WasmSanityResponse {
    pub applied: bool,
    pub sanity: f32,
    pub decay: f32,
    pub reason: String,
    pub prevstatehash: Option<String>,
    pub newstatehash: Option<String>,
}

#[wasm_bindgen]
pub fn wasm_apply_sanity_decay(json_req: &str) -> String {
    init_globals();

    let parsed: Result<WasmSanityRequest, _> = serde_json::from_str(json_req);
    let req = match parsed {
        Ok(v) => v,
        Err(e) => {
            return serde_json::json!({
                "applied": false,
                "sanity": 0.0,
                "decay": 0.0,
                "reason": format!("invalid-json: {}", e),
                "prevstatehash": null,
                "newstatehash": null
            }).to_string();
        }
    };

    // EEG/BCI safety, consent, and DID gating are enforced downstream
    // by the InnerLedger / access layer; here we just compute deltas.
    let sanity_mutex = SANITY.get().unwrap();
    let mut snap = sanity_mutex.lock().unwrap();

    let (ds, dd) = map_bci_to_sanity_decay(&req.event, &snap);

    snap.sanity = (snap.sanity + ds).clamp(0.0, 1.0);
    snap.decay  = (snap.decay  + dd).clamp(0.0, 1.0);

    // Optional: reflect psycho-environmental pressure into inner ledger.
    let eco_cost = 5.0_f64;
    let adj = sanity_decay_to_adjustment(ds, dd, eco_cost);

    let ledger_mutex = LEDGER.get().unwrap();
    let rope_mutex   = ROPE.get().unwrap();
    let mut ledger = ledger_mutex.lock().unwrap();
    let mut rope   = rope_mutex.lock().unwrap();

    let mut orchestrator = BciLedgerOrchestrator::new(&mut ledger, &mut rope);

    let prev_hash = orchestrator.ledger.laststatehash.clone();
    let result = orchestrator.ledger.system_apply(
        req.identity,
        req.required_knowledge,
        adj,
        &req.timestamputc
    );

    match result {
        Ok(ev) => {
            let resp = WasmSanityResponse {
                applied: true,
                sanity: snap.sanity,
                decay: snap.decay,
                reason: "sanity-decay-applied".to_string(),
                prevstatehash: Some(prev_hash),
                newstatehash: Some(ev.newstatehash.clone()),
            };
            serde_json::to_string(&resp).unwrap_or_else(|e| {
                format!(r#"{{"applied":false,"sanity":0.0,"decay":0.0,"reason":"serialization-error: {}"}}"#, e)
            })
        }
        Err(e) => {
            let resp = WasmSanityResponse {
                applied: false,
                sanity: snap.sanity,
                decay: snap.decay,
                reason: format!("inner-ledger-error: {}", e),
                prevstatehash: Some(prev_hash),
                newstatehash: None,
            };
            serde_json::to_string(&resp).unwrap_or_else(|e2| {
                format!(r#"{{"applied":false,"sanity":0.0,"decay":0.0,"reason":"serialization-error: {}"}}"#, e2)
            })
        }
    }
}
