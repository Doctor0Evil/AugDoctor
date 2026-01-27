use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Buf;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper::server::conn::http1;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use bci_bioledger_bridge::{BciEvent, BciLedgerOrchestrator};
use biophysical_blockchain::{
    BioTokenState, HostEnvelope, IdentityHeader, InnerLedger,
};
use bioscaleupgradeservice::neuralrope::NeuralRope;
use augdoctorpolicies::neurohandshakeorchestrator::NeuroHandshakeState;

type HttpBody = Full<Bytes>;

#[derive(Clone)]
struct AppState {
    ledger: Arc<Mutex<InnerLedger>>,
    rope: Arc<Mutex<NeuralRope>>,
}

#[derive(Debug, Deserialize)]
struct BciRequest {
    pub identity: IdentityHeader,
    pub event: BciEvent,
    pub required_knowledge_factor: f32,
    pub timestamp_utc: String,
}

#[derive(Debug, Serialize)]
struct BciResponse {
    pub applied: bool,
    pub reason: String,
    pub session_id: String,
    pub host_id: String,
    pub prev_state_hash: Option<String>,
    pub new_state_hash: Option<String>,
}

async fn handle_request(
    state: AppState,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<HttpBody>, Infallible> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    if method == hyper::Method::POST && path == "/bci/apply" {
        let body_bytes = hyper::body::to_bytes(req.into_body())
            .await
            .unwrap_or_default();
        let parsed: Result<BciRequest, _> = serde_json::from_slice(&body_bytes);
        let req = match parsed {
            Ok(v) => v,
            Err(e) => {
                let body = serde_json::to_vec(&serde_json::json!({
                    "error": format!("invalid JSON: {}", e)
                }))
                .unwrap();
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("content-type", "application/json")
                    .body(Full::from(Bytes::from(body)))
                    .unwrap());
            }
        };

        let mut ledger = state.ledger.lock().unwrap();
        let mut rope = state.rope.lock().unwrap();

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
                let resp = BciResponse {
                    applied: ledger_result.applied,
                    reason: ledger_result.reason,
                    session_id: ledger_result.session_id,
                    host_id: ledger_result.host_id,
                    prev_state_hash: ledger_result.prev_state_hash,
                    new_state_hash: ledger_result.new_state_hash,
                };
                let body = serde_json::to_vec(&resp).unwrap();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Full::from(Bytes::from(body)))
                    .unwrap())
            }
            Err(e) => {
                let body = serde_json::to_vec(&serde_json::json!({
                    "applied": false,
                    "error": format!("{:?}", e)
                }))
                .unwrap();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Full::from(Bytes::from(body)))
                    .unwrap())
            }
        }
    } else if method == hyper::Method::GET && path == "/health" {
        let body = Bytes::from_static(b"{\"status\":\"ok\"}");
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Full::from(body))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::from(Bytes::from_static(b"not found")))
            .unwrap())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Host-local envelope and initial state (non-financial, per doctrine).[file:1]
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

    let ledger = InnerLedger::new(env, state);
    let rope = NeuralRope::new("bci-hci-eeg".to_string());

    let state = AppState {
        ledger: Arc::new(Mutex::new(ledger)),
        rope: Arc::new(Mutex::new(rope)),
    };

    let addr: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    println!("bci-bioledger-service listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let state_clone = state.clone();

        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(stream, service_fn(move |req| {
                    handle_request(state_clone.clone(), req)
                }))
                .await
            {
                eprintln!("connection error: {:?}", e);
            }
        });
    }
}
