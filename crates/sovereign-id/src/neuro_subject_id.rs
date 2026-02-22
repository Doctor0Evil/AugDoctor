use serde::{Serialize, Deserialize};

/// Canonical sovereign identity for an augmented citizen.
/// Single instance per root_did across all planes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NeuroSubjectId {
    /// Root DID: Bostrom/ALN only.
    pub root_did: String,      // e.g. "bostrom18sd2u..." or "didaln:..."
    /// Inner-ledger binding: immutable host ledger identifier.
    pub host_ledger_id: String, // e.g. hash of inner-ledger genesis block.
    /// Jurisdiction/microspace tag (corridor, swarm, neuromorph zone).
    pub jurisdiction_tag: String,
    /// Versioning for future migrations (non-forking).
    pub version: u32,
}

/// Non-cloneable, non-mergeable profile envelope.
/// Exactly one per NeuroSubjectId, sealed outside core crate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SovereignProfileEnvelope {
    pub subject_id: NeuroSubjectId,
    /// Pointer into ALN shards that hold doctrine, microspace, and rights.
    pub aln_profile_ref: String,   // e.g. "sovereign.identity.profile.v1.aln"
    /// Cryptographic attestation bundle (DID signatures, ledger proofs).
    pub attestation_ref: String,   // e.g. hash of inner-ledger proof shard
}

// In lib.rs of this crate, mark the constructor as non-public to prevent arbitrary minting.
// Only the inner-ledger bootstrap can create one SovereignProfileEnvelope per NeuroSubjectId.
