use crate::NanoswarmEnvelope;
use aln_core::aln_spec::{
    AlnClause, AlnClauseId, AlnManifest, MayThisRunSummary, MetricBinding,
};
use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use chrono::Utc;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
struct NanoswarmEnvelopes {
    nanoswarm: NanoswarmEnvelope,
}

pub fn generate_daily_manifest(evidence_bundle: EvidenceBundle) -> AlnManifest {
    let envelope = NanoswarmEnvelope::from_evidence(&evidence_bundle);
    let envelopes =
        serde_json::to_value(NanoswarmEnvelopes { nanoswarm: envelope })
            .expect("serialize envelopes");

    let nanoswarm_clearance = AlnClause {
        id: AlnClauseId::NanoswarmClearanceGuard,
        description: "Nanoswarm plans must satisfy evidence-bounded clearance half life.",
        bindings: vec![MetricBinding {
            metric_name: "clearance_half_life".into(),
            evidence_tags: vec![EvidenceTagId::PerfusionIndex],
            envelope_fields: vec!["nanoswarm.clearance_half_life_max".into()],
        }],
    };

    let may_this_run = MayThisRunSummary {
        proof_artifact_hashes: vec!["0x8c2f1d97".into()],
        test_harness_hash: "nanoswarm-guards-tests-v1".into(),
        required_dids: vec![
            "did:bostrom:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
        ],
    };

    let mut spec_identifiers = BTreeSet::new();
    spec_identifiers.insert("envelope:nanoswarm:nanoswarm_v1".into());
    spec_identifiers.insert("clause:NanoswarmClearanceGuard".into());

    AlnManifest {
        date: Utc::now().format("%Y-%m-%d").to_string(),
        domain: "nanoswarm".into(),
        evidence_bundle,
        envelopes,
        clauses: vec![nanoswarm_clearance],
        may_this_run,
        spec_identifiers,
    }
}
