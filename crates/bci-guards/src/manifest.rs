use crate::{CognitiveLoadEnvelopeV2, MuscleSafetyEnvelopeV2};
use aln_core::aln_spec::{
    AlnClause, AlnClauseId, AlnManifest, MayThisRunSummary, MetricBinding,
};
use aln_core::evidence::{EvidenceBundle, EvidenceTagId};
use chrono::Utc;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Serialize)]
struct BciEnvelopes {
    cognitive_load_v2: CognitiveLoadEnvelopeV2,
    muscle_safety_v2: MuscleSafetyEnvelopeV2,
}

pub fn generate_daily_manifest(evidence_bundle: EvidenceBundle) -> AlnManifest {
    let cognitive = CognitiveLoadEnvelopeV2::from_evidence(&evidence_bundle);
    let muscle = MuscleSafetyEnvelopeV2::from_evidence(&evidence_bundle);

    let envelopes = serde_json::to_value(BciEnvelopes {
        cognitive_load_v2: cognitive,
        muscle_safety_v2: muscle,
    })
    .expect("serialize envelopes");

    let mut clauses = Vec::new();

    let nocortical = AlnClause {
        id: AlnClauseId::NoCorticalActuation,
        description: "BCI decoder must not drive cortical stimulation actuators.",
        bindings: vec![MetricBinding {
            metric_name: "bci_actuation_mode".into(),
            evidence_tags: vec![EvidenceTagId::InflammationIndex],
            envelope_fields: vec!["cortical_actuation_allowed".into()],
        }],
    };
    clauses.push(nocortical);

    let cognitive_rest = AlnClause {
        id: AlnClauseId::CognitiveRestWindow,
        description:
            "BCI sessions must schedule microbreaks so that cumulative cognitive load remains within evidence-anchored bounds.",
        bindings: vec![MetricBinding {
            metric_name: "microbreak_interval".into(),
            evidence_tags: vec![
                EvidenceTagId::FatigueIndex,
                EvidenceTagId::Hrv,
                EvidenceTagId::SleepDebt,
            ],
            envelope_fields: vec!["cognitive_load_v2.microbreak_interval_min".into()],
        }],
    };
    clauses.push(cognitive_rest);

    let may_this_run = MayThisRunSummary {
        proof_artifact_hashes: vec!["0x8c2f1d97".into()],
        test_harness_hash: "bci-guards-tests-v1".into(),
        required_dids: vec![
            "did:bostrom:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".into(),
        ],
    };

    let mut spec_identifiers = BTreeSet::new();
    spec_identifiers.insert("envelope:bci:cognitive_load_v2".into());
    spec_identifiers.insert("envelope:bci:muscle_safety_v2".into());
    spec_identifiers.insert("clause:NoCorticalActuation".into());
    spec_identifiers.insert("clause:CognitiveRestWindow".into());

    AlnManifest {
        date: Utc::now().format("%Y-%m-%d").to_string(),
        domain: "bci".into(),
        evidence_bundle,
        envelopes,
        clauses,
        may_this_run,
        spec_identifiers,
    }
}
