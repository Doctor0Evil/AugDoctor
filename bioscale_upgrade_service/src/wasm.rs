#![cfg(feature = "wasm")]

use crate::neural_rope::{NeuralRope, NeuralRopeSegmentSnapshot};
use bioscale_upgrade_store::bioscale::upgrade_asset::{
    BioscaleAwarenessProfile, BioscaleUpgradeAsset, ConsciousnessComplianceLevel,
    HardwareBindingProfile,
};
use bioscale_upgrade_store::{
    BioscaleStoreConfig, BioscaleUpgradeStore, UpgradeApplicationResult,
};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use wasm_bindgen::prelude::*;

static STORE: OnceLock<Mutex<BioscaleUpgradeStore>> = OnceLock::new();
static ROPE: OnceLock<Mutex<NeuralRope>> = OnceLock::new();

fn init_globals() {
    STORE.get_or_init(|| {
        let cfg = BioscaleStoreConfig {
            allow_offline_registration: true,
            default_regulatory_labels: vec![
                String::from("ALN"),
                String::from("KYC"),
                String::from("DID"),
            ],
            max_upgrade_assets: 1024,
        };
        Mutex::new(BioscaleUpgradeStore::new(cfg))
    });

    ROPE.get_or_init(|| Mutex::new(NeuralRope::new()));
}

#[derive(Debug, Deserialize)]
pub struct WasmRegisterUpgradeRequest {
    pub human_label: String,
    pub tags: Vec<String>,
    pub tissue_interface: Vec<String>,
    pub organ_targets: Vec<String>,
    pub biosignal_channels: Vec<String>,
    pub consciousness_compliance: String,
    pub allowed_hardware_ids: Vec<String>,
    pub required_safety_modules: Vec<String>,
    pub bioscale_resolution_microns: u32,
    pub metadata_hash: String,
}

#[derive(Debug, Serialize)]
pub struct WasmRegisterUpgradeResponse {
    pub upgrade_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WasmApplyUpgradeRequest {
    pub upgrade_id: String,
    pub environment_id: String,
    pub environment_hardware: Vec<String>,
    pub environment_tags: Vec<String>,
    pub reward_score: f32,
}

#[derive(Debug, Serialize)]
pub struct WasmApplyUpgradeResponse {
    pub result: UpgradeApplicationResult,
    pub neural_rope_segments: Vec<NeuralRopeSegmentSnapshot>,
}

#[wasm_bindgen]
pub fn bioscale_register_upgrade(json_req: &str) -> String {
    init_globals();

    let req: WasmRegisterUpgradeRequest = match serde_json::from_str(json_req) {
        Ok(v) => v,
        Err(e) => {
            return format!(r#"{{"error":"invalid_json","detail":"{}"}}"#, e);
        }
    };

    let awareness = BioscaleAwarenessProfile {
        involves_living_organism: true,
        tissue_interface: req.tissue_interface,
        organ_targets: req.organ_targets,
        biosignal_channels: req.biosignal_channels,
    };

    let compliance = match req.consciousness_compliance.to_lowercase().as_str() {
        "no-conscious-substrate" => ConsciousnessComplianceLevel::NoConsciousSubstrate,
        "indirect-non-identity" => ConsciousnessComplianceLevel::IndirectNonIdentity,
        "direct-immutable-non-quantifying" => {
            ConsciousnessComplianceLevel::DirectImmutableNonQuantifying
        }
        _ => ConsciousnessComplianceLevel::IndirectNonIdentity,
    };

    let hardware_binding = HardwareBindingProfile {
        allowed_hardware_ids: req.allowed_hardware_ids,
        required_safety_modules: req.required_safety_modules,
        bioscale_resolution_microns: req.bioscale_resolution_microns,
    };

    let asset = BioscaleUpgradeAsset::new(
        &req.human_label,
        awareness,
        compliance,
        hardware_binding,
        req.tags,
        &req.metadata_hash,
    );

    let store_mutex = STORE.get().unwrap();
    let mut store = store_mutex.lock().unwrap();

    match store.register_upgrade(asset) {
        Ok(id) => {
            let resp = WasmRegisterUpgradeResponse { upgrade_id: id };
            serde_json::to_string(&resp).unwrap_or_else(|e| {
                format!(r#"{{"error":"serialization_error","detail":"{}"}}"#, e)
            })
        }
        Err(e) => {
            format!(r#"{{"error":"guard_or_store_error","detail":"{}"}}"#, e)
        }
    }
}

#[wasm_bindgen]
pub fn bioscale_apply_upgrade(json_req: &str) -> String {
    init_globals();

    let req: WasmApplyUpgradeRequest = match serde_json::from_str(json_req) {
        Ok(v) => v,
        Err(e) => {
            return format!(r#"{{"error":"invalid_json","detail":"{}"}}"#, e);
        }
    };

    let store_mutex = STORE.get().unwrap();
    let mut store = store_mutex.lock().unwrap();

    let result = store.apply_upgrade_to_environment(
        &req.upgrade_id,
        &req.environment_id,
        req.environment_hardware.clone(),
        req.environment_tags.clone(),
    );

    match result {
        Ok(upgrade_result) => {
            let plane_label = upgrade_result
                .environment_metadata
                .environment_plane
                .to_string();
            let safety_decision = upgrade_result.guard_decision.clone();

            let rope_mutex = ROPE.get().unwrap();
            let mut rope = rope_mutex.lock().unwrap();
            let trace_text = format!(
                "env_id={} upgrade_id={} plane={} decision={}",
                req.environment_id, req.upgrade_id, plane_label, safety_decision
            );
            rope.append_trace(
                &trace_text,
                &plane_label,
                Some(req.upgrade_id.clone()),
                req.reward_score,
                &safety_decision,
            );
            let segments = rope.export_snapshot(16);

            let resp = WasmApplyUpgradeResponse {
                result: upgrade_result,
                neural_rope_segments: segments,
            };
            serde_json::to_string(&resp).unwrap_or_else(|e| {
                format!(r#"{{"error":"serialization_error","detail":"{}"}}"#, e)
            })
        }
        Err(e) => format!(r#"{{"error":"guard_or_store_error","detail":"{}"}}"#, e),
    }
}
