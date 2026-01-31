use crate::NeuromorphicEnvelope;
use aln_core::aln_spec::{
    AlnClause, AlnClauseId, AlnManifest, MayThisRunSummary, MetricBinding,
};
use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use chrono::Utc;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
struct NeuroEnvelopes {
    neuromorphic: NeuromorphicEnvelope,
}

pub fn generate_daily_manifest(evidence_bundle: EvidenceBundle) -> AlnManifest {
    let envelope = NeuromorphicEnvelope::from_evidence(&evidence_bundle);
    let envelopes =
        serde_json::to_value(NeuroEnvelopes { neuromorphic: envelope })
            .expect("serialize envelopes");

    let local_decode_required = AlnClause {
        id: AlnClauseId::LocalDecodeRequired,
        description:
            "Critical BCI decode channels must remain locally decodable with bounded latency and power.",
        bindings: vec![MetricBinding {
            metric_name: "decode_path".into(),
            evidence_tags: vec![EvidenceTagId::NeuromorphicEnergyIndex],
            envelope_fields: vec!["neuromorphic.local_decode_only".into()],
        }],
    };

    let may_this_run = MayThisRunSummary {
        proof_artifact_hashes: vec!["0x8c2f1d97".into()],
        test_harness_hash: "neuromorphic-guards-tests-v1".into(),
        required_dids: vec![
            "did:bostrom:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
        ],
    };

    let mut spec_identifiers = BTreeSet::new();
    spec_identifiers.insert("envelope:neuromorphic:neuromorphic_v1".into());
    spec_identifiers.insert("clause:LocalDecodeRequired".into());

    AlnManifest {
        date: Utc::now().format("%Y-%m-%d").to_string(),
        domain: "neuro".into(),
        evidence_bundle,
        envelopes,
        clauses: vec![local_decode_required],
        may_this_run,
        spec_identifiers,
    }
}
