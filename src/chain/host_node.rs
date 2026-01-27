//! Biophysical Host Node (AugDoctor, Phoenix-grade test)
//!
//! Responsibilities:
//! - Maintain a sealed, per-host BioTokenState.
//! - Expose a minimal RPC surface (JSON over TCP) for:
//!     - Reading redacted state summaries.
//!     - Submitting RuntimeEvents (WaveLoad, SmartAutonomy, EvolutionUpgrade).
//! - Gossip Lorentz-attested ConsensusFrames to peers (no economic consensus).
//! - Enforce ALN/DID gating and Lifeforce safety at the node boundary.
//!
//! This is a lab-grade, non-financial node: no transfers, no staking, no bridges,
//! only biophysical-governance and audit.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::chain::biophysical_runtime::{
    aln_did,
    biospectre_consensus,
    lifeforce_safety,
    quantum_hash,
    BiophysicalRuntime,
    BioTokenState,
    LorentzTimeSource,
    LorentzTimestamp,
    RuntimeEvent,
    RuntimeEventKind,
    RuntimeResult,
    RuntimeConfig,
    ALNHostFrame,
    SystemLorentzClock,
};

// ------------------ Storage & Node State ------------------------------------

#[derive(Clone)]
struct HostStorage {
    inner: Arc<RwLock<InnerStore>>,
}

struct InnerStore {
    state: BioTokenState,
    last_frame: Option<biospectre_consensus::ConsensusFrame>,
}

impl HostStorage {
    fn new(initial_state: BioTokenState) -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerStore {
                state: initial_state,
                last_frame: None,
            })),
        }
    }

    fn read_state(&self) -> BioTokenState {
        self.inner.read().unwrap().state.clone()
    }

    fn read_last_frame(&self) -> Option<biospectre_consensus::ConsensusFrame> {
        self.inner.read().unwrap().last_frame.clone()
    }

    fn apply_state_and_frame(
        &self,
        new_state: BioTokenState,
        frame: biospectre_consensus::ConsensusFrame,
    ) {
        let mut guard = self.inner.write().unwrap();
        guard.state = new_state;
        guard.last_frame = Some(frame);
    }
}

// ------------------ Simple DID directory & consent verifier -----------------

#[derive(Clone)]
struct InMemoryDIDDirectory {
    // did.id -> AccessEnvelope
    access: Arc<RwLock<HashMap<String, aln_did::AccessEnvelope>>>,
}

impl InMemoryDIDDirectory {
    fn new() -> Self {
        Self {
            access: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn insert(&self, env: aln_did::AccessEnvelope) {
        self.access.write().unwrap().insert(env.did.id.clone(), env);
    }
}

impl aln_did::DIDDirectory for InMemoryDIDDirectory {
    fn resolve_access(&self, did: &aln_did::ALNDID) -> Option<aln_did::AccessEnvelope> {
        self.access.read().unwrap().get(&did.id).cloned()
    }

    fn is_ethical_operator(&self, did: &aln_did::ALNDID) -> bool {
        self.resolve_access(did)
            .map(|a| a.roles.contains(&aln_did::RoleClass::EthicalOperator))
            .unwrap_or(false)
    }
}

#[derive(Clone)]
struct SimpleConsentVerifier;

impl aln_did::ConsentVerifier for SimpleConsentVerifier {
    fn verify_self_consent(&self, proof: &aln_did::ConsentProof) -> bool {
        // Lab-grade stub: accept if zk_sig non-empty and evolution_event_id non-empty.
        !proof.zk_sig.is_empty() && !proof.evolution_event_id.is_empty()
    }
}

// ------------------ Simple host consensus implementation --------------------

#[derive(Clone)]
struct LocalHostConsensus;

impl biospectre_consensus::HostConsensus for LocalHostConsensus {
    fn validate_state_step(
        &self,
        previous: Option<&biospectre_consensus::ConsensusFrame>,
        next: &biospectre_consensus::ConsensusFrame,
        _: &BioTokenState,
    ) -> Result<(), &'static str> {
        if let Some(prev) = previous {
            if next.seq_no != prev.seq_no + 1 {
                return Err("Sequence mismatch in consensus frame.");
            }
            if let Some(prev_hash) = &next.prev_state_hash {
                if *prev_hash != prev.state_hash {
                    return Err("Previous hash mismatch in consensus frame.");
                }
            } else {
                return Err("Missing prev_state_hash while previous frame exists.");
            }
        } else if next.seq_no != 0 {
            return Err("Genesis frame must have seq_no 0.");
        }
        Ok(())
    }
}

// ------------------ Lifeforce configuration ---------------------------------

fn default_lifeforce_state() -> lifeforce_safety::LifeforceState {
    lifeforce_safety::LifeforceState {
        bands: lifeforce_safety::MetabolicBands {
            blood_min: 0.25,
            blood_soft_floor: 0.35,
            oxygen_min: 0.90,
            oxygen_soft_floor: 0.94,
        },
        wave_curve: lifeforce_safety::DraculaWaveCurve {
            max_wave_factor: 0.6,
            decay_coefficient: 0.01,
        },
        nano_envelope: lifeforce_safety::NanoEnvelope {
            max_concurrent_workload: 1.0,
            eco_penalty_factor: 0.5,
        },
    }
}

// ------------------ Gossip messages -----------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct GossipFrame {
    host_id: String,
    shard: String,
    seq_no: u64,
    state_hash: String,
    prev_state_hash: Option<String>,
}

// ------------------ RPC protocol -------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RpcRequest {
    GetStateSummary {},
    SubmitEvent {
        event: RpcEvent,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RpcEventKind {
    EvolutionUpgrade { evolution_id: String },
    WaveLoad { task_id: String, requested_wave: f64 },
    SmartAutonomy { agent_id: String, requested_smart: f64 },
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcEvent {
    initiator_did: String,
    initiator_shard: String,
    consent: Option<RpcConsent>,
    event_kind: RpcEventKind,
}

#[derive(Debug, Serialize, Deserialize)]
struct RpcConsent {
    evolution_event_id: String,
    zk_sig: String,
    timestamp_ms_utc: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum RpcResponse {
    OkStateSummary { summary: StateSummary },
    OkEventApplied { seq_no: u64, state_hash: String },
    Error { error: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct StateSummary {
    brain: f64,
    wave: f64,
    blood: f64,
    oxygen: f64,
    nano: f64,
    smart: f64,
    lorentz_ts: (i128, i64),
}

// Security header for AI-Chat → HostNode calls, mirroring EEG schema headers.
#[derive(Debug, Serialize, Deserialize)]
struct RpcSecurityHeader {
    pub issuer_did: String,
    pub subject_role: String,   // "augmented_citizen", "authorized_researcher", "system_daemon"
    pub network_tier: String,   // "core", "edge", "sandbox"
    pub biophysical_chain_allowed: bool,
}

// Extended RPC request that carries security metadata.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RpcRequest {
    GetStateSummary {
        header: RpcSecurityHeader,
    },
    SubmitEvent {
        header: RpcSecurityHeader,
        event: RpcEvent,
    },
}

// Hard guard matching your EEG header doctrine.
fn validate_rpc_header(header: &RpcSecurityHeader) -> Result<(), String> {
    const SUPPORTED_TIER_CORE: &str = "core";
    const SUPPORTED_TIER_EDGE: &str = "edge";
    const TIER_SANDBOX: &str = "sandbox";

    // Only ALNDID/Bostrom namespaces may talk to inner ledger.
    if !header.issuer_did.starts_with("bostrom")
        && !header.issuer_did.starts_with("did:")
    {
        return Err("issuer_did must be ALNDID/Bostrom namespace".to_string());
    }

    // Subject roles restricted to augmented-citizen mechanics.
    match header.subject_role.as_str() {
        "augmented_citizen" | "authorized_researcher" | "system_daemon" => {}
        _ => return Err(format!("unauthorized subject_role {}", header.subject_role)),
    }

    // Sandbox nodes can never anchor to biophysical chain or mutate state.
    match header.network_tier.as_str() {
        TIER_SANDBOX => {
            if header.biophysical_chain_allowed {
                return Err("sandbox tier cannot set biophysical_chain_allowed=true".to_string());
            }
            // We also block any mutating SubmitEvent from sandbox below.
        }
        SUPPORTED_TIER_CORE | SUPPORTED_TIER_EDGE => {}
        other => return Err(format!("invalid network_tier {}", other)),
    }

    Ok(())
}

// ------------------ Host node structure -------------------------------------

pub struct HostNode<D, C, HC>
where
    D: aln_did::DIDDirectory + Send + Sync + 'static,
    C: aln_did::ConsentVerifier + Send + Sync + 'static,
    HC: biospectre_consensus::HostConsensus + Send + Sync + 'static,
{
    host_id: aln_did::ALNDID,
    runtime: Arc<BiophysicalRuntime<D, C, HC>>,
    storage: HostStorage,
    clock: Arc<dyn LorentzTimeSource + Send + Sync>,
    gossip_tx: mpsc::Sender<GossipFrame>,
}

impl<D, C, HC> HostNode<D, C, HC>
where
    D: aln_did::DIDDirectory + Send + Sync + 'static,
    C: aln_did::ConsentVerifier + Send + Sync + 'static,
    HC: biospectre_consensus::HostConsensus + Send + Sync + 'static,
{
    pub fn new(
        host_id: aln_did::ALNDID,
        runtime: BiophysicalRuntime<D, C, HC>,
        initial_state: BioTokenState,
    ) -> (Self, mpsc::Receiver<GossipFrame>) {
        let storage = HostStorage::new(initial_state);
        let (gossip_tx, gossip_rx) = mpsc::channel(128);
        let node = Self {
            host_id,
            runtime: Arc::new(runtime),
            storage,
            clock: Arc::new(SystemLorentzClock),
            gossip_tx,
        };
        (node, gossip_rx)
    }

    fn build_host_frame(&self) -> ALNHostFrame {
        ALNHostFrame {
            host_id: self.host_id.clone(),
            access: aln_did::AccessEnvelope {
                did: self.host_id.clone(),
                roles: vec![aln_did::RoleClass::Host],
                min_biophysics_knowledge_score: 1.0,
            },
            lorentz_ts: self.clock.now_lorentz(),
        }
    }

    fn convert_rpc_event(&self, rpc: RpcEvent) -> RuntimeEvent {
        let initiator = aln_did::ALNDID {
            id: rpc.initiator_did,
            shard: rpc.initiator_shard,
        };
        let consent = rpc.consent.map(|c| aln_did::ConsentProof {
            did: initiator.clone(),
            evolution_event_id: c.evolution_event_id,
            timestamp_ms_utc: c.timestamp_ms_utc,
            zk_sig: c.zk_sig.into_bytes(),
        });

        let kind = match rpc.event_kind {
            RpcEventKind::EvolutionUpgrade { evolution_id } => RuntimeEventKind::EvolutionUpgrade {
                evolution_id,
            },
            RpcEventKind::WaveLoad { task_id, requested_wave } => {
                RuntimeEventKind::WaveLoad { task_id, requested_wave }
            }
            RpcEventKind::SmartAutonomy {
                agent_id,
                requested_smart,
            } => RuntimeEventKind::SmartAutonomy {
                agent_id,
                requested_smart,
            },
        };

        RuntimeEvent {
            kind,
            initiator,
            consent,
            lorentz_ts: self.clock.now_lorentz(),
        }
    }

   async fn handle_rpc(&self, req: RpcRequest) -> RpcResponse {
    match req {
        RpcRequest::GetStateSummary { header } => {
            if let Err(e) = validate_rpc_header(&header) {
                return RpcResponse::Error { error: e };
            }
            let s = self.storage.read_state();
            RpcResponse::OkStateSummary {
                summary: StateSummary {
                    brain: s.brain,
                    wave: s.wave,
                    blood: s.blood,
                    oxygen: s.oxygen,
                    nano: s.nano,
                    smart: s.smart,
                    lorentz_ts: (s.lorentz_ts.0, s.lorentz_ts.1),
                },
            }
        }
        RpcRequest::SubmitEvent { header, event } => {
            if let Err(e) = validate_rpc_header(&header) {
                return RpcResponse::Error { error: e };
            }
            // Explicitly refuse sandbox network tier for mutating calls.
            if header.network_tier == "sandbox" {
                return RpcResponse::Error {
                    error: "sandbox tier cannot submit mutating events".to_string(),
                };
            }

            let mut state = self.storage.read_state();
            let host_frame = self.build_host_frame();
            state.lorentz_ts = host_frame.lorentz_ts;

            let runtime_event = self.convert_rpc_event(event);
            let previous = self.storage.read_last_frame();
            let previous_ref = previous.as_ref();

            let result: RuntimeResult<biospectre_consensus::ConsensusFrame> =
                self.runtime
                    .execute_event(&mut state, previous_ref, &host_frame, &runtime_event);

            match result {
                Ok(frame) => {
                    let state_hash_str = hex::encode(frame.state_hash.0);
                    let prev_state_hash_str =
                        frame.prev_state_hash.as_ref().map(|h| hex::encode(h.0));
                    self.storage.apply_state_and_frame(state.clone(), frame.clone());

                    let _ = self
                        .gossip_tx
                        .send(GossipFrame {
                            host_id: self.host_id.id.clone(),
                            shard: self.host_id.shard.clone(),
                            seq_no: frame.seq_no,
                            state_hash: state_hash_str.clone(),
                            prev_state_hash: prev_state_hash_str,
                        })
                        .await;

                    RpcResponse::OkEventApplied {
                        seq_no: frame.seq_no,
                        state_hash: state_hash_str,
                    }
                }
                Err(e) => RpcResponse::Error {
                    error: format!("{:?}", e),
                },
            }
        }
    }
}

    pub async fn serve(self, bind: SocketAddr) -> anyhow::Result<()> {
        let listener = TcpListener::bind(bind).await?;
        println!("[HostNode] Listening on {}", bind);

        loop {
            let (socket, addr) = listener.accept().await?;
            let node = self.clone();
            tokio::spawn(async move {
                if let Err(e) = node.handle_connection(socket).await {
                    eprintln!("[HostNode] Connection {} error: {:?}", addr, e);
                }
            });
        }
    }

    async fn handle_connection(&self, socket: TcpStream) -> anyhow::Result<()> {
        let (reader, mut writer) = socket.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        while reader.read_line(&mut line).await? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            let parsed: Result<RpcRequest, _> = serde_json::from_str(trimmed);
            let resp = match parsed {
                Ok(req) => self.handle_rpc(req).await,
                Err(e) => RpcResponse::Error {
                    error: format!("Invalid request: {}", e),
                },
            };

            let resp_json = serde_json::to_string(&resp)?;
            writer.write_all(resp_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
            line.clear();
        }

        Ok(())
    }
}

impl<D, C, HC> Clone for HostNode<D, C, HC>
where
    D: aln_did::DIDDirectory + Send + Sync + 'static + Clone,
    C: aln_did::ConsentVerifier + Send + Sync + 'static + Clone,
    HC: biospectre_consensus::HostConsensus + Send + Sync + 'static + Clone,
{
    fn clone(&self) -> Self {
        Self {
            host_id: self.host_id.clone(),
            runtime: self.runtime.clone(),
            storage: self.storage.clone(),
            clock: self.clock.clone(),
            gossip_tx: self.gossip_tx.clone(),
        }
    }
}

// ------------------ Bootstrap helper ----------------------------------------

pub async fn bootstrap_single_host_node(
    host_id: aln_did::ALNDID,
    bind: SocketAddr,
) -> anyhow::Result<(JoinHandle<()>, mpsc::Receiver<GossipFrame>)> {
    let did_dir = InMemoryDIDDirectory::new();
    did_dir.insert(aln_did::AccessEnvelope {
        did: host_id.clone(),
        roles: vec![aln_did::RoleClass::Host],
        min_biophysics_knowledge_score: 1.0,
    });

    let runtime = BiophysicalRuntime::new(
        RuntimeConfig::default(),
        default_lifeforce_state(),
        did_dir,
        SimpleConsentVerifier,
        LocalHostConsensus,
    );

    let initial_state = BioTokenState {
        brain: 1.0,
        wave: 0.0,
        blood: 1.0,
        oxygen: 0.98,
        nano: 0.0,
        smart: 0.0,
        host_id: host_id.clone(),
        lorentz_ts: LorentzTimestamp(0, 0),
    };

    let (node, gossip_rx) = HostNode::new(host_id, runtime, initial_state);
    let handle = tokio::spawn(async move {
        if let Err(e) = node.serve(bind).await {
            eprintln!("[HostNode] Server failed: {:?}", e);
        }
    });

    Ok((handle, gossip_rx))
}
