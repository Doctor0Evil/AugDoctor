use crate::envelope::NanoswarmEnvelope;
use aln_core::host_risk::{HostRiskComponents, HostRiskScalar, HostRiskWeights};
use aln_core::{EvidenceBundle, EvidenceTag, EvidenceTagId};
use rand::Rng;

fn dummy_evidence() -> EvidenceBundle {
    EvidenceBundle {
        tags: vec![
            EvidenceTag {
                id: EvidenceTagId::PerfusionIndex,
                name: "Perfusion".into(),
                value: 1.0,
                lower_bound: 0.5,
                upper_bound: 1.5,
            },
            EvidenceTag {
                id: EvidenceTagId::ThermalMargin,
                name: "Thermal".into(),
                value: 1.0,
                lower_bound: 0.5,
                upper_bound: 1.5,
            },
            EvidenceTag {
                id: EvidenceTagId::InflammationIndex,
                name: "Inflammation".into(),
                value: 0.3,
                lower_bound: 0.0,
                upper_bound: 1.0,
            },
        ],
    }
}

#[test]
fn nanoswarm_upgrade_monotone_n() {
    let evidence = dummy_evidence();
    let base = NanoswarmEnvelope::from_evidence(&evidence);

    let weights = HostRiskWeights {
        w_e: 0.2,
        w_t: 0.2,
        w_d: 0.2,
        w_c: 0.2,
        w_n: 0.2,
    };

    let base_components = HostRiskComponents {
        e: 0.5,
        t: 0.5,
        d: 0.5,
        c: 0.5,
        n: base.n_risk_scalar,
    };

    let base_scalar = HostRiskScalar::from_components(weights, base_components);

    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let improved_n = (base.n_risk_scalar - rng.gen_range(0.0..0.1)).max(0.0);
        let next_components = HostRiskComponents {
            n: improved_n,
            ..base_components
        };
        let next_scalar =
            HostRiskScalar::from_components(weights, next_components);
        assert!(base_scalar.is_monotone_non_increasing(next_scalar));
    }
}
