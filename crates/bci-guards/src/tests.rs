use crate::manifest::generate_daily_manifest;
use aln_core::host_risk::{HostRiskComponents, HostRiskScalar, HostRiskWeights};
use aln_core::{AlnManifest, EvidenceBundle, EvidenceTag, EvidenceTagId};
use rand::Rng;

fn dummy_evidence() -> EvidenceBundle {
    EvidenceBundle {
        tags: vec![
            EvidenceTag {
                id: EvidenceTagId::Cmro2,
                name: "Cmro2".into(),
                value: 1.0,
                lower_bound: 0.5,
                upper_bound: 1.5,
            },
            EvidenceTag {
                id: EvidenceTagId::Hrv,
                name: "HRV".into(),
                value: 0.8,
                lower_bound: 0.0,
                upper_bound: 1.0,
            },
            EvidenceTag {
                id: EvidenceTagId::FatigueIndex,
                name: "Fatigue".into(),
                value: 0.4,
                lower_bound: 0.0,
                upper_bound: 1.0,
            },
            EvidenceTag {
                id: EvidenceTagId::StressIndex,
                name: "Stress".into(),
                value: 0.3,
                lower_bound: 0.0,
                upper_bound: 1.0,
            },
            EvidenceTag {
                id: EvidenceTagId::SleepDebt,
                name: "SleepDebt".into(),
                value: 0.2,
                lower_bound: 0.0,
                upper_bound: 1.0,
            },
        ],
    }
}

#[test]
fn host_risk_monotone_over_sessions() {
    let evidence = dummy_evidence();
    let manifest = generate_daily_manifest(evidence.clone());

    let weights = HostRiskWeights {
        w_e: 0.2,
        w_t: 0.2,
        w_d: 0.2,
        w_c: 0.2,
        w_n: 0.2,
    };

    let mut rng = rand::thread_rng();
    let mut previous = HostRiskScalar::from_components(
        weights,
        HostRiskComponents {
            e: 0.5,
            t: 0.5,
            d: 0.5,
            c: 0.5,
            n: 0.5,
        },
    );

    for _ in 0..100 {
        let delta_e = rng.gen_range(-0.1..0.0);
        let delta_t = rng.gen_range(-0.1..0.0);
        let delta_d = rng.gen_range(-0.1..0.0);
        let delta_c = rng.gen_range(-0.1..0.0);
        let delta_n = rng.gen_range(-0.1..0.0);

        let components = HostRiskComponents {
            e: (previous.components.e + delta_e).max(0.0),
            t: (previous.components.t + delta_t).max(0.0),
            d: (previous.components.d + delta_d).max(0.0),
            c: (previous.components.c + delta_c).max(0.0),
            n: (previous.components.n + delta_n).max(0.0),
        };

        let next = HostRiskScalar::from_components(weights, components);
        assert!(previous.is_monotone_non_increasing(next));
        assert!(previous.has_strict_improvement(next));
        previous = next;
    }

    assert!(manifest.evidence_bundle.ensure_within_bounds());
}
