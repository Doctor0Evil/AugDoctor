use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

fn main() {
    let repo_root = std::env::var("GITHUB_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("cwd"));

    let graph = build_autonomy_graph(&repo_root);
    enforce_autonomy_invariants(&graph);
}

/// Graph nodes and edges.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub autonomy_level: Option<AutonomyLevel>,
    pub chat_factor: Option<f32>,
    pub blood_factor: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeKind {
    Crate,
    AutonomyGrant,
    QpuDatashard,
    AlnParticle,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AutonomyLevel {
    LabOnly,
    CityMesh,
    GlobalMesh,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub relation: String, // "uses", "gates", "guards", "stim-path", "organic-cpu"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyGraph {
    pub nodes: BTreeMap<String, Node>,
    pub edges: Vec<Edge>,
    pub cyber_rank: BTreeMap<String, f32>,
}

/// Build a minimal autonomy graph from manifests and ALN.
fn build_autonomy_graph(repo_root: &Path) -> AutonomyGraph {
    let mut nodes = BTreeMap::new();
    let mut edges = Vec::new();

    // 1. Crate nodes from Cargo.toml (workspace members).
    let cargo = repo_root.join("Cargo.toml");
    if let Ok(raw) = fs::read_to_string(&cargo) {
        for line in raw.lines() {
            if line.trim_start().starts_with("members") {
                // Very simple parse; real implementation should parse TOML.
                // Here we just add a generic repo node.
                let root_id = repo_root
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("root")
                    .to_string();
                nodes.insert(
                    root_id.clone(),
                    Node {
                        id: root_id.clone(),
                        kind: NodeKind::Crate,
                        autonomy_level: Some(AutonomyLevel::LabOnly),
                        chat_factor: Some(0.5),
                        blood_factor: Some(0.1),
                    },
                );
                break;
            }
        }
    }

    // 2. AutonomyGrant nodes from ALN shards.
    let aln_dir = repo_root.join("aln");
    if aln_dir.is_dir() {
        for entry in walk_dir(&aln_dir) {
            if entry.extension().and_then(|e| e.to_str()) == Some("aln") {
                if let Ok(grant) = parse_autonomy_grant(&entry) {
                    nodes.insert(
                        grant.id.clone(),
                        Node {
                            id: grant.id.clone(),
                            kind: NodeKind::AutonomyGrant,
                            autonomy_level: Some(grant.level),
                            chat_factor: Some(grant.min_chat_factor),
                            blood_factor: Some(grant.required_blood),
                        },
                    );
                }
            }
        }
    }

    // 3. Manifest‑derived nodes/edges (neuromorph, upgrades, chat, autonomy).
    let manifests_dir = repo_root.join("research-manifests");
    if manifests_dir.is_dir() {
        for entry in walk_dir(&manifests_dir) {
            if entry
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.contains("neuromorph-manifest"))
                == Some(true)
            {
                let raw = fs::read_to_string(&entry)
                    .expect("Failed to read neuromorph manifest");
                let neuromorph_manifest: NeuromorphManifest =
                    serde_json::from_str(&raw).expect("Bad neuromorph manifest JSON");

                for k in neuromorph_manifest.kernels {
                    let kernel_node_id = k.id.clone();
                    nodes.insert(
                        kernel_node_id.clone(),
                        Node {
                            id: kernel_node_id.clone(),
                            kind: NodeKind::Crate,
                            autonomy_level: Some(k.autonomy_level),
                            chat_factor: Some(0.6),
                            blood_factor: Some(0.2),
                        },
                    );
                    edges.push(Edge {
                        from: kernel_node_id.clone(),
                        to: k.autonomy_grant_id.clone(),
                        relation: "gated-by-grant".to_string(),
                    });

                    // Heuristic edges for stim / organic_cpu based on IDs.
                    if kernel_node_id.contains("stim") {
                        edges.push(Edge {
                            from: "github-actions".to_string(),
                            to: kernel_node_id.clone(),
                            relation: "stim-path".to_string(),
                        });
                    }
                    if kernel_node_id.contains("organic-cpu") {
                        edges.push(Edge {
                            from: "github-actions".to_string(),
                            to: kernel_node_id.clone(),
                            relation: "organic-cpu".to_string(),
                        });
                    }
                }
            }
        }
    }

    let cyber_rank = compute_cyber_rank(&nodes);

    AutonomyGraph {
        nodes,
        edges,
        cyber_rank,
    }
}

/// Minimal manifest mirror (same as cyberswarm-neuromorph-ci).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphManifest {
    pub date_utc: String,
    pub repo: String,
    pub kernels: Vec<NeuromorphKernelDescriptor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuromorphKernelDescriptor {
    pub id: String,
    pub kind: String,
    pub autonomy_grant_id: String,
    pub autonomy_level: AutonomyLevel,
}

/// Simple CyberRank scoring from CHAT, Blood, and level.
fn compute_cyber_rank(nodes: &BTreeMap<String, Node>) -> BTreeMap<String, f32> {
    let mut rank = BTreeMap::new();
    for (id, node) in nodes {
        let base = match node.autonomy_level {
            Some(AutonomyLevel::LabOnly) => 0.1,
            Some(AutonomyLevel::CityMesh) => 0.5,
            Some(AutonomyLevel::GlobalMesh) => 1.0,
            None => 0.0,
        };
        let chat = node.chat_factor.unwrap_or(0.0);
        let blood = node.blood_factor.unwrap_or(0.0);
        let score = base + 0.4 * chat + 0.5 * blood;
        rank.insert(id.clone(), score);
    }
    rank
}

/// Enforce CI invariants on autonomy graph.
fn enforce_autonomy_invariants(graph: &AutonomyGraph) {
    // 1. No neuromorph / BCI crate may exceed host AutonomyGrant.
    for edge in &graph.edges {
        if edge.relation == "gated-by-grant" {
            let kernel = graph.nodes.get(&edge.from).unwrap();
            let grant = graph.nodes.get(&edge.to).unwrap();
            if let (Some(k_level), Some(g_level)) =
                (kernel.autonomy_level, grant.autonomy_level)
            {
                if k_level > g_level {
                    panic!(
                        "Autonomy violation: {} {:?} > grant {} {:?}",
                        kernel.id, k_level, grant.id, g_level
                    );
                }
            }
        }
    }

    // 2. No path from GitHub Actions or dev tunnel to stim / organic_cpu
    //    without NeurorightsGuard & HostBudget / EvidenceBundle edges.
    let mut stim_targets: BTreeSet<String> = BTreeSet::new();
    for edge in &graph.edges {
        if edge.relation == "stim-path" || edge.relation == "organic-cpu" {
            stim_targets.insert(edge.to.clone());
        }
    }

    for target in stim_targets {
        // For now we enforce that any stim/organic‑cpu node must have
        // at least one "guards" edge from NeurorightsGuard and HostBudget.
        let mut has_neurorights_guard = false;
        let mut has_host_budget_guard = false;
        let mut has_evidence_guard = false;

        for e in &graph.edges {
            if e.to == target && e.relation == "guarded-by-neurorights" {
                has_neurorights_guard = true;
            }
            if e.to == target && e.relation == "guarded-by-hostbudget" {
                has_host_budget_guard = true;
            }
            if e.to == target && e.relation == "guarded-by-evidencebundle" {
                has_evidence_guard = true;
            }
        }

        if !(has_neurorights_guard && has_host_budget_guard && has_evidence_guard) {
            panic!(
                "CI autonomy guard violation: node {} reachable from github-actions \
                 without NeurorightsGuard + HostBudget + EvidenceBundle guards",
                target
            );
        }
    }
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyGrant {
    pub id: String,
    pub level: AutonomyLevel,
    pub required_blood: f32,
    pub min_chat_factor: f32,
    pub jurisdiction: String,
    pub permitted_backends: Vec<String>,
}

fn parse_autonomy_grant(path: &Path) -> Result<AutonomyGrant, ()> {
    let content = fs::read_to_string(path).map_err(|_| ())?;
    if !content.contains("schema autonomy.neuromorph") {
        return Err(());
    }
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
