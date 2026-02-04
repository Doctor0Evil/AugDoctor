#![forbid(unsafe_code)]

use aln_core::{AlnId, AlnShardId};
use bioscale_core::{
    HostBudget,
    ComplianceError,
};
use cybernano_snrgate::UcnSnrGate;
use cybernano_energy::UcnEnergyManager;
use cybernano_risk::UcnRiskMonitor;
use cybernano_density::UcnDensityGuard;
use cybernano_connectivity::UcnConnectivityGuard;
use cybernano_coverage::UcnCoverageMonitor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XRHostSnapshot {
    pub host_id: String,
    pub hrv_ms: f32,
    pub thermo_c: f32,
    pub duty_fraction: f32,
    pub fps: f32,
    pub fov_deg: f32,
    pub snr_db: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct XRSessionEnvelope {
    pub session_id: String,
    pub snr_min_db: f32,
    pub e_max_j: f64,
    pub duty_max: f32,
    pub latency_max_ms: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum XRData {
    HostSnapshot(XRHostSnapshot),
    CorridorEnvelope(XRSessionEnvelope),
    // extend with the rest of your 25 actions as needed
}

#[derive(Clone, Debug)]
pub enum XRAction {
    HostSnapshot,
    CorridorEnvelope,
}

#[derive(Clone, Debug)]
pub struct XRParams {
    pub host_id: Option<String>,
    pub session_id: Option<String>,
    pub aln_policy_shard: Option<AlnShardId>,
}

/// Core retrieval entrypoint: ALN-gated, no normal reversal allowed.
pub fn retrieve_xr_data(
    action: XRAction,
    params: XRParams,
    host_budget: &HostBudget,
) -> Result<XRData, ComplianceError> {
    // 1. Load thresholds from XRRetrieval.aln (snr_min, e_max, latency_max, etc.)
    let aln_cfg = cybernano_aln_loader::load_xr_retrieval_policy(params.aln_policy_shard)?;
    let snr_gate = UcnSnrGate::from_aln(&aln_cfg);
    let energy_mgr = UcnEnergyManager::from_aln(&aln_cfg);
    let risk_mon = UcnRiskMonitor::from_aln(&aln_cfg);
    let density_guard = UcnDensityGuard::from_aln(&aln_cfg);
    let conn_guard = UcnConnectivityGuard::from_aln(&aln_cfg);
    let cov_mon = UcnCoverageMonitor::from_aln(&aln_cfg);

    // 2. Pre-check energy, risk, density, connectivity, coverage envelopes.
    energy_mgr.assert_can_afford_pull(host_budget)?;
    risk_mon.assert_within_risk()?;
    density_guard.assert_rho_within_limits()?;
    conn_guard.assert_connectivity_ok()?;
    cov_mon.assert_coverage_ok()?;

    // 3. Perform backend retrieval (BCI/nanoswarm/XR stack).
    match action {
        XRAction::HostSnapshot => {
            let host_id = params.host_id.as_ref().ok_or_else(|| {
                ComplianceError::BadParams("host_id required for HostSnapshot".into())
            })?;

            let raw = cybernano_backend::pull_host_snapshot(host_id)?;
            // Enforce SNR and energy inequalities.
            snr_gate.assert_snr(raw.snr_db)?;
            energy_mgr.debit_for_pull(host_budget)?;

            Ok(XRData::HostSnapshot(XRHostSnapshot {
                host_id: host_id.clone(),
                hrv_ms: raw.hrv_ms,
                thermo_c: raw.thermo_c,
                duty_fraction: raw.duty_fraction,
                fps: raw.fps,
                fov_deg: raw.fov_deg,
                snr_db: raw.snr_db,
            }))
        }
        XRAction::CorridorEnvelope => {
            let session_id = params.session_id.as_ref().ok_or_else(|| {
                ComplianceError::BadParams("session_id required for CorridorEnvelope".into())
            })?;

            let env = cybernano_backend::pull_xr_corridor(session_id)?;
            snr_gate.assert_snr(env.snr_min_db)?;
            // corridor envelope itself is cheap; energy already checked above

            Ok(XRData::CorridorEnvelope(env))
        }
    }
}

/// New macro surface: xr_retrieve!
/// Usage: xr_retrieve!(HostSnapshot, { host_id: "...", aln_policy_shard: shard_id }, host_budget)
#[macro_export]
macro_rules! xr_retrieve {
    ($action:ident, { $($field:ident : $value:expr),* $(,)? }, $budget:expr) => {{
        use cybernano_runtime::xr_retrieval::{XRAction, XRParams, retrieve_xr_data};
        let params = XRParams {
            $($field : Some($value.into())),*,
            ..XRParams {
                host_id: None,
                session_id: None,
                aln_policy_shard: None,
            }
        };
        retrieve_xr_data(XRAction::$action, params, $budget)
    }};
}
