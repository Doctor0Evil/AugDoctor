use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Lightweight mirrors of your core traits so this crate can compile
/// without pulling in the full runtime. These must exactly match the
/// real traits at the type/field level.
pub trait NeuromorphMorphKernel {
    fn kernel_id(&self) -> String;
    fn corridor(&self) -> NeuromorphCorridorBundle;
}

pub trait BioMorphKernel {
    fn biomorph_id(&self) -> String;
    fn corridor(&self) -> NeuromorphCorridorBundle;
}

pub trait BioscaleUpgrade {
    fn upgrade_id(&self) -> String;
    fn host_budget(&self) -> HostBudget;
    fn thermo(&self) -> ThermodynamicEnvelope;
    fn ml_pass(&self) -> MlPassSchedule;
    fn reversal(&self) -> ReversalConditions;
    fn evidence(&self) -> EvidenceBundle10;
}

/// Corridor bundle derived from tests & ALN shards.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphCorridorBundle {
    pub kernel_id: String,
    pub echip_nj: f32,
    pub spike_density_hz: f32,
    pub sbio: f32,
    pub augment_load: f32,
    pub ljur: f32,
    pub evidence_hex: String,
}

/// Host budget mirrors your eco‑budgeting types.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostBudget {
    pub energy_nj_headroom: f32,
    pub protein_headroom_mg: f32,
    pub duty_fraction_max: f32,
}

/// Thermodynamic envelope and ML pass schedule mirrors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThermodynamicEnvelope {
    pub delta_t_c_max: f32,
    pub il6_index_max: f32,
    pub thermo_class: String, // e.g. "Class-C"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlPassSchedule {
    pub passes_per_day: u32,
    pub max_concurrent_jobs: u32,
}

/// Reversal conditions (NeuralRope rollback rights).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReversalConditions {
    pub rollback_seconds_max: u32,
    pub requires_human_ack: bool,
    pub neurorights_rollback_required: bool,
}

/// Ten‑tag evidence bundle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceBundle10 {
    pub tags: [EvidenceTag; 10],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceTag {
    pub name: String,   // e.g. "ATP", "CMRO2", "IL-6"
    pub value: f32,
    pub hex: String,    // short-hex evidence id
}

/// ALN‑mirrored AutonomyGrant.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyGrant {
    pub id: String,                 // autonomy.neuromorph.local.v1
    pub level: AutonomyLevel,
    pub required_blood: f32,
    pub min_chat_factor: f32,
    pub jurisdiction: String,
    pub permitted_backends: Vec<String>, // ["OrganicCPU", "Loihi", ...]
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel {
    LabOnly,
    CityMesh,
    GlobalMesh,
}

/// ALNComplianceParticle with neurorights flags.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ALNComplianceParticle {
    pub neurorights_mental_privacy: bool,
    pub neurorights_reversibility: bool,
    pub neurorights_no_finance: bool,
    pub neurorights_no_expropriation: bool,
}

/// Neurorights supertrait marker.
pub trait NeurorightsCompatibleKernel {
    fn check_neurorights(&self) -> bool;
    fn lyapunov_safe(&self) -> bool;
    fn rollback_contract(&self) -> bool;
}

/// CI‑side description of a kernel or adapter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphKernelDescriptor {
    pub id: String,
    pub kind: String, // "NeuromorphMorphKernel" | "BioMorphKernel"
    pub corridor: NeuromorphCorridorBundle,
    pub host_budget: HostBudget,
    pub thermo: ThermodynamicEnvelope,
    pub ml_pass: MlPassSchedule,
    pub reversal: ReversalConditions,
    pub evidence: EvidenceBundle10,
    pub autonomy_grant_id: String,
    pub autonomy_level: AutonomyLevel,
    pub aln_compliance: ALNComplianceParticle,
    pub neurorights_ok: bool,
}

/// Top‑level manifest emitted once per CI run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphManifest {
    pub date_utc: String,
    pub repo: String,
    pub kernels: Vec<NeuromorphKernelDescriptor>,
}

/// Invariant macros (CI‑only – panic on violation).
#[macro_export]
macro_rules! neverexceedenergyjoules {
    ($val:expr, $max:expr, $id:expr) => {
        if $val > $max {
            panic!(
                "Energy corridor violation for {}: {} > {} nJ",
                $id, $val, $max
            );
        }
    };
}

#[macro_export]
macro_rules! duty_ceiling {
    ($duty:expr, $ceiling:expr, $id:expr) => {
        if $duty > $ceiling {
            panic!(
                "Duty ceiling violation for {}: duty {} > {}",
                $id, $duty, $ceiling
            );
        }
    };
}

/// Entry point for CI tooling (invoked via `cargo test` or a small bin).
pub fn run_neuromorph_ci(repo_root: &Path) {
    let mut kernels: Vec<NeuromorphKernelDescriptor> = Vec::new();
    let mut autonomy_grants: BTreeMap<String, AutonomyGrant> = BTreeMap::new();

    // 1. Discover ALN AutonomyGrant shards in `aln/`.
    let aln_dir = repo_root.join("aln");
    if aln_dir.is_dir() {
        for entry in walk_dir(&aln_dir) {
            if entry.extension().and_then(|e| e.to_str()) == Some("aln") {
                if let Ok(grant) = parse_autonomy_grant(&entry) {
                    autonomy_grants.insert(grant.id.clone(), grant);
                }
            }
        }
    }

    // 2. Discover neuromorph kernels and bioscale upgrades via JSON descriptors
    //    generated from tests (e.g., `target/neuromorph-descriptors/*.json`).
    let desc_dir = repo_root.join("target").join("neuromorph-descriptors");
    if desc_dir.is_dir() {
        for entry in walk_dir(&desc_dir) {
            if entry.extension().and_then(|e| e.to_str()) == Some("json") {
                let raw = fs::read_to_string(&entry)
                    .expect("Failed to read neuromorph descriptor JSON");
                let desc: NeuromorphKernelDescriptor =
                    serde_json::from_str(&raw).expect("Bad neuromorph descriptor JSON");
                validate_descriptor(&desc, &autonomy_grants);
                kernels.push(desc);
            }
        }
    }

    let manifest = NeuromorphManifest {
        date_utc: Utc::now().to_rfc3339(),
        repo: repo_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        kernels,
    };

    let date = Utc::now().format("%Y-%m-%d").to_string();
    let out = repo_root
        .join("research-manifests")
        .join(format!("research-{}-neuromorph-manifest.json", date));
    fs::create_dir_all(out.parent().unwrap())
        .expect("Failed to create research-manifests directory");
    let json = serde_json::to_string_pretty(&manifest)
        .expect("Failed to serialize neuromorph manifest");
    fs::write(&out, json).expect("Failed to write neuromorph manifest");
}

/// Validate invariants for a single kernel descriptor.
fn validate_descriptor(desc: &NeuromorphKernelDescriptor,
                       autonomy_grants: &BTreeMap<String, AutonomyGrant>) {
    // 10‑tag evidence requirement.
    if desc.evidence.tags.len() != 10 {
        panic!(
            "Kernel {} missing evidence tags: expected 10, found {}",
            desc.id,
            desc.evidence.tags.len()
        );
    }

    // Energy & duty invariants using macros.
    neverexceedenergyjoules!(
        desc.corridor.echip_nj,
        1_000.0,
        desc.id
    );
    duty_ceiling!(
        desc.host_budget.duty_fraction_max,
        0.25,
        desc.id
    );

    // Thermodynamic envelope ceilings (Class‑C).
    if desc.thermo.thermo_class == "Class-C" {
        if desc.thermo.delta_t_c_max > 1.5 {
            panic!(
                "Kernel {} exceeds Class-C ΔT envelope: {}°C",
                desc.id, desc.thermo.delta_t_c_max
            );
        }
        if desc.thermo.il6_index_max > 0.3 {
            panic!(
                "Kernel {} exceeds Class-C IL-6 envelope: {}",
                desc.id, desc.thermo.il6_index_max
            );
        }
    }

    // Autonomy grant linkage.
    let grant = autonomy_grants
        .get(&desc.autonomy_grant_id)
        .unwrap_or_else(|| panic!(
            "Kernel {} missing AutonomyGrant {}",
            desc.id, desc.autonomy_grant_id
        ));

    // Ensure declared autonomy level is not above grant level.
    if desc.autonomy_level > grant.level {
        panic!(
            "Kernel {} autonomy {:?} exceeds grant {:?}",
            desc.id, desc.autonomy_level, grant.level
        );
    }

    // Neurorights compliance.
    if !desc.aln_compliance.neurorights_mental_privacy
        || !desc.aln_compliance.neurorights_reversibility
        || !desc.aln_compliance.neurorights_no_finance
        || !desc.aln_compliance.neurorights_no_expropriation
        || !desc.neurorights_ok
    {
        panic!(
            "Kernel {} is not NeurorightsCompatibleKernel / ALN compliant",
            desc.id
        );
    }
}

/// Simple recursive walker.
fn walk_dir(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(read) = fs::read_dir(root) {
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk_dir(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

/// Minimal ALN parser stub: in your repo, replace this with your real
/// ALN loader that maps `*.aln` → `AutonomyGrant`.
fn parse_autonomy_grant(path: &Path) -> Result<AutonomyGrant, ()> {
    let content = fs::read_to_string(path).map_err(|_| ())?;
    if !content.contains("schema autonomy.neuromorph") {
        return Err(());
    }
    // Simple, deterministic extraction with regex or line scanning.
    // Here we construct a placeholder that is still structurally valid.
    Ok(AutonomyGrant {
        id: "autonomy.neuromorph.local.v1".to_string(),
        level: AutonomyLevel::LabOnly,
        required_blood: 0.1,
        min_chat_factor: 0.4,
        jurisdiction: "local".to_string(),
        permitted_backends: vec![
            "OrganicCPU".to_string(),
            "Loihi".to_string(),
            "GPU".to_string(),
        ],
    })
}
