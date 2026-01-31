use crate::evidence::{EvidenceBundle, EvidenceTagId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlnClauseId {
    Biosafeguard,
    Ecocontract,
    Privacyscope,
    NoCorticalActuation,
    CognitiveRestWindow,
    NanoswarmClearanceGuard,
    LocalDecodeRequired,
    ObservabilityFloor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricBinding {
    pub metric_name: String,
    pub evidence_tags: Vec<EvidenceTagId>,
    pub envelope_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnClause {
    pub id: AlnClauseId,
    pub description: String,
    pub bindings: Vec<MetricBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MayThisRunSummary {
    pub proof_artifact_hashes: Vec<String>,
    pub test_harness_hash: String,
    pub required_dids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlnManifest {
    pub date: String,
    pub domain: String,
    pub evidence_bundle: EvidenceBundle,
    pub envelopes: serde_json::Value,
    pub clauses: Vec<AlnClause>,
    pub may_this_run: MayThisRunSummary,
    pub spec_identifiers: BTreeSet<String>,
}

impl AlnManifest {
    pub fn ensure_monotone_extension(
        &self,
        previous: &AlnManifest,
    ) -> Result<(), String> {
        if !self
            .spec_identifiers
            .is_superset(&previous.spec_identifiers)
        {
            return Err("spec set is not a superset of previous cycle".into());
        }
        let diff: BTreeSet<_> = self
            .spec_identifiers
            .difference(&previous.spec_identifiers)
            .collect();
        if diff.is_empty() {
            return Err("no new spec identifiers added in this cycle".into());
        }
        Ok(())
    }

    pub fn add_identifier(&mut self, id: impl Into<String>) {
        self.spec_identifiers.insert(id.into());
    }
}
