use serde::{Deserialize, Serialize};

/// Snapshot of an organic host, parallel to BciHostSnapshot / ChatHostSnapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganicCpuSnapshot {
    pub hrv_ms: f32,
    pub eeg_load_norm: f32,
    pub core_temp_c: f32,
    pub local_temp_c: f32,
    pub duty_fraction: f32,
    pub il6_proxy: f32,
}

/// HostBudget + thermo + ML schedule for scheduling decisions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostBudget {
    pub energy_nj_headroom: f32,
    pub protein_headroom_mg: f32,
    pub duty_fraction_max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThermodynamicEnvelope {
    pub delta_t_c_max: f32,
    pub il6_index_max: f32,
    pub thermo_class: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlPassSchedule {
    pub passes_per_day: u32,
    pub max_concurrent_jobs: u32,
}

/// ALNComplianceParticle with neurorights flags.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ALNComplianceParticle {
    pub neurorights_mental_privacy: bool,
    pub neurorights_reversibility: bool,
    pub neurorights_no_finance: bool,
    pub neurorights_no_expropriation: bool,
}

/// AutonomyGrant mirror.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyGrant {
    pub id: String,
    pub level: AutonomyLevel,
    pub required_blood: f32,
    pub min_chat_factor: f32,
    pub jurisdiction: String,
    pub permitted_backends: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel {
    LabOnly,
    CityMesh,
    GlobalMesh,
}

/// Generic neuromorph job descriptor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphJob {
    pub job_id: String,
    pub kernel_id: String,
    pub energy_cost_nj: f32,
    pub protein_cost_mg: f32,
    pub sbio_cost: f32,
    pub duty_cost: f32,
    pub delta_t_c: f32,
    pub autonomy_grant_id: String,
    pub aln_compliance: ALNComplianceParticle,
}

/// Simple decision surface.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Approved,
    Deferred,
    Denied,
}

/// BioVirtualScheduler / OrganicCpuScheduler surface.
pub trait BioVirtualScheduler {
    fn schedule_neuromorph_job(
        &self,
        host: &OrganicCpuSnapshot,
        budget: &HostBudget,
        thermo: &ThermodynamicEnvelope,
        ml: &MlPassSchedule,
        job: &NeuromorphJob,
        autonomy_grant: &AutonomyGrant,
    ) -> Decision;
}

/// Lyapunov‑based scheduler with corridor polytopes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganicCpuScheduler {
    pub u_safe_duty: f32,
    pub max_energy_nj: f32,
    pub max_protein_mg: f32,
    pub max_sbio: f32,
}

impl OrganicCpuScheduler {
    /// V(u) = (u - u_safe)^2
    fn lyapunov(&self, duty: f32) -> f32 {
        let diff = duty - self.u_safe_duty;
        diff * diff
    }

    /// Check corridor polytope over (E, Mprot, Sbio, duty, ΔT).
    fn corridor_ok(
        &self,
        host: &OrganicCpuSnapshot,
        budget: &HostBudget,
        thermo: &ThermodynamicEnvelope,
        job: &NeuromorphJob,
    ) -> bool {
        // Energy: require headroom after job.
        let e_after = budget.energy_nj_headroom - job.energy_cost_nj;
        if e_after < 0.0 || job.energy_cost_nj > self.max_energy_nj {
            return false;
        }

        // Protein headroom.
        let mprot_after = budget.protein_headroom_mg - job.protein_cost_mg;
        if mprot_after < 0.0 || job.protein_cost_mg > self.max_protein_mg {
            return false;
        }

        // Bioscale cost.
        if job.sbio_cost > self.max_sbio {
            return false;
        }

        // Duty: current + cost must be within host & scheduler envelope.
        let duty_after = host.duty_fraction + job.duty_cost;
        if duty_after > budget.duty_fraction_max {
            return false;
        }

        // Thermal polytope: requested ΔT within envelope.
        if job.delta_t_c > thermo.delta_t_c_max {
            return false;
        }

        true
    }

    /// Neurorights & ALNCompliance check.
    fn neurorights_ok(&self, job: &NeuromorphJob) -> bool {
        let c = &job.aln_compliance;
        c.neurorights_mental_privacy
            && c.neurorights_reversibility
            && c.neurorights_no_finance
            && c.neurorights_no_expropriation
    }

    /// AutonomyGrant vs job autonomy sanity.
    fn autonomy_ok(&self, grant: &AutonomyGrant, job: &NeuromorphJob) -> bool {
        // Require that backend is permitted for OrganicCPU.
        if !grant
            .permitted_backends
            .iter()
            .any(|b| b == "OrganicCPU")
        {
            return false;
        }
        // Energy/protein headroom thresholds enforced by caller via HostBudget.
        if job.energy_cost_nj > grant.required_blood * 1_000.0 {
            // Bridge blood requirement to energy envelope in a deterministic way.
            return false;
        }
        true
    }
}

impl BioVirtualScheduler for OrganicCpuScheduler {
    fn schedule_neuromorph_job(
        &self,
        host: &OrganicCpuSnapshot,
        budget: &HostBudget,
        thermo: &ThermodynamicEnvelope,
        ml: &MlPassSchedule,
        job: &NeuromorphJob,
        autonomy_grant: &AutonomyGrant,
    ) -> Decision {
        // 1. Lyapunov duty control: require V(u_after) <= V(u_current).
        let duty_after = host.duty_fraction + job.duty_cost;
        let v_before = self.lyapunov(host.duty_fraction);
        let v_after = self.lyapunov(duty_after);
        let lyapunov_ok = v_after <= v_before;

        if !lyapunov_ok {
            return Decision::Deferred;
        }

        // 2. Corridor polytope.
        if !self.corridor_ok(host, budget, thermo, job) {
            return Decision::Denied;
        }

        // 3. ML schedule ceilings (Class‑C).
        if ml.max_concurrent_jobs > 8 || ml.passes_per_day > 128 {
            return Decision::Denied;
        }

        // 4. Neurorights and ALN compliance.
        if !self.neurorights_ok(job) {
            return Decision::Denied;
        }

        // 5. Autonomy grant scope.
        if !self.autonomy_ok(autonomy_grant, job) {
            return Decision::Denied;
        }

        // 6. Organic safety band heuristics: IL‑6 & EEG load.
        if host.il6_proxy > thermo.il6_index_max {
            return Decision::Deferred;
        }
        if host.eeg_load_norm > 0.85 {
            return Decision::Deferred;
        }

        Decision::Approved
    }
}
