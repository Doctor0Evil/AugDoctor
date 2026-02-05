//! CI guard for neuromorphic envelopes.
//! - Validates research-*-autonomy-manifest.json against JSON Schemas
//! - Enforces MORPH <= EVOLVE corridor math
//! - Emits a machine-readable validation report on stdout

use std::fs::File;
use std::io::Read;
use std::path::Path;

use clap::{Arg, ArgAction, Command};
use schemars::schema::RootSchema;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{self as json, Value};

/// Minimal view of the MORPH token (as per policies/morph_token.schema.json)
#[derive(Debug, Deserialize, JsonSchema)]
struct MorphToken {
    id: String,
    subject_id: String,
    dimensions: MorphDimensions,
    valid_from: String,
    valid_until: String,
    revocable: bool,
    eco_impact_guard: EcoGuard,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MorphDimensions {
    eco: MorphEco,
    cyber: MorphCyber,
    neuro: MorphNeuro,
    smart: MorphSmart,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MorphEco {
    max_kwh_inference: f32,
    #[allow(dead_code)]
    grid_intensity_threshold_gco2_kwh: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MorphCyber {
    max_latency_ms: u32,
    #[allow(dead_code)]
    max_thermal_dissipation_w: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MorphNeuro {
    max_atp_per_synapse: u64,
    spike_density_corridor: Option<[f32; 2]>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MorphSmart {
    max_inference_latency_ms: u32,
    #[allow(dead_code)]
    core_temp_max_c: Option<f32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EcoGuard {
    #[allow(dead_code)]
    embodied_energy_max_mj: Option<f32>,
    carbon_aware_scheduling: bool,
}

/// Minimal EVOLVE bounds profile (host-local, non-financial).
/// In a real deployment this would come from an ALN shard or
/// biophysical-blockchain eco/evo profile.
#[derive(Debug, Deserialize, JsonSchema)]
struct EvolveBounds {
    /// max_kwh_inference_corridor.upper
    max_kwh_inference: f32,
    /// global ATP ceiling per synapse per day
    max_atp_per_synapse: u64,
    /// spike_density corridor, e.g. [0.7, 1.3]
    spike_density_corridor: [f32; 2],
}

/// Per-turn manifest view (research-DATE-autonomy-manifest.json)
#[derive(Debug, Deserialize, JsonSchema)]
struct AutonomyManifest {
    date: String,
    subject_id: String,
    autonomy_graph: Vec<String>,
    evidence_bundles: Vec<Value>,
    per_turn_profile: PerTurnProfile,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PerTurnProfile {
    power_token_ref: String,
    morph_token_ref: String,
    corridor_math_invariants: Vec<String>,
}

#[derive(Debug)]
struct CorridorInvariant {
    dimension: String,
    lower: f32,
    upper: f32,
}

fn parse_corridor(expr: &str) -> Option<CorridorInvariant> {
    // Example: "CORRIDOR ECO 0.05 0.12"
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 4 {
        return None;
    }
    if parts[0] != "CORRIDOR" {
        return None;
    }
    let dim = parts[1].to_string();
    let lower = parts[2].parse::<f32>().ok()?;
    let upper = parts[3].parse::<f32>().ok()?;
    Some(CorridorInvariant {
        dimension: dim,
        lower,
        upper,
    })
}

fn load_json(path: &Path) -> Value {
    let mut file = File::open(path)
        .unwrap_or_else(|e| panic!("Failed to open {}: {}", path.display(), e));
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));
    json::from_str(&buf).unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", path.display(), e))
}

fn load_typed<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let v = load_json(path);
    json::from_value(v).unwrap_or_else(|e| panic!("Schema mismatch in {}: {}", path.display(), e))
}

/// Simple JSON Schema validation using schemars at build-time
fn validate_against_schema<T: JsonSchema + for<'de> Deserialize<'de>>(
    instance: &Value,
) -> Result<(), String> {
    let schema: RootSchema = schemars::schema_for!(T);
    let compiled = jsonschema::JSONSchema::compile(&json::to_value(schema).unwrap())
        .map_err(|e| format!("Failed to compile schema: {}", e))?;
    let result = compiled.validate(instance);
    if let Err(errors) = result {
        let mut msgs = Vec::new();
        for err in errors {
            msgs.push(format!("path {}: {}", err.instance_path, err));
        }
        return Err(msgs.join("; "));
    }
    Ok(())
}

fn main() {
    let matches = Command::new("cyberswarm-neuromorph-ci")
        .about("MORPH / EVOLVE CI validation guard")
        .arg(
            Arg::new("manifest")
                .long("manifest")
                .required(true)
                .value_name("PATH")
                .help("Path to research-DATE-autonomy-manifest.json"),
        )
        .arg(
            Arg::new("morph-token")
                .long("morph-token")
                .required(true)
                .value_name("PATH")
                .help("Path to MORPH token JSON referenced by manifest"),
        )
        .arg(
            Arg::new("evolve-bounds")
                .long("evolve-bounds")
                .required(true)
                .value_name("PATH")
                .help("Host-local EVOLVE bounds JSON"),
        )
        .arg(
            Arg::new("check-morph-evolve")
                .long("check-morph-evolve")
                .action(ArgAction::SetTrue)
                .help("Enforce MORPH <= EVOLVE invariants"),
        )
        .arg(
            Arg::new("check-corridors")
                .long("check-corridors")
                .action(ArgAction::SetTrue)
                .help("Validate per_turn_profile.corridor_math_invariants"),
        )
        .get_matches();

    let manifest_path = Path::new(matches.get_one::<String>("manifest").unwrap());
    let morph_path = Path::new(matches.get_one::<String>("morph-token").unwrap());
    let evolve_path = Path::new(matches.get_one::<String>("evolve-bounds").unwrap());

    let manifest_raw = load_json(manifest_path);
    let morph_raw = load_json(morph_path);
    let evolve_raw = load_json(evolve_path);

    // Schema validation (local-only, no network fetch).
    validate_against_schema::<AutonomyManifest>(&manifest_raw)
        .expect("Autonomy manifest failed schema validation");
    validate_against_schema::<MorphToken>(&morph_raw)
        .expect("MORPH token failed schema validation");
    validate_against_schema::<EvolveBounds>(&evolve_raw)
        .expect("EVOLVE bounds failed schema validation");

    let manifest: AutonomyManifest =
        json::from_value(manifest_raw).expect("Manifest parse failed (typed)");
    let morph: MorphToken = json::from_value(morph_raw).expect("MORPH parse failed");
    let evolve: EvolveBounds = json::from_value(evolve_raw).expect("EVOLVE parse failed");

    let mut errors: Vec<String> = Vec::new();

    // Sanity: manifest references the passed MORPH token
    if manifest.per_turn_profile.morph_token_ref != morph.id {
        errors.push(format!(
            "per_turn_profile.morph_token_ref = {} does not match MORPH.id = {}",
            manifest.per_turn_profile.morph_token_ref, morph.id
        ));
    }

    // Enforce MORPH <= EVOLVE
    if matches.get_flag("check-morph-evolve") {
        // eco corridor: MORPH.eco.max_kwh_inference <= EVOLVE.max_kwh_inference
        if morph.dimensions.eco.max_kwh_inference > evolve.max_kwh_inference {
            errors.push(format!(
                "MORPH.eco.max_kwh_inference={} exceeds EVOLVE.max_kwh_inference={}",
                morph.dimensions.eco.max_kwh_inference, evolve.max_kwh_inference
            ));
        }

        // neuro ATP corridor
        if morph.dimensions.neuro.max_atp_per_synapse > evolve.max_atp_per_synapse {
            errors.push(format!(
                "MORPH.neuro.max_atp_per_synapse={} exceeds EVOLVE.max_atp_per_synapse={}",
                morph.dimensions.neuro.max_atp_per_synapse, evolve.max_atp_per_synapse
            ));
        }

        // spike density corridor compatibility (if present in MORPH)
        if let Some(corr) = morph.dimensions.neuro.spike_density_corridor {
            let [l, u] = corr;
            let [gl, gu] = evolve.spike_density_corridor;
            if l < gl || u > gu {
                errors.push(format!(
                    "MORPH.neuro.spike_density_corridor=[{}, {}] is outside EVOLVE corridor [{}, {}]",
                    l, u, gl, gu
                ));
            }
        }

        if !morph.eco_impact_guard.carbon_aware_scheduling {
            errors.push("MORPH.eco_impact_guard.carbon_aware_scheduling must be true".to_string());
        }
    }

    // Corridor math invariants in manifest (CORRIDOR DIM lower upper)
    if matches.get_flag("check-corridors") {
        for expr in &manifest.per_turn_profile.corridor_math_invariants {
            if let Some(inv) = parse_corridor(expr) {
                if inv.lower < 0.0 || inv.upper <= 0.0 {
                    errors.push(format!(
                        "Corridor '{}' has non-positive bounds ({}, {})",
                        expr, inv.lower, inv.upper
                    ));
                }
                if inv.lower > inv.upper {
                    errors.push(format!(
                        "Corridor '{}' is invalid: lower > upper ({} > {})",
                        expr, inv.lower, inv.upper
                    ));
                }
            } else {
                errors.push(format!(
                    "Could not parse corridor_math_invariant expression '{}'",
                    expr
                ));
            }
        }
    }

    if !errors.is_empty() {
        eprintln!("cyberswarm-neuromorph-ci: validation FAILED");
        for e in &errors {
            eprintln!("- {}", e);
        }
        std::process::exit(1);
    }

    println!(
        "{{\"tool\":\"cyberswarm-neuromorph-ci\",\"status\":\"ok\",\"manifest\":\"{}\",\"morph\":\"{}\"}}",
        manifest_path.display(),
        morph.id
    );
}
