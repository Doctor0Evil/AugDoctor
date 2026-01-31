use crate::SwarmNodeEnvelope;
use aln_core::aln_spec::{
    AlnClause, AlnClauseId, AlnManifest, MayThisRunSummary, MetricBinding,
};
use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use aln_core::host_risk::HostRiskScalar;
use chrono::Utc;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
struct SmartcityEnvelopes {
    swarm_node: SwarmNodeEnvelope,
}

pub fn compute_bci_host_risk_index(hosts: &[HostRiskScalar]) -> f64 {
    if hosts.is_empty() {
        return 0.0;
    }
    let sum: f64 = hosts.iter().map(|h| h.v_host).sum();
    sum / hosts.len() as f64
}

pub fn generate_daily_manifest(
    evidence_bundle: EvidenceBundle,
) -> AlnManifest {
    let envelope = SwarmNodeEnvelope::default_for_city();
    let envelopes =
        serde_json::to_value(SmartcityEnvelopes { swarm_node: envelope })
            .expect("serialize envelopes");

    let observability_floor = AlnClause {
        id: AlnClauseId::ObservabilityFloor,
        description:
            "Smart-city nodes must maintain observability bounds tied to host rollback energy and perfusion.",
        bindings: vec![MetricBinding {
            metric_name: "max_blind_window".into(),
            evidence_tags: vec![
                EvidenceTagId::EcoImpactScoreBaseline,
                EvidenceTagId::PerfusionIndex,
                EvidenceTagId::ThermalMargin,
            ],
            envelope_fields: vec!["swarm_node.max_blind_window".into()],
        }],
    };

    let may_this_run = MayThisRunSummary {
        proof_artifact_hashes: vec!["0x8c2f1d97".into()],
        test_harness_hash: "smartcity-guards-tests-v1".into(),
        required_dids: vec![
            "did:bostrom:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
        ],
    };

    let mut spec_identifiers = BTreeSet::new();
    spec_identifiers.insert("envelope:smartcity:swarm_node_v1".into());
    spec_identifiers.insert("clause:ObservabilityFloor".into());

    AlnManifest {
        date: Utc::now().format("%Y-%m-%d").to_string(),
        domain: "smartcity".into(),
        evidence_bundle,
        envelopes,
        clauses: vec![observability_floor],
        may_this_run,
        spec_identifiers,
    }
}
