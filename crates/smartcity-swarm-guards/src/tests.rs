use crate::manifest::compute_bci_host_risk_index;
use crate::SwarmNodeEnvelope;
use aln_core::host_risk::{HostRiskComponents, HostRiskScalar, HostRiskWeights};

#[test]
fn host_risk_index_monotone() {
    let weights = HostRiskWeights {
        w_e: 0.2,
        w_t: 0.2,
        w_d: 0.2,
        w_c: 0.2,
        w_n: 0.2,
    };

    let base_hosts: Vec<_> = (0..10)
        .map(|i| {
            HostRiskScalar::from_components(
                weights,
                HostRiskComponents {
                    e: 0.5 - 0.01 * i as f64,
                    t: 0.5,
                    d: 0.5,
                    c: 0.5,
                    n: 0.5,
                },
            )
        })
        .collect();

    let improved_hosts: Vec<_> = base_hosts
        .iter()
        .enumerate()
        .map(|(i, h)| {
            HostRiskScalar::from_components(
                weights,
                HostRiskComponents {
                    e: (h.components.e - 0.05).max(0.0),
                    ..h.components
                },
            )
        })
        .collect();

    let base_idx = compute_bci_host_risk_index(&base_hosts);
    let improved_idx = compute_bci_host_risk_index(&improved_hosts);
    assert!(improved_idx <= base_idx);
}

#[test]
fn envelope_respects_blind_window() {
    let env = SwarmNodeEnvelope::default_for_city();
    assert!(env.max_blind_window <= 5.0);
}
