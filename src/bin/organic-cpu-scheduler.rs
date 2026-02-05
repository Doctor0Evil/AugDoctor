//! Organic CPU scheduler guard for POWER tokens and eco / neurorights gates.
//! This is a software-only enforcement layer suitable for OrganicCPU / Raspberry Pi Pico class devices.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use clap::{Arg, ArgAction, Command};
use serde::Deserialize;
use serde_json::{self as json, Value};

#[derive(Debug, Deserialize)]
struct PerTurnValidationProfile {
    max_turn_duration_ms: u64,
    agentic_caps: AgenticCaps,
    eco_gates: EcoGates,
    neurorights_guards: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AgenticCaps {
    max_actions: u64,
    max_cycles: u64,
}

#[derive(Debug, Deserialize)]
struct EcoGates {
    max_kwh: f64,
    require_carbon_aware: bool,
}

#[derive(Debug)]
struct RuntimeTurnMetrics {
    actions_used: u64,
    cycles_used: u64,
    kwh_used: f64,
    carbon_aware: bool,
    // Optional: in a real system, bring in grid intensity / ATP etc.
}

fn load_profile(path: &Path) -> (PerTurnValidationProfile, Value) {
    let mut file = File::open(path).unwrap_or_else(|e| panic!("Profile not found {}: {}", path.display(), e));
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

    let json_val: Value = json::from_str(&buf).unwrap_or_else(|e| panic!("Invalid JSON in {}: {}", path.display(), e));
    let profile: PerTurnValidationProfile =
        json::from_value(json_val.clone()).unwrap_or_else(|e| panic!("Profile schema mismatch: {}", e));
    (profile, json_val)
}

fn simulate_runtime_metrics() -> RuntimeTurnMetrics {
    // In production, this would read from OrganicCPU telemetry (cycles, power meter).
    RuntimeTurnMetrics {
        actions_used: 2,
        cycles_used: 42_000,
        kwh_used: 0.006,
        carbon_aware: true,
    }
}

fn enforce_power_caps(profile: &PerTurnValidationProfile, metrics: &RuntimeTurnMetrics) -> Vec<String> {
    let mut errors = Vec::new();

    if metrics.actions_used > profile.agentic_caps.max_actions {
        errors.push(format!(
            "actions_used={} exceeds POWER.max_actions={}",
            metrics.actions_used, profile.agentic_caps.max_actions
        ));
    }

    if metrics.cycles_used > profile.agentic_caps.max_cycles {
        errors.push(format!(
            "cycles_used={} exceeds POWER.max_cycles={}",
            metrics.cycles_used, profile.agentic_caps.max_cycles
        ));
    }

    errors
}

fn enforce_eco(profile: &PerTurnValidationProfile, metrics: &RuntimeTurnMetrics, check_eco: bool) -> Vec<String> {
    let mut errors = Vec::new();

    if !check_eco {
        return errors;
    }

    if metrics.kwh_used > profile.eco_gates.max_kwh {
        errors.push(format!(
            "kwh_used={} exceeds eco_gates.max_kwh={}",
            metrics.kwh_used, profile.eco_gates.max_kwh
        ));
    }

    if profile.eco_gates.require_carbon_aware && !metrics.carbon_aware {
        errors.push("carbon-aware flag is required but metrics.carbon_aware=false".to_string());
    }

    errors
}

fn enforce_neurorights(profile: &PerTurnValidationProfile) -> Vec<String> {
    // Guard is deliberately string-based; the heavy LTL lives in ALN shards.
    let mut errors = Vec::new();
    for guard in &profile.neurorights_guards {
        // We only enforce simple forbidden patterns here; ALN does the full temporal logic.
        if guard.contains("max_state_divergence > 0.15") {
            errors.push(format!(
                "Neurorights guard '{}' encodes a forbidden divergence (> 0.15)",
                guard
            ));
        }
        if guard.contains("allow_self_augmentation = false") {
            errors.push("Neurorights guard forbids self augmentation (COGNITIVE_LIBERTY violated)".to_string());
        }
    }
    errors
}

fn main() {
    let matches = Command::new("organic-cpu-scheduler")
        .about("OrganicCPU POWER + eco + neurorights enforcement guard")
        .arg(
            Arg::new("profile")
                .long("profile")
                .required(true)
                .value_name("PATH")
                .help("Path to per-turn validation profile JSON"),
        )
        .arg(
            Arg::new("eco-gate")
                .long("eco-gate")
                .action(ArgAction::SetTrue)
                .help("Enable eco impact eligibility enforcement"),
        )
        .get_matches();

    let profile_path = Path::new(matches.get_one::<String>("profile").unwrap());
    let (profile, _) = load_profile(profile_path);
    let metrics = simulate_runtime_metrics();

    let mut errors = Vec::new();
    errors.extend(enforce_power_caps(&profile, &metrics));
    errors.extend(enforce_eco(&profile, &metrics, matches.get_flag("eco-gate")));
    errors.extend(enforce_neurorights(&profile));

    if !errors.is_empty() {
        eprintln!("organic-cpu-scheduler: validation FAILED");
        for e in &errors {
            eprintln!("- {}", e);
        }
        std::process::exit(1);
    }

    println!(
        "{{\"tool\":\"organic-cpu-scheduler\",\"status\":\"ok\",\"profile\":\"{}\",\"actions_used\":{},\"cycles_used\":{},\"kwh_used\":{}}}",
        profile_path.display(),
        metrics.actions_used,
        metrics.cycles_used,
        metrics.kwh_used
    );
}
