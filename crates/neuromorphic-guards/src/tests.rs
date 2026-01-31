use crate::envelope::NeuromorphicEnvelope;
use aln_core::host_risk::{HostRiskComponents, HostRiskScalar, HostRiskWeights};
use aln_core::{EvidenceBundle, EvidenceTag, EvidenceTagId};
use rand::Rng;

fn dummy_evidence() -> EvidenceBundle {
    EvidenceBundle {
        tags: vec![
            EvidenceTag {
                id: EvidenceTagId::NeuromorphicEnergyIndex,
                name: "NeuroEnergy".into(),
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
        ],
    }
}

#[test]
fn neuromorphic_energy_per_inference_decreases() {
    let evidence = dummy_evidence();
    let envelope = NeuromorphicEnvelope::from_evidence(&evidence);

    let base_e = NeuromorphicEnvelope::energy_per_inference(1.0, 1.0, 10.0);
    let improved_e = NeuromorphicEnvelope::energy_per_inference(0.8, 0.9, 12.0);
    assert!(improved_e <= base_e);

    let weights = HostRiskWeights {
        w_e: 0.2,
        w_t: 0.2,
        w_d: 0.2,
        w_c: 0.2,
        w_n: 0.2,
    };
    let base_components = HostRiskComponents {
        e: base_e,
        t: envelope.max_power_density,
        d: 0.5,
        c: 0.5,
        n: 0.5,
    };
    let improved_components = HostRiskComponents {
        e: improved_e,
        ..base_components
    };

    let base_scalar = HostRiskScalar::from_components(weights, base_components);
    let improved_scalar =
        HostRiskScalar::from_components(weights, improved_components);
    assert!(base_scalar.is_monotone_non_increasing(improved_scalar));
}
