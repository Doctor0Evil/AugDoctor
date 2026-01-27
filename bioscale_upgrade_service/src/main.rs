mod neural_rope;

use bioscale_upgrade_store::bioscale::upgrade_asset::{
    BioscaleAwarenessProfile, BioscaleUpgradeAsset, ConsciousnessComplianceLevel,
    HardwareBindingProfile,
};
use bioscale_upgrade_store::{
    BioscaleStoreConfig, BioscaleUpgradeStore, UpgradeApplicationResult,
};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, StatusCode};
use neural_rope::NeuralRope;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    store: Arc<Mutex<BioscaleUpgradeStore>>,
    neural_rope: Arc<Mutex<NeuralRope>>,
}

#[derive(Debug, Deserialize)]
struct RegisterUpgradeRequest {
    human_label: String,
    tags: Vec<String>,
    tissue_interface: Vec<String>,
    organ_targets: Vec<String>,
    biosignal_channels: Vec<String>,
    consciousness_compliance: String,
    allowed_hardware_ids: Vec<String>,
    required_safety_modules: Vec<String>,
    bioscale_resolution_microns: u32,
    metadata_hash: String,
}

#[derive(Debug, Serialize)]
struct RegisterUpgradeResponse {
    upgrade_id: String,
}

#[derive(Debug, Deserialize)]
struct ApplyUpgradeRequest {
    upgrade_id: String,
    environment_id: String,
    environment_hardware: Vec<String>,
    environment_tags: Vec<String>,
    reward_score: f32,
}

#[derive(Debug, Serialize)]
struct ApplyUpgradeResponse {
    result: UpgradeApplicationResult,
    neural_rope_segments: Vec<neural_rope::NeuralRopeSegmentSnapshot>,
}

async fn handle_request(
    state: AppState,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    match (method, path.as_str()) {
        (Method::POST, "/register_upgrade") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
            let parsed: Result<RegisterUpgradeRequest, _> =
                serde_json::from_slice(&body_bytes);

            if let Err(e) = parsed {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("invalid JSON: {}", e)))
                    .unwrap());
            }

            let req = parsed.unwrap();

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

            let mut store = state.store.lock().unwrap();
            let res = store.register_upgrade(asset);

            match res {
                Ok(id) => {
                    let resp = RegisterUpgradeResponse { upgrade_id: id };
                    let body = serde_json::to_vec(&resp).unwrap();
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap())
                }
                Err(e) => Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(e.to_string()))
                    .unwrap()),
            }
        }

        (Method::POST, "/apply_upgrade") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap_or_default();
            let parsed: Result<ApplyUpgradeRequest, _> =
                serde_json::from_slice(&body_bytes);

            if let Err(e) = parsed {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("invalid JSON: {}", e)))
                    .unwrap());
            }

            let req = parsed.unwrap();

            let mut store = state.store.lock().unwrap();
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

                    let mut rope = state.neural_rope.lock().unwrap();
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

                    let resp = ApplyUpgradeResponse {
                        result: upgrade_result,
                        neural_rope_segments: segments,
                    };

                    let body = serde_json::to_vec(&resp).unwrap();
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap())
                }
                Err(e) => Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(e.to_string()))
                    .unwrap()),
            }
        }

        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = BioscaleStoreConfig {
        allow_offline_registration: true,
        default_regulatory_labels: vec![
            String::from("ALN"),
            String::from("KYC"),
            String::from("DID"),
        ],
        max_upgrade_assets: 1024,
    };

    let store = BioscaleUpgradeStore::new(cfg);
    let neural_rope = NeuralRope::new();

    let state = AppState {
        store: Arc::new(Mutex::new(store)),
        neural_rope: Arc::new(Mutex::new(neural_rope)),
    };

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("bioscale_upgrade_service listening on http://0.0.0.0:8080");

    loop {
        let (stream, _) = listener.accept().await?;
        let state_clone = state.clone();

        tokio::task::spawn(async move {
            let io = hyper::server::conn::http1::Builder::new()
                .serve_connection(
                    stream,
                    service_fn(move |req| {
                        let state_clone2 = state_clone.clone();
                        async move { handle_request(state_clone2, req).await }
                    }),
                )
                .await;

            if let Err(e) = io {
                eprintln!("connection error: {}", e);
            }
        });
    }
}
