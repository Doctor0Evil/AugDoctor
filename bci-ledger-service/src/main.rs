mod security;

use bci_bioledger_bridge::{BciEvent, BciLedgerOrchestrator};
use biophysical_blockchain::{BioTokenState, HostEnvelope, InnerLedger, IdentityHeader};
use bioscaleupgradeservice::neuralrope::NeuralRope;
use hyper::{
    body::Bytes,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, StatusCode,
};
use security::{classify_civic, civic_reward_multiplier, AuthEnvelope, CivicClass};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    ledger: Arc<Mutex<InnerLedger>>,
    rope: Arc<Mutex<NeuralRope>>,
}

/// JSON request body for /bci-ledger/apply.
#[derive(Debug, Deserialize)]
struct ApplyRequest {
    pub auth: AuthEnvelope,
    pub bci_event: BciEvent,
    pub civic_tags: Vec<String>, // semantic labels of the act
    pub timestamp_utc: String,
}

#[derive(Debug, Serialize)]
struct ApplyResponse {
    pub result: bci_bioledger_bridge::BciLedgerResult,
    pub civic_class: CivicClass,
    pub reward_multiplier: f64,
}

/// Initialize a demo InnerLedger for a host.
/// In production, this would be loaded from persistent storage.
fn init_ledger() -> InnerLedger {
    let env = HostEnvelope {
        host_id: "bostrom-host-demo".to_string(),
        brain_min: 0.0,
        blood_min: 0.1,
        oxygen_min: 0.1,
        nano_max_fraction: 0.25,
        smart_max: 1_000.0,
        eco_flops_limit: 10_000.0,
    };
    let state = BioTokenState {
        brain: 10.0,
        wave: 1.0,
        blood: 1.0,
        oxygen: 1.0,
        nano: 0.01,
        smart: 0.5,
    };
    InnerLedger::new(env, state)
}

async fn handle_request(
    state: AppState,
    req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    match (method, path.as_str()) {
        (Method::POST, "/bci-ledger/apply") => {
            let body_bytes = hyper::body::to_bytes(req.into_body())
                .await
                .unwrap_or(Bytes::new());
            let parsed: Result<ApplyRequest, _> = serde_json::from_slice(&body_bytes);
            if let Err(e) = parsed {
                let resp = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from(format!("invalid JSON: {e}")))
                    .unwrap();
                return Ok(resp);
            }
            let req = parsed.unwrap();

            // 1. Civic classification and multiplier.
            let civic_class = classify_civic(&req.civic_tags);
            if civic_class == CivicClass::Disallowed {
                let resp = Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("disallowed civic class"))
                    .unwrap();
                return Ok(resp);
            }
            let multiplier = civic_reward_multiplier(civic_class.clone());

            // 2. Build IdentityHeader from auth envelope.
            let id_header: IdentityHeader = req.auth.to_identity_header();

            // 3. Lock shared ledger + rope.
            let mut ledger = state.ledger.lock().unwrap();
            let mut rope = state.rope.lock().unwrap();
            let mut orchestrator = BciLedgerOrchestrator::new(&mut ledger, &mut rope);

            // 4. Scale BCI event eco_cost by civic multiplier for reward shaping
            // (more heroic acts justify slightly higher eco budget within host limits).
            let mut event = req.bci_event.clone();
            event.eco_cost_estimate =
                (event.eco_cost_estimate * multiplier).min(ledger.env.eco_flops_limit);

            // 5. Handshake state: for simplicity, use a new session per request.
            let session_id = Uuid::new_v4().to_string();
            let mut handshake = augdoctorpolicies::neurohandshakeorchestrator::NeuroHandshakeOrchestrator::initial(
                &session_id,
                3,
            );

            // 6. Call orchestrator. If handshake not ready, this returns an error; we treat it as soft.
            let result = orchestrator.handle_bci_event(
                &event,
                handshake.clone(),
                &id_header,
                0.6, // required knowledge factor (ensures useful-knowledge threshold)
                &req.timestamp_utc,
            );

            match result {
                Ok((ledger_result, new_handshake, _ledger_event, _shot_decision)) => {
                    handshake = new_handshake;
                    let resp_body = ApplyResponse {
                        result: ledger_result,
                        civic_class,
                        reward_multiplier: multiplier,
                    };
                    let body = serde_json::to_vec(&resp_body).unwrap();
                    let resp = Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap();
                    Ok(resp)
                }
                Err(e) => {
                    // Handshake not ready, or inner-ledger guard failure.
                    let resp = Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from(format!("guard or handshake error: {e}")))
                        .unwrap();
                    Ok(resp)
                }
            }
        }
        _ => {
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("not found"))
                .unwrap();
            Ok(resp)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ledger = init_ledger();
    let state = AppState {
        ledger: Arc::new(Mutex::new(ledger)),
        rope: Arc::new(Mutex::new(NeuralRope::new())),
    };

    let listener = TcpListener::bind("0.0.0.0:8181").await?;
    println!("bci-ledger-service listening on http://0.0.0.0:8181");

    loop {
        let (stream, _) = listener.accept().await?;
        let state_clone = state.clone();
        tokio::task::spawn(async move {
            let svc = make_service_fn(move |_conn| {
                let st = state_clone.clone();
                async move {
                    Ok::<_, Infallible>(service_fn(move |req| {
                        handle_request(st.clone(), req)
                    }))
                }
            });

            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(stream, svc)
                .await
            {
                eprintln!("connection error: {e}");
            }
        });
    }
}
