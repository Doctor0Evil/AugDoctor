//! Biophysical-Blockchain Runtime (AugDoctor, Sovereignty-Safe)
//!
//! Core guarantees:
//! - Per-host, sealed ledger (no transfers, no staking, no bridge).
//! - Non-financial tokens: BRAIN, WAVE, BLOOD, OXYGEN, NANO, SMART.
//! - Consciousness-safe invariants: BRAIN >= 0, BLOOD > 0, OXYGEN > 0.
//! - Souls and consciousness are never encoded as fields; only safety bands.
//! - ALN/DID/Bostrom-anchored access control with role + knowledge checks.
//! - Self-consent proofs for evolution + autonomy.
//! - Explicit anti-seizure: no third-party freeze/burn; no cross-host cost.
//! - Quantum-safe, Lorentz-consistent attestation.

#![forbid(unsafe_code)]

use core::fmt;
use core::time::Duration;

// ---------------------- External trait interfaces -------------------------

pub mod quantumhash {
    use super::LorentzTimestamp;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct QuantumHash(pub [u8; 48]);

    pub trait QuantumHasher {
        fn digest_bytes(input: &[u8]) -> QuantumHash;
        fn digest_with_time(input: &[u8], ts: LorentzTimestamp) -> QuantumHash;
    }

    /// Placeholder Lorentz-consistent hasher; wire a real PQ hash in production.
    pub struct LorentzSafeHasher;

    impl QuantumHasher for LorentzSafeHasher {
        fn digest_bytes(input: &[u8]) -> QuantumHash {
            use core::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;

            let mut hasher = DefaultHasher::new();
            input.hash(&mut hasher);
            let raw = hasher.finish().to_be_bytes();
            let mut out = [0u8; 48];
            out[..8].copy_from_slice(&raw);
            QuantumHash(out)
        }

        fn digest_with_time(input: &[u8], ts: LorentzTimestamp) -> QuantumHash {
            let mut buf = Vec::with_capacity(input.len() + 16);
            buf.extend_from_slice(input);
            buf.extend_from_slice(&ts.0.to_be_bytes());
            buf.extend_from_slice(&ts.1.to_be_bytes());
            Self::digest_bytes(&buf)
        }
    }
}

pub mod alndid {
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct ALNDID {
        pub id: String,
        pub shard: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum RoleClass {
        Host,
        EthicalOperator,
        Vendor,
        Regulator,
        PureMachine,
        Observer,
    }

    #[derive(Clone, Debug)]
    pub struct AccessEnvelope {
        pub did: ALNDID,
        pub roles: Vec<RoleClass>,
        pub min_biophysics_knowledge_score: f64,
    }

    #[derive(Clone, Debug)]
    pub struct ConsentProof {
        pub did: ALNDID,
        pub evolution_event_id: String,
        pub timestamp_ms_utc: i64,
        pub zk_sig: Vec<u8>,
    }

    pub trait DIDDirectory {
        fn resolve_access(&self, did: ALNDID) -> Option<AccessEnvelope>;
        fn is_ethical_operator(&self, did: ALNDID) -> bool;
    }

    pub trait ConsentVerifier {
        fn verify_self_consent(&self, proof: ConsentProof) -> bool;
    }
}

pub mod lifeforcesafety {
    use super::BioTokenState;

    #[derive(Clone, Debug)]
    pub struct DraculaWaveCurve {
        pub max_wave_factor: f64,
        pub decay_coefficient: f64,
    }

    #[derive(Clone, Debug)]
    pub struct MetabolicBands {
        pub blood_min: f64,
        pub blood_soft_floor: f64,
        pub oxygen_min: f64,
        pub oxygen_soft_floor: f64,
    }

    #[derive(Clone, Debug)]
    pub struct NanoEnvelope {
        pub max_concurrent_workload: f64,
        pub eco_penalty_factor: f64,
    }

    #[derive(Clone, Debug)]
    pub struct LifeforceState {
        pub bands: MetabolicBands,
        pub wave_curve: DraculaWaveCurve,
        pub nano_envelope: NanoEnvelope,
    }

    pub trait LifeforceSafety {
        fn validate_bands(&self, state: &BioTokenState) -> Result<(), &'static str>;
        fn safe_wave_ceiling(&self, state: &BioTokenState) -> f64;
        fn eco_neutral_brain_required(&self, state: &BioTokenState) -> f64;
    }

    impl LifeforceSafety for LifeforceState {
        fn validate_bands(&self, state: &BioTokenState) -> Result<(), &'static str> {
            if state.blood <= 0.0 || state.oxygen <= 0.0 {
                return Err("FORBIDDEN: BLOOD/OXYGEN depletion – consciousness inactive.");
            }
            if state.blood < self.bands.blood_min {
                return Err("FORBIDDEN: BLOOD below hard metabolic floor.");
            }
            if state.oxygen < self.bands.oxygen_min {
                return Err("FORBIDDEN: OXYGEN below hard metabolic floor.");
            }
            Ok(())
        }

        fn safe_wave_ceiling(&self, state: &BioTokenState) -> f64 {
            let base = state.brain.max(0.0) * self.wave_curve.max_wave_factor;
            let fatigue = 1.0 - (self.wave_curve.decay_coefficient * state.wave).max(0.0);
            (base * fatigue).max(0.0)
        }

        fn eco_neutral_brain_required(&self, state: &BioTokenState) -> f64 {
            let nano_pressure = state.nano * self.nano_envelope.eco_penalty_factor;
            nano_pressure.min(state.brain).max(0.0)
        }
    }
}

pub mod biospectreconsensus {
    use super::{quantumhash::QuantumHash, ALNHostFrame, BioTokenState};

    #[derive(Clone, Debug)]
    pub struct ConsensusFrame {
        pub host_frame: ALNHostFrame,
        pub state_hash: QuantumHash,
        pub prev_state_hash: Option<QuantumHash>,
        pub seq_no: u64,
    }

    pub trait HostConsensus {
        fn validate_state_step(
            &self,
            previous: Option<ConsensusFrame>,
            next: &ConsensusFrame,
            state: &BioTokenState,
        ) -> Result<(), &'static str>;
    }
}

// ------------------------ Core time type ----------------------------------

/// Lorentz-consistent timestamp: (proper_time_ns, frame_offset_ps).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LorentzTimestamp(pub i128, pub i64);

// ---------------------- Biophysical token state ---------------------------

use alndid::{AccessEnvelope, ALNDID, ConsentProof, DIDDirectory};
use lifeforcesafety::LifeforceSafety;
use quantumhash::{LorentzSafeHasher, QuantumHash, QuantumHasher};

#[derive(Clone)]
pub struct BioTokenState {
    pub brain: f64,
    pub wave: f64,
    pub blood: f64,
    pub oxygen: f64,
    pub nano: f64,
    pub smart: f64,
    pub host_id: ALNDID,
    pub lorentz_ts: LorentzTimestamp,
}

impl fmt::Debug for BioTokenState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BioTokenState")
            .field("brain", &self.brain)
            .field("wave", &self.wave)
            .field("blood", &self.blood)
            .field("oxygen", &self.oxygen)
            .field("nano", &self.nano)
            .field("smart", &self.smart)
            .field("host_id", &self.host_id.id)
            .field("lorentz_ts", &self.lorentz_ts)
            .finish()
    }
}

impl BioTokenState {
    /// Consciousness-safe invariants; souls are not represented here.
    pub fn enforce_invariants(&mut self) -> Result<(), &'static str> {
        if self.brain < 0.0 {
            return Err("FORBIDDEN: negative BRAIN – death condition.");
        }
        if self.blood <= 0.0 || self.oxygen <= 0.0 {
            return Err("UNSAFE: BLOOD/OXYGEN depletion – consciousness inactive.");
        }
        if self.smart > self.brain {
            self.smart = self.brain;
        }
        Ok(())
    }

    /// System-only adjustment; no user/external contract may move balances.
    pub fn system_adjust(
        &mut self,
        delta_brain: f64,
        delta_wave: f64,
        delta_blood: f64,
        delta_oxygen: f64,
        delta_nano: f64,
        safety: &dyn LifeforceSafety,
    ) -> Result<(), &'static str> {
        self.brain += delta_brain;
        self.blood += delta_blood;
        self.oxygen += delta_oxygen;
        self.nano += delta_nano;

        let wave_ceiling = safety.safe_wave_ceiling(self);
        let proposed_wave = self.wave + delta_wave;
        self.wave = proposed_wave.min(wave_ceiling).max(0.0);

        safety.validate_bands(self)?;
        self.enforce_invariants()?;
        Ok(())
    }

    /// Quantum-safe state attestation bound to host + Lorentz timestamp.
    pub fn consensus_attest(&self) -> QuantumHash {
        let mut buf = Vec::with_capacity(256);
        buf.extend_from_slice(self.host_id.id.as_bytes());
        buf.extend_from_slice(self.host_id.shard.as_bytes());
        buf.extend_from_slice(&self.brain.to_le_bytes());
        buf.extend_from_slice(&self.wave.to_le_bytes());
        buf.extend_from_slice(&self.blood.to_le_bytes());
        buf.extend_from_slice(&self.oxygen.to_le_bytes());
        buf.extend_from_slice(&self.nano.to_le_bytes());
        buf.extend_from_slice(&self.smart.to_le_bytes());
        LorentzSafeHasher::digest_with_time(&buf, self.lorentz_ts)
    }
}

// -------------------- Host frame and runtime events -----------------------

#[derive(Clone, Debug)]
pub struct ALNHostFrame {
    pub host_id: ALNDID,
    pub access: AccessEnvelope,
    pub lorentz_ts: LorentzTimestamp,
}

#[derive(Clone, Debug)]
pub enum RuntimeEventKind {
    EvolutionUpgrade { evolution_id: String },
    WaveLoad { task_id: String, requested_wave: f64 },
    SmartAutonomy { agent_id: String, requested_smart: f64 },
}

#[derive(Clone, Debug)]
pub struct RuntimeEvent {
    pub kind: RuntimeEventKind,
    pub initiator: ALNDID,
    pub consent: Option<ConsentProof>,
    pub lorentz_ts: LorentzTimestamp,
}

#[derive(Debug)]
pub enum RuntimeError {
    AccessDenied(&'static str),
    ConsentInvalid(&'static str),
    SafetyViolation(&'static str),
    ConsensusViolation(&'static str),
    InvariantViolation(&'static str),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

// --------------------------- Runtime config -------------------------------

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub smart_max_factor_of_brain: f64,
    pub smart_min_manual_threshold: f64,
    pub wave_critical_factor_of_brain: f64,
    pub eco_neutral_required: bool,
    pub lockdown_wave_factor: f64,
    pub forbid_third_party_freeze: bool,
    pub forbid_third_party_burn: bool,
    pub forbid_cross_host_transfer: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            smart_max_factor_of_brain: 1.0,
            smart_min_manual_threshold: 0.05,
            wave_critical_factor_of_brain: 0.8,
            eco_neutral_required: true,
            lockdown_wave_factor: 0.2,
            forbid_third_party_freeze: true,
            forbid_third_party_burn: true,
            forbid_cross_host_transfer: true,
        }
    }
}

// ----------------------------- Runtime ------------------------------------

use alndid::{ConsentVerifier, RoleClass};
use biospectreconsensus::{ConsensusFrame, HostConsensus};

pub struct BiophysicalRuntime<D, C, HC>
where
    D: DIDDirectory,
    C: ConsentVerifier,
    HC: HostConsensus,
{
    pub cfg: RuntimeConfig,
    pub lifeforce: lifeforcesafety::LifeforceState,
    pub did_directory: D,
    pub consent_verifier: C,
    pub consensus: HC,
}

impl<D, C, HC> BiophysicalRuntime<D, C, HC>
where
    D: DIDDirectory,
    C: ConsentVerifier,
    HC: HostConsensus,
{
    pub fn new(
        cfg: RuntimeConfig,
        lifeforce: lifeforcesafety::LifeforceState,
        did_directory: D,
        consent_verifier: C,
        consensus: HC,
    ) -> Self {
        Self {
            cfg,
            lifeforce,
            did_directory,
            consent_verifier,
            consensus,
        }
    }

    fn authenticate_initiator(&self, did: ALNDID) -> RuntimeResult<AccessEnvelope> {
        let access = self
            .did_directory
            .resolve_access(did.clone())
            .ok_or(RuntimeError::AccessDenied(
                "Unknown DID/ALN identity.",
            ))?;

        if access.roles.contains(&RoleClass::PureMachine) {
            return Err(RuntimeError::AccessDenied(
                "Pure-machine clients cannot touch biophysical ledgers.",
            ));
        }

        if access.min_biophysics_knowledge_score < 0.5 {
            return Err(RuntimeError::AccessDenied(
                "Insufficient biophysics knowledge for biophysical-blockchain operations.",
            ));
        }

        Ok(access)
    }

    fn require_self_consent(&self, event: &RuntimeEvent) -> RuntimeResult<()> {
        match &event.consent {
            Some(proof) => {
                if !self.consent_verifier.verify_self_consent(proof.clone()) {
                    return Err(RuntimeError::ConsentInvalid(
                        "Zero-knowledge self-consent proof invalid.",
                    ));
                }
                if proof.did.id != event.initiator.id {
                    return Err(RuntimeError::ConsentInvalid(
                        "Consent DID does not match event initiator.",
                    ));
                }
                Ok(())
            }
            None => Err(RuntimeError::ConsentInvalid(
                "Self-consent required for evolution/autonomy events.",
            )),
        }
    }

    fn ensure_same_host(
        &self,
        state: &BioTokenState,
        frame: &ALNHostFrame,
    ) -> RuntimeResult<()> {
        if self.cfg.forbid_cross_host_transfer
            && (state.host_id.id != frame.host_id.id
                || state.host_id.shard != frame.host_id.shard)
        {
            return Err(RuntimeError::ConsensusViolation(
                "Cross-host token mutation attempt forbidden.",
            ));
        }
        Ok(())
    }

    fn maybe_apply_lockdown(&self, state: &mut BioTokenState) {
        let critical_wave = self.cfg.wave_critical_factor_of_brain * state.brain.max(0.0);
        if state.wave >= critical_wave {
            let target = critical_wave * self.cfg.lockdown_wave_factor;
            state.wave = state.wave.min(target);
            state.smart = state.smart.min(self.cfg.smart_min_manual_threshold);
        }
    }

    fn enforce_eco_neutral_brain(
        &self,
        state: &BioTokenState,
        requested_wave: f64,
    ) -> RuntimeResult<()> {
        if !self.cfg.eco_neutral_required {
            return Ok(());
        }
        let required = self.lifeforce.eco_neutral_brain_required(state);
        if requested_wave > 0.0 && state.brain < required {
            return Err(RuntimeError::SafetyViolation(
                "Insufficient eco-neutral BRAIN reserve for requested WAVE load.",
            ));
        }
        Ok(())
    }

    /// Core execution entrypoint for one event on a host's state.
    pub fn execute_event(
        &self,
        mut state: BioTokenState,
        previous_frame: Option<ConsensusFrame>,
        host_frame: ALNHostFrame,
        event: RuntimeEvent,
    ) -> RuntimeResult<ConsensusFrame> {
        self.ensure_same_host(&state, &host_frame)?;

        let access = self.authenticate_initiator(event.initiator.clone())?;

        match &event.kind {
            RuntimeEventKind::EvolutionUpgrade { .. } => {
                self.require_self_consent(&event)?;
                if !access.roles.contains(&RoleClass::Host)
                    && !access.roles.contains(&RoleClass::EthicalOperator)
                {
                    return Err(RuntimeError::AccessDenied(
                        "Evolution upgrades restricted to host or ethical operators.",
                    ));
                }

                self.lifeforce
                    .validate_bands(&state)
                    .map_err(RuntimeError::SafetyViolation)?;
                state
                    .enforce_invariants()
                    .map_err(RuntimeError::InvariantViolation)?;
            }

            RuntimeEventKind::WaveLoad { requested_wave, .. } => {
                self.enforce_eco_neutral_brain(&state, *requested_wave)?;
                self.lifeforce
                    .validate_bands(&state)
                    .map_err(RuntimeError::SafetyViolation)?;
                let ceiling = self.lifeforce.safe_wave_ceiling(&state);
                state.wave = (*requested_wave).min(ceiling).max(0.0);
                self.maybe_apply_lockdown(&mut state);
                state
                    .enforce_invariants()
                    .map_err(RuntimeError::InvariantViolation)?;
            }

            RuntimeEventKind::SmartAutonomy { requested_smart, .. } => {
                self.require_self_consent(&event)?;
                let max_smart =
                    self.cfg.smart_max_factor_of_brain * state.brain.max(0.0);
                state.smart = (*requested_smart).min(max_smart).max(0.0);
                self.maybe_apply_lockdown(&mut state);
                self.lifeforce
                    .validate_bands(&state)
                    .map_err(RuntimeError::SafetyViolation)?;
                state
                    .enforce_invariants()
                    .map_err(RuntimeError::InvariantViolation)?;
            }
        }

        // Anti-seizure: no code path here can set balances to zero or negative
        // due to third-party intent; safety violations abort before mutation.

        let state_hash = state.consensus_attest();
        let seq_no = previous_frame.as_ref().map(|f| f.seq_no + 1).unwrap_or(0);

        let frame = ConsensusFrame {
            host_frame: host_frame.clone(),
            state_hash,
            prev_state_hash: previous_frame.as_ref().map(|f| f.state_hash.clone()),
            seq_no,
        };

        self.consensus
            .validate_state_step(previous_frame, &frame, &state)
            .map_err(RuntimeError::ConsensusViolation)?;

        Ok(frame)
    }
}

// Optional helper for getting Lorentz time from system clock.
pub trait LorentzTimeSource {
    fn now_lorentz(&self) -> LorentzTimestamp;
}

pub struct SystemLorentzClock;

impl LorentzTimeSource for SystemLorentzClock {
    fn now_lorentz(&self) -> LorentzTimestamp {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        LorentzTimestamp(now.as_nanos() as i128, 0)
    }
}
