use augdoctor_wordmath_core::{classify_prompt, PromptBand, PromptBandThresholds, PromptMetrics, PromptRightsHeader};
use hyper::{Body, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Clone, Debug, Deserialize)]
struct IncomingPromptRequest {
    pub did: String,
    pub phoenix_profile: String,
    pub neurorights: NeurorightsFlags,
    pub impact: SocialImpactVector,
    pub text: String,
    pub metrics: PromptMetrics, // computed by a front-end model
}

#[derive(Clone, Debug, Serialize)]
struct RewriteSuggestion {
    pub explanation: String,
    pub suggested_text: String,
}

#[derive(Clone, Debug, Serialize)]
struct WordMathResponse {
    pub header: PromptRightsHeader,
    pub action: String, // "admit", "rewrite", "block"
    pub rewrite: Option<RewriteSuggestion>,
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if req.method() != hyper::Method::POST {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::from("use POST /wordmath"))
            .unwrap());
    }

    let whole = hyper::body::to_bytes(req.into_body()).await?;
    let incoming: IncomingPromptRequest = match serde_json::from_slice(&whole) {
        Ok(v) => v,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("bad json"))
                .unwrap());
        }
    };

    let thresholds = PromptBandThresholds::default();
    let score = classify_prompt(incoming.metrics.clone(), &thresholds);

    let header = PromptRightsHeader {
        did: incoming.did,
        phoenix_profile: incoming.phoenix_profile,
        neurorights: incoming.neurorights,
        impact: incoming.impact,
        wordmath: score,
    };

    let (action, rewrite) = match header.wordmath.band {
        PromptBand::GreenAdmit => ("admit".to_string(), None),
        PromptBand::AmberRewrite => {
            let suggestion = RewriteSuggestion {
                explanation: "Reduce repetition, center on declared task triad, soften adversarial language, and add one DID-bound evidential hook.".to_string(),
                suggested_text: incoming
                    .text
                    .replace("!", ".")
                    .replace("must", "should")
                    .to_string(),
            };
            ("rewrite".to_string(), Some(suggestion))
        }
        PromptBand::RedBlocked => {
            let suggestion = RewriteSuggestion {
                explanation: "Prompt is too off-topic, repetitive, or toxic for safe quantum-learning. Remove hostile framing, narrow to cybernetics/biophysical-blockchain/quantum-learning, and specify non-financial, DID-bound constraints."
                    .to_string(),
                suggested_text: "".to_string(),
            };
            ("block".to_string(), Some(suggestion))
        }
    };

    let resp = WordMathResponse { header, action, rewrite };
    let body = serde_json::to_vec(&resp).unwrap();
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = "127.0.0.1:8088".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("wordmath-rewrite-service listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = hyper::server::conn::http1::Builder::new().serve_connection(
            stream,
            hyper::service::service_fn(handle),
        );
        tokio::task::spawn(async move {
            if let Err(e) = io.await {
                eprintln!("conn error: {:?}", e);
            }
        });
    }
}
