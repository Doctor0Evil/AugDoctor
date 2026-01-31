use aln_core::host_risk::HostRiskScalar;
use prometheus::{register_gauge_vec, GaugeVec};
use smartcity_swarm_guards::compute_bci_host_risk_index;

pub struct SwarmMetrics {
    pub swarm_node_duty_ratio: GaugeVec,
    pub swarm_node_blind_seconds_total: GaugeVec,
    pub eco_impact_score: GaugeVec,
    pub bci_host_risk_index: GaugeVec,
}

impl SwarmMetrics {
    pub fn new() -> Self {
        let swarm_node_duty_ratio = register_gauge_vec!(
            "swarm_node_duty_ratio",
            "Duty ratio of swarm node",
            &["node_id", "district"]
        )
        .unwrap();

        let swarm_node_blind_seconds_total = register_gauge_vec!(
            "swarm_node_blind_seconds_total",
            "Total blind seconds per node",
            &["node_id"]
        )
        .unwrap();

        let eco_impact_score = register_gauge_vec!(
            "eco_impact_score",
            "Eco impact score per node",
            &["node_id", "district"]
        )
        .unwrap();

        let bci_host_risk_index = register_gauge_vec!(
            "bci_host_risk_index",
            "Average host risk scalar per node",
            &["node_id"]
        )
        .unwrap();

        Self {
            swarm_node_duty_ratio,
            swarm_node_blind_seconds_total,
            eco_impact_score,
            bci_host_risk_index,
        }
    }

    pub fn observe_host_risk(
        &self,
        node_id: &str,
        hosts: &[HostRiskScalar],
    ) {
        let idx = compute_bci_host_risk_index(hosts);
        self.bci_host_risk_index
            .with_label_values(&[node_id])
            .set(idx);
    }
}
