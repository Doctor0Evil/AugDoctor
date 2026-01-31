use crate::SwarmMetrics;
use aln_core::host_risk::HostRiskScalar;

pub fn reject_if_bci_host_risk_increases(
    metrics: &SwarmMetrics,
    node_id: &str,
    before: &[HostRiskScalar],
    after: &[HostRiskScalar],
) -> bool {
    let before_idx = crate::metrics::compute_bci_host_risk_index(before);
    let after_idx = crate::metrics::compute_bci_host_risk_index(after);
    metrics.observe_host_risk(node_id, after);
    after_idx > before_idx
}
