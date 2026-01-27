//! Biophysical Host Node (AugDoctor, Phoenix-grade test)
//
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
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
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
    RuntimeEvent as CoreRuntimeEvent,
    RuntimeEventKind,
    RuntimeResult,
    RuntimeConfig,
    ALNHostFrame,
    SystemLorentzClock,
};

use crate::security::{AuthEnvelope, CivicClass};
use crate::civic_profile::CivicRewardProfile;
use crate::civic_audit::{CivicAuditEntry, append_civic_audit_entry, eco_band_label};
use augdoctorpolicies::neurohandshakeorchestrator::{
    HandshakePhase, NeuroHandshakeOrchestrator, NeuroHandshakeState,
};
use augdoctorpolicies::shotlevelpolicy::{
    ShotLevel, ShotLevelDecision, ShotLevelPolicy, ShotLevelPolicyConfig,
};
use bioscaleupgradeservice::neuralrope::NeuralRope;
use biophysical_blockchain::{HostEnvelope, InnerLedger};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader as StdBufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

// ------------------ Storage & Node State (inner consensus) ------------------

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

// ------------------ Outer JSON-RPC for AI-Chat ------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct RpcSecurityHeader {
    pub issuer_did: String,
    pub subject_role: String,   // "augmented_citizen", "authorized_researcher", "system_daemon"
    pub network_tier: String,   // "core", "edge", "sandbox"
    pub biophysical_chain_allowed: bool,
}

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

// Hard guard matching EEG / ALN header doctrine.
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
        }
        SUPPORTED_TIER_CORE | SUPPORTED_TIER_EDGE => {}
        other => return Err(format!("invalid network_tier {}", other)),
    }

    Ok(())
}

// ------------------ Disk layout for Phoenix host ----------------------------

#[derive(Clone, Debug)]
pub struct HostNodePaths {
    pub base_dir: PathBuf,
    pub state_file: PathBuf,
    pub consensus_log: PathBuf,
    pub civic_audit_log: PathBuf,
    pub civic_profile_json: PathBuf,
}

impl HostNodePaths {
    pub fn new<P: AsRef<Path>>(base: P) -> Self {
        let base_dir = base.as_ref().to_path_buf();
        Self {
            state_file: base_dir.join("state/bio_token_state.json"),
            consensus_log: base_dir.join("consensus/frames.log"),
            civic_audit_log: base_dir.join("audit/civic-audit-log.jsonl"),
            civic_profile_json: base_dir.join("profiles/civic-reward-profile.json"),
            base_dir,
        }
    }

    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.consensus_log.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.civic_audit_log.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.civic_profile_json.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

// ------------------ Mock ALN / Bostrom directory & consent ------------------

#[derive(Clone, Debug)]
pub struct DidDirectory {
    pub identities: HashMap<String, Vec<String>>,
}

impl DidDirectory {
    pub fn new() -> Self {
        let mut identities = HashMap::new();
        identities.insert(
            "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            vec!["augmented-citizen".to_string()],
        );
        Self { identities }
    }

    pub fn has_role(&self, subject: &str, role: &str) -> bool {
        self.identities
            .get(subject)
            .map(|roles| roles.iter().any(|r| r == role))
            .unwrap_or(false)
    }
}

#[derive(Clone, Debug)]
pub struct ConsentVerifier {
    pub require_augmented_citizen: bool,
}

impl ConsentVerifier {
    pub fn new() -> Self {
        Self {
            require_augmented_citizen: true,
        }
    }

    pub fn verify(
        &self,
        did_directory: &DidDirectory,
        auth: &AuthEnvelope,
    ) -> Result<(), String> {
        let subject = &auth.subject_id;
        if self.require_augmented_citizen
            && !did_directory.has_role(subject, "augmented-citizen")
        {
            return Err("missing required role: augmented-citizen".to_string());
        }
        if auth.financialization_intent {
            return Err("financialization_intent not permitted on biophysical host".to_string());
        }
        Ok(())
    }
}

// ------------------ Minimal inner consensus frame (non-financial) -----------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusFrame {
    pub frame_id: String,
    pub prev_frame_id: String,
    pub timestamp_utc: String,
    pub hostid: String,
    pub event_hash: String,
    pub lifeforce_ok: bool,
    pub eco_cost: f64,
    pub eco_band: String,
}

#[derive(Clone, Debug)]
pub struct HostConsensus {
    pub hostid: String,
    pub last_frame_id: String,
}

impl HostConsensus {
    pub fn new(hostid: &str) -> Self {
        Self {
            hostid: hostid.to_string(),
            last_frame_id: "0xGENESIS".to_string(),
        }
    }

    pub fn build_frame(
        &mut self,
        timestamp_utc: &str,
        event_hash: &str,
        lifeforce_ok: bool,
        eco_cost: f64,
        eco_band: &str,
    ) -> ConsensusFrame {
        let frame_id = format!("0xFRM-{}", Uuid::new_v4());
        let frame = ConsensusFrame {
            frame_id: frame_id.clone(),
            prev_frame_id: self.last_frame_id.clone(),
            timestamp_utc: timestamp_utc.to_string(),
            hostid: self.hostid.clone(),
            event_hash: event_hash.to_string(),
            lifeforce_ok,
            eco_cost,
            eco_band: eco_band.to_string(),
        };
        self.last_frame_id = frame_id;
        frame
    }
}

// ------------------ Host-local runtime event (AI-Chat → host) ---------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub auth: AuthEnvelope,
    pub civic_tags: Vec<String>,
    pub timestamp_utc: String,
    pub bci_event: crate::api::BciEvent,
}

// ------------------ JSON-RPC envelope (AI-Chat side) ------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub id: serde_json::Value,
    pub params: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

// ------------------ Node-internal shared state ------------------------------

#[derive(Clone)]
pub struct HostNodeState {
    pub hostid: String,
    pub ledger: Arc<Mutex<InnerLedger>>,
    pub rope: Arc<Mutex<NeuralRope>>,
    pub civic_profile: Arc<CivicRewardProfile>,
    pub did_directory: Arc<DidDirectory>,
    pub consent_verifier: Arc<ConsentVerifier>,
    pub consensus: Arc<Mutex<HostConsensus>>,
    pub handshake_state: Arc<Mutex<NeuroHandshakeState>>,
    pub shot_policy: Arc<ShotLevelPolicy>,
    pub paths: HostNodePaths,
}

impl HostNodeState {
    pub fn load_or_init(
        hostid: &str,
        base_dir: impl AsRef<Path>,
    ) -> std::io::Result<Self> {
        let paths = HostNodePaths::new(base_dir);
        paths.ensure_dirs()?;

        // Load or init BioTokenState via InnerLedger.
        let ledger = if paths.state_file.exists() {
            let raw = fs::read_to_string(&paths.state_file)?;
            let state: BioTokenState = serde_json::from_str(&raw)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            InnerLedger::from_state(state)
        } else {
            InnerLedger::new_default(hostid.to_string())
        };

        // Civic profile from JSON (or default).
        let civic_profile = CivicRewardProfile::load_from_json(&paths.civic_profile_json)
            .unwrap_or_else(|_| CivicRewardProfile {
                multiplier_min: 0.0,
                multiplier_max: 4.0,
                default_multiplier: 1.0,
                required_knowledge_factor: 0.60,
                heroic_tags: std::collections::HashSet::new(),
                heroic_multiplier: 3.0,
                good_tags: std::collections::HashSet::new(),
                good_multiplier: 1.5,
                neutral_multiplier: 1.0,
                disallowed_tags: std::collections::HashSet::new(),
                eco_bonus_enabled: false,
                eco_low_threshold: 0.0,
                eco_low_bonus: 1.0,
            });

        let did_directory = DidDirectory::new();
        let consent_verifier = ConsentVerifier::new();
        let consensus = HostConsensus::new(hostid);

        let handshake_state =
            NeuroHandshakeOrchestrator::initial("session-host", 3);
        let shot_policy = ShotLevelPolicy::new(ShotLevelPolicyConfig {
            maxexamplesfewshot: 4,
            riskthresholdforfewshot: 0.5,
            errorratethresholdforfewshot: 0.2,
            minlatencyforfewshotms: 250,
            mintokenbudgetforfewshot: 512,
        });

        Ok(Self {
            hostid: hostid.to_string(),
            ledger: Arc::new(Mutex::new(ledger)),
            rope: Arc::new(Mutex::new(NeuralRope::new())),
            civic_profile: Arc::new(civic_profile),
            did_directory: Arc::new(did_directory),
            consent_verifier: Arc::new(consent_verifier),
            consensus: Arc::new(Mutex::new(consensus)),
            handshake_state: Arc::new(Mutex::new(handshake_state)),
            shot_policy: Arc::new(shot_policy),
            paths,
        })
    }

    pub fn persist_state(&self) -> std::io::Result<()> {
        let ledger = self.ledger.lock().unwrap();
        let state: BioTokenState = ledger.to_state();
        let raw = serde_json::to_string_pretty(&state)?;
        if let Some(parent) = self.paths.state_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.paths.state_file, raw)?;
        Ok(())
    }

    pub fn append_consensus_frame(
        &self,
        frame: &ConsensusFrame,
    ) -> std::io::Result<()> {
        if let Some(parent) = self.paths.consensus_log.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.paths.consensus_log)?;
        let line = serde_json::to_string(frame)?;
        writeln!(f, "{line}")?;
        Ok(())
    }
}

// ------------------ Runtime event pipeline (AI-Chat → inner ledger) ---------

async fn handle_runtime_event(
    state: HostNodeState,
    event: RuntimeEvent,
) -> Result<serde_json::Value, JsonRpcError> {
    // 1. Verify DID / consent (no financialization, augmented‑citizen only).
    if let Err(e) = state
        .consent_verifier
        .verify(&state.did_directory, &event.auth)
    {
        return Err(JsonRpcError {
            code: -32001,
            message: e,
        });
    }

    // 2. Civic classification and knowledge floor.
    let profile = state.civic_profile.as_ref();
    let civic_class = profile.classify(&event.civic_tags);
    if civic_class == CivicClass::Disallowed {
        return Err(JsonRpcError {
            code: -32002,
            message: "disallowed civic class".to_string(),
        });
    }
    if event.auth.knowledge_factor < profile.required_knowledge_factor {
        return Err(JsonRpcError {
            code: -32003,
            message: "knowledge factor below required threshold".to_string(),
        });
    }

    // 3. Neuro‑handshake progression (host‑local Phoenix test).
    let mut handshake_state = state.handshake_state.lock().unwrap().clone();
    if handshake_state.phase != HandshakePhase::Operation {
        handshake_state =
            NeuroHandshakeOrchestrator::apply_event(handshake_state, "userconsented");
        handshake_state =
            NeuroHandshakeOrchestrator::apply_event(handshake_state, "calibrationsamplerecorded");
        *state.handshake_state.lock().unwrap() = handshake_state.clone();
        if handshake_state.phase != HandshakePhase::Operation {
            return Err(JsonRpcError {
                code: -32004,
                message: format!(
                    "handshake not in Operation phase: {:?}",
                    handshake_state.phase
                ),
            });
        }
    }

    // 4. Eco‑aware multiplier.
    let eco_cost = event.bci_event.eco_cost_estimate;
    let multiplier =
        profile.eco_adjusted_multiplier(civic_class.clone(), eco_cost);
    let eco_band = eco_band_label(eco_cost);

    // 5. Shot‑level policy (observability only).
    let shot_signal = augdoctorpolicies::shotlevelpolicy::ShotLevelSignal {
        taskid: event.bci_event.intent_label.clone(),
        planelabel: "bci-hci-eeg".to_string(),
        riskscore: event.bci_event.risk_score,
        latencybudgetms: event.bci_event.latency_budget_ms,
        tokenbudget: event.bci_event.token_budget,
        historicalerrorrate: 0.0,
        requiresexamples: false,
    };
    let shot_decision: ShotLevelDecision =
        state.shot_policy.decide(shot_signal);

    // 6. Apply event to InnerLedger via BciLedgerOrchestrator.
    let id_header = event.auth.to_identity_header();
    let mut ledger = state.ledger.lock().unwrap();
    let mut rope = state.rope.lock().unwrap();
    let mut orchestrator =
        crate::orchestration::BciLedgerOrchestrator::new(&mut ledger, &mut rope);

    let mut adjusted_event = event.bci_event.clone();
    adjusted_event.eco_cost_estimate =
        (adjusted_event.eco_cost_estimate * multiplier)
            .min(ledger.env.eco_flops_limit);

    let handshake_copy = handshake_state.clone();
    let res = orchestrator.handle_bci_event(
        &adjusted_event,
        handshake_copy,
        &id_header,
        profile.required_knowledge_factor,
        &event.timestamp_utc,
    );

    let (ledger_result, new_handshake, _le, shot_level) = match res {
        Ok(v) => v,
        Err(e) => {
            return Err(JsonRpcError {
                code: -32005,
                message: format!("ledger_orchestrator_error:{e}"),
            });
        }
    };
    *state.handshake_state.lock().unwrap() = new_handshake.clone();

    // 7. Civic audit entry (host-local, non-financial).
    let lifeforce_ok = ledger_result.applied;

    let r = adjusted_event.risk_score.clamp(0.0, 1.0) as f64;
    let safety_factor = 1.0 - r * 0.8;
    let base_brain = 0.002_f64;
    let base_wave = 0.0015_f64;
    let base_nano = 0.0005_f64;
    let base_smart = 0.001_f64;

    let brain_delta_abs = (base_brain * safety_factor * multiplier).abs();
    let wave_delta_abs = (base_wave * safety_factor * multiplier).abs();
    let nano_delta_abs = (base_nano * safety_factor * multiplier).abs();
    let smart_delta_abs = (base_smart * safety_factor * multiplier).abs();

    let audit_entry = CivicAuditEntry {
        timestamp_utc: event.timestamp_utc.clone(),
        civic_tags: event.civic_tags.clone(),
        civic_class: civic_class.clone(),
        reward_multiplier: multiplier,
        eco_cost,
        eco_band: eco_band.clone(),
        lifeforce_ok,
        brain_delta_abs,
        wave_delta_abs,
        nano_delta_abs,
        smart_delta_abs,
        shot_level: shot_level.chosenlevel,
        handshake_phase: format!("{:?}", new_handshake.phase),
    };

    let _ = append_civic_audit_entry(
        &state.paths.civic_audit_log,
        &audit_entry,
        16_384,
    );

    // 8. Consensus frame (append‑only, non‑financial).
    let event_hash = format!("0xEVT-{}", Uuid::new_v4());
    let mut consensus = state.consensus.lock().unwrap();
    let frame = consensus.build_frame(
        &event.timestamp_utc,
        &event_hash,
        lifeforce_ok,
        eco_cost,
        &eco_band,
    );
    drop(consensus);
    let _ = state.append_consensus_frame(&frame);

    // 9. Persist inner state.
    let _ = state.persist_state();

    // 10. Redacted JSON result for AI‑Chat.
    let result = serde_json::json!({
        "hostid": state.hostid,
        "civic_class": format!("{:?}", civic_class),
        "reward_multiplier": multiplier,
        "eco_band": eco_band,
        "lifeforce_ok": lifeforce_ok,
        "shot_level": match shot_level.chosenlevel {
            ShotLevel::ZeroShot => "ZeroShot",
            ShotLevel::FewShot => "FewShot",
        },
        "handshake_phase": format!("{:?}", new_handshake.phase),
        "event_hash": event_hash,
        "consensus_frame_id": frame.frame_id,
    });

    Ok(result)
}

// ------------------ JSON-RPC dispatcher over TCP ----------------------------

async fn handle_jsonrpc(
    state: HostNodeState,
    req: JsonRpcRequest,
) -> JsonRpcResponse {
    let mut response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id.clone(),
        result: None,
        error: None,
    };

    match req.method.as_str() {
        "host.submitRuntimeEvent" => {
            let parsed: Result<RuntimeEvent, _> =
                serde_json::from_value(req.params.clone());
            match parsed {
                Ok(ev) => match handle_runtime_event(state, ev).await {
                    Ok(val) => {
                        response.result = Some(val);
                    }
                    Err(err) => {
                        response.error = Some(err);
                    }
                },
                Err(e) => {
                    response.error = Some(JsonRpcError {
                        code: -32602,
                        message: format!("invalid params: {e}"),
                    });
                }
            }
        }
        "host.getHandshakePhase" => {
            let phase = {
                let hs = state.handshake_state.lock().unwrap();
                hs.phase.clone()
            };
            response.result = Some(serde_json::json!({
                "phase": format!("{:?}", phase)
            }));
        }
        "host.getHostInfo" => {
            response.result = Some(serde_json::json!({
                "hostid": state.hostid,
                "tokens": ["BRAIN","WAVE","BLOOD","OXYGEN","NANO"],
                "financialization": false,
                "bridge_enabled": false,
                "staking_enabled": false,
            }));
        }
        _ => {
            response.error = Some(JsonRpcError {
                code: -32601,
                message: "method not found".to_string(),
            });
        }
    }

    response
}

/// Start a single‑host JSON‑RPC server for AI‑Chat platforms.
///
/// Example call:
///   { "jsonrpc":"2.0", "id":1, "method":"host.submitRuntimeEvent", "params":{...} }
pub async fn run_host_node(
    hostid: &str,
    base_dir: impl AsRef<Path>,
    bind_addr: &str,
) -> std::io::Result<JoinHandle<()>> {
    let state = HostNodeState::load_or_init(hostid, base_dir)?;
    let listener = TcpListener::bind(bind_addr).await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::AddrInUse, e))?;
    let shared = Arc::new(state);

    let handle = tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(v) => v,
                Err(_) => continue,
            };
            let state = shared.clone();
            tokio::spawn(async move {
                let mut buf = Vec::new();
                if socket.read_to_end(&mut buf).await.is_err() {
                    return;
                }
                let req: Result<JsonRpcRequest, _> =
                    serde_json::from_slice(&buf);
                let resp = match req {
                    Ok(r) => handle_jsonrpc((*state).clone(), r).await,
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: serde_json::Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("parse error: {e}"),
                        }),
                    },
                };
                let raw = serde_json::to_vec(&resp).unwrap_or_else(|_| b"{}".to_vec());
                let _ = socket.write_all(&raw).await;
            });
        }
    });

    Ok(handle)
}

// ------------------ Inner HostNode (Lorentz + BiophysicalRuntime) -----------

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

    fn convert_rpc_event(&self, rpc: RpcEvent) -> CoreRuntimeEvent {
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
            RpcEventKind::EvolutionUpgrade { evolution_id } => {
                RuntimeEventKind::EvolutionUpgrade { evolution_id }
            }
            RpcEventKind::WaveLoad { task_id, requested_wave } => {
                RuntimeEventKind::WaveLoad { task_id, requested_wave }
            }
            RpcEventKind::SmartAutonomy { agent_id, requested_smart } => {
                RuntimeEventKind::SmartAutonomy { agent_id, requested_smart }
            }
        };

        CoreRuntimeEvent {
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
