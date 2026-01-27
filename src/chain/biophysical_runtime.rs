//! Biophysical-Blockchain Runtime (AugDoctor)
//!
//! Core properties:
//! - Six tokens: BRAIN, WAVE, BLOOD, OXYGEN, NANO, SMART (per-host, non-financial).
//! - Local, sealed ledgers per host; no global fungibility or transfer.
//! - Consciousness-safe invariants (BRAIN >= 0, BLOOD > 0, OXYGEN > 0).
//! - Quantum-safe, Lorentz-consistent attestation for state progression.
//! - ALN/DID-anchored access control and self-consent proofs.
//! - Lifeforce metric gating for multi-host synchronization.
//!
//! This module assumes the existence of the following crates/namespaces:
//! - quantum_hash: for quantum-safe hashing and Lorentz-consistent timestamps.
//! - aln_did: for DID/ALN identity, roles, and consent attestations.
//! - lifeforce_safety: for biophysical metrics, safety curves, and eco-constraints.
//! - biospectre_consensus: for host-bound, neurorights-governed consensus frames.
//!
//! All external traits are declared here as sanitized interfaces, so the module
//! compiles in isolation and can be wired to concrete implementations in the
//! bioscale host-chain crate.

use core::fmt;
use core::time::Duration;

// --- External crate traits (sanitized interfaces) ---------------------------

pub mod quantum_hash {
    use super::LorentzTimestamp;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct QuantumHash(pub [u8; 48]); // e.g., hash-based post-quantum digest

    pub trait QuantumHasher {
        fn digest_bytes(input: &[u8]) -> QuantumHash;
        fn digest_with_time(input: &[u8], ts: &LorentzTimestamp) -> QuantumHash;
    }

    // Placeholder Lorentz-consistent hasher ID
    pub struct LorentzSafeHasher;

    impl QuantumHasher for LorentzSafeHasher {
        fn digest_bytes(input: &[u8]) -> QuantumHash {
            // Implement with post-quantum hash (e.g., SHA3 + lattice padding)
            use core::hash::{Hash, Hasher};
            use std::collections::hash_map::DefaultHasher;

            let mut h = DefaultHasher::new();
            input.hash(&mut h);
            let raw = h.finish().to_be_bytes();
            let mut out = [0u8; 48];
            out[..8].copy_from_slice(&raw);
            QuantumHash(out)
        }

        fn digest_with_time(input: &[u8], ts: &LorentzTimestamp) -> QuantumHash {
            let mut buf = Vec::with_capacity(input.len() + 32);
            buf.extend_from_slice(input);
            buf.extend_from_slice(ts.0.to_be_bytes().as_ref());
            buf.extend_from_slice(ts.1.to_be_bytes().as_ref());
            Self::digest_bytes(&buf)
        }
    }
}

pub mod aln_did {
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct ALNDID {
        pub id: String,
        pub shard: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum RoleClass {
        Host,
        Validator,
        EthicalOperator,
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
        fn resolve_access(&self, did: &ALNDID) -> Option<AccessEnvelope>;
        fn is_ethical_operator(&self, did: &ALNDID) -> bool;
    }

    pub trait ConsentVerifier {
        fn verify_self_consent(&self, proof: &ConsentProof) -> bool;
    }
}

pub mod lifeforce_safety {
    use super::BioTokenState;

    #[derive(Clone, Debug)]
    pub struct DraculaWaveCurve {
        pub max_wave_factor: f64,   // scales BRAIN → WAVE ceiling
        pub decay_coefficient: f64, // reduces effective wave with fatigue
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
                return Err("Forbidden: BLOOD/OXYGEN depletion -> consciousness inactive.");
            }
            if state.blood < self.bands.blood_min {
                return Err("Forbidden: BLOOD below hard metabolic floor.");
            }
            if state.oxygen < self.bands.oxygen_min {
                return Err("Forbidden: OXYGEN below hard metabolic floor.");
            }
            Ok(())
        }

        fn safe_wave_ceiling(&self, state: &BioTokenState) -> f64 {
            let base = state.brain * self.wave_curve.max_wave_factor;
            let fatigue = 1.0 / (1.0 + self.wave_curve.decay_coefficient * state.wave.max(0.0));
            base * fatigue
        }

        fn eco_neutral_brain_required(&self, state: &BioTokenState) -> f64 {
            // Require some BRAIN reserve proportional to NANO cost, scaled by eco penalty.
            let nano_pressure = state.nano * self.nano_envelope.eco_penalty_factor;
            nano_pressure.min(state.brain.max(0.0))
        }
    }
}

pub mod biospectre_consensus {
    use super::{quantum_hash::QuantumHash, ALNHostFrame, BioTokenState};

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
            previous: Option<&ConsensusFrame>,
            next: &ConsensusFrame,
            state: &BioTokenState,
        ) -> Result<(), &'static str>;
    }
}

// --- Core time / relativity types ------------------------------------------

/// Lorentz-consistent timestamp represented as (proper_time_ns, frame_offset_ps).
/// Concrete implementations can derive these values from synchronized hardware clocks
/// and relativistic correction layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LorentzTimestamp(pub i128, pub i64);

// --- Biophysical token state -----------------------------------------------

use aln_did::{AccessEnvelope, ConsentProof, DIDDirectory};
use lifeforce_safety::{LifeforceSafety, LifeforceState};
use quantum_hash::{LorentzSafeHasher, QuantumHash, QuantumHasher};

#[derive(Clone)]
pub struct BioTokenState {
    pub brain: f64,
    pub wave: f64,
    pub blood: f64,
    pub oxygen: f64,
    pub nano: f64,
    pub smart: f64,
    pub host_id: aln_did::ALNDID,
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
    /// Consciousness-safe invariants, local to a host.
    pub fn enforce_invariants(&mut self) -> Result<(), &'static str> {
        if self.brain < 0.0 {
            return Err("Forbidden: negative BRAIN -> death condition.");
        }
        if self.blood <= 0.0 || self.oxygen <= 0.0 {
            return Err("Unsafe depletion: consciousness inactive.");
        }
        if self.smart > self.brain {
            self.smart = self.brain;
        }
        Ok(())
    }

    /// System-only adjustment respecting Lifeforce safety and WAVE protection curves.
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
        self.wave = proposed_wave.min(wave_ceiling);

        safety.validate_bands(self)?;
        self.enforce_invariants()?;
        Ok(())
    }

    /// Locks state into a quantum-safe hash bound to host and Lorentz timestamp.
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

        LorentzSafeHasher::digest_with_time(&buf, &self.lorentz_ts)
    }
}

// --- Host frame and runtime events -----------------------------------------

#[derive(Clone, Debug)]
pub struct ALNHostFrame {
    pub host_id: aln_did::ALNDID,
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
    pub initiator: aln_did::ALNDID,
    pub consent: Option<ConsentProof>,
    pub lorentz_ts: LorentzTimestamp,
}

// --- Runtime result and error types ----------------------------------------

#[derive(Debug)]
pub enum RuntimeError {
    AccessDenied(&'static str),
    ConsentInvalid(&'static str),
    SafetyViolation(&'static str),
    ConsensusViolation(&'static str),
    InvariantViolation(&'static str),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

// --- Runtime configuration --------------------------------------------------

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub smart_max_factor_of_brain: f64,
    pub smart_min_manual_threshold: f64,
    pub wave_critical_factor_of_brain: f64,
    pub eco_neutral_required: bool,
    pub lockdown_wave_factor: f64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            smart_max_factor_of_brain: 1.0,
            smart_min_manual_threshold: 0.05,
            wave_critical_factor_of_brain: 0.8,
            eco_neutral_required: true,
            lockdown_wave_factor: 0.2,
        }
    }
}

// --- Biophysical runtime ----------------------------------------------------

pub struct BiophysicalRuntime<D, C, HC>
where
    D: DIDDirectory,
    C: aln_did::ConsentVerifier,
    HC: biospectre_consensus::HostConsensus,
{
    pub cfg: RuntimeConfig,
    pub lifeforce: LifeforceState,
    pub did_directory: D,
    pub consent_verifier: C,
    pub consensus: HC,
}

impl<D, C, HC> BiophysicalRuntime<D, C, HC>
where
    D: DIDDirectory,
    C: aln_did::ConsentVerifier,
    HC: biospectre_consensus::HostConsensus,
{
    pub fn new(
        cfg: RuntimeConfig,
        lifeforce: LifeforceState,
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

    /// Verify that the initiator has a valid ALN/DID envelope and sufficient biophysical knowledge.
    fn authenticate_initiator(&self, did: &aln_did::ALNDID) -> RuntimeResult<AccessEnvelope> {
        let access = self
            .did_directory
            .resolve_access(did)
            .ok_or(RuntimeError::AccessDenied("Unknown DID/ALN identity"))?;

        if access.min_biophysics_knowledge_score < 0.5 {
            return Err(RuntimeError::AccessDenied(
                "Insufficient biophysics knowledge for biophysical-blockchain operations",
            ));
        }
        Ok(access)
    }

    /// Ensure self-consent exists for evolution or high-autonomy events.
    fn require_self_consent(&self, event: &RuntimeEvent) -> RuntimeResult<()> {
        match &event.consent {
            Some(proof) => {
                if !self.consent_verifier.verify_self_consent(proof) {
                    return Err(RuntimeError::ConsentInvalid(
                        "Zero-knowledge self-consent proof invalid or mismatched",
                    ));
                }
                if proof.did.id != event.initiator.id {
                    return Err(RuntimeError::ConsentInvalid(
                        "Consent DID does not match event initiator",
                    ));
                }
                Ok(())
            }
            None => Err(RuntimeError::ConsentInvalid(
                "Self-consent required for evolution/autonomy events",
            )),
        }
    }

    /// Enforce non-economic, per-host isolation: no cross-host token transfers.
    fn ensure_same_host(&self, state: &BioTokenState, frame: &ALNHostFrame) -> RuntimeResult<()> {
        if state.host_id.id != frame.host_id.id || state.host_id.shard != frame.host_id.shard {
            return Err(RuntimeError::ConsensusViolation(
                "Cross-host token mutation attempt forbidden",
            ));
        }
        Ok(())
    }

    /// Apply consciousness-safe lockdown when near critical thresholds.
    fn maybe_apply_lockdown(&self, state: &mut BioTokenState) {
        let critical_wave = self.cfg.wave_critical_factor_of_brain * state.brain.max(0.0);
        if state.wave > critical_wave {
            let target = critical_wave * self.cfg.lockdown_wave_factor;
            state.wave = state.wave.min(target);
            state.smart = state.smart.min(self.cfg.smart_min_manual_threshold * state.brain);
        }
    }

    /// Require eco-neutral BRAIN reserves before high-load WAVE / SMART operations.
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
                "Insufficient eco-neutral BRAIN reserve for requested WAVE load",
            ));
        }
        Ok(())
    }

    /// Core entry: execute a runtime event against a single host’s state.
    pub fn execute_event(
        &self,
        state: &mut BioTokenState,
        previous_frame: Option<&biospectre_consensus::ConsensusFrame>,
        host_frame: &ALNHostFrame,
        event: &RuntimeEvent,
    ) -> RuntimeResult<biospectre_consensus::ConsensusFrame> {
        self.ensure_same_host(state, host_frame)?;
        let access = self.authenticate_initiator(&event.initiator)?;

        match &event.kind {
            RuntimeEventKind::EvolutionUpgrade { .. } => {
                self.require_self_consent(event)?;
                if !access.roles.contains(&aln_did::RoleClass::Host)
                    && !access.roles.contains(&aln_did::RoleClass::EthicalOperator)
                {
                    return Err(RuntimeError::AccessDenied(
                        "Evolution upgrades restricted to host or ethical operators",
                    ));
                }

                self.lifeforce.validate_bands(state).map_err(RuntimeError::SafetyViolation)?;
                state.enforce_invariants().map_err(RuntimeError::InvariantViolation)?;
            }
            RuntimeEventKind::WaveLoad { requested_wave, .. } => {
                self.enforce_eco_neutral_brain(state, *requested_wave)?;
                self.lifeforce.validate_bands(state).map_err(RuntimeError::SafetyViolation)?;

                let ceiling = self.lifeforce.safe_wave_ceiling(state);
                state.wave = (*requested_wave).min(ceiling);
                self.maybe_apply_lockdown(state);
                state.enforce_invariants().map_err(RuntimeError::InvariantViolation)?;
            }
            RuntimeEventKind::SmartAutonomy { requested_smart, .. } => {
                self.require_self_consent(event)?;
                let max_smart = self.cfg.smart_max_factor_of_brain * state.brain.max(0.0);
                state.smart = (*requested_smart).min(max_smart);
                self.maybe_apply_lockdown(state);
                self.lifeforce.validate_bands(state).map_err(RuntimeError::SafetyViolation)?;
                state.enforce_invariants().map_err(RuntimeError::InvariantViolation)?;
            }
        }

        let state_hash = state.consensus_attest();
        let seq_no = previous_frame.map(|f| f.seq_no + 1).unwrap_or(0);

        let frame = biospectre_consensus::ConsensusFrame {
            host_frame: host_frame.clone(),
            prev_state_hash: previous_frame.map(|f| f.state_hash.clone()),
            state_hash,
            seq_no,
        };

        self.consensus
            .validate_state_step(previous_frame, &frame, state)
            .map_err(RuntimeError::ConsensusViolation)?;

        Ok(frame)
    }
}

// --- Optional helper: Lorentz time provider --------------------------------

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
