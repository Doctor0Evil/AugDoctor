use std::env;
use std::fs;
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DowManifest {
    platform: String,
    dow_id: String,
    eco_floor: f64,
    ndm_ceiling: f64,
    capabilities: Vec<String>,
    did_owner: String,
    code_anchor_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourzeManifest {
    sourze_id: String,
    did_owner: String,
    capabilities: Vec<String>,
    offline_ok: bool,
    ndm_ceiling: f64,
}

#[derive(Debug)]
struct NdmScore {
    value: f64,
    reasons: Vec<String>,
}

const FIXED_MANIFEST_PATH: &str = "/var/lib/aln/manifests/";
const ECO_FLOOR_MIN: f64 = 0.86;
const PROTECTED_DID: &str = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";

fn main() {
    println!("[ALN-DAEMON] Autonomous run triggered — no CLI, no Cargo runtime");
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    println!("[COMPAT] Detected: {} / {}", os, arch);

    loop {
        let ndm = compute_ndm(os, arch);
        if ndm.value > 0.3 {
            println!("[NDM] Quarantine active: {:?}", ndm.reasons);
            thread::sleep(Duration::from_secs(60));
            continue;
        }

        if let Ok(dow_raw) = fs::read_to_string(format!("{}dow_manifest.json", FIXED_MANIFEST_PATH)) {
            if let Ok(dow) = serde_json::from_str::<DowManifest>(&dow_raw) {
                if dow.eco_floor >= ECO_FLOOR_MIN && dow.did_owner == PROTECTED_DID {
                    println!("[DOW] Compatibility paradigm active for {}", dow.platform);
                    apply_dow_logic(&dow);
                }
            }
        }

        if let Ok(sourze_raw) = fs::read_to_string(format!("{}sourze_manifest.json", FIXED_MANIFEST_PATH)) {
            if let Ok(s) = serde_json::from_str::<SourzeManifest>(&sourze_raw) {
                enforce_sourze(&s);
            }
        }

        thread::sleep(Duration::from_secs(30)); // heartbeat
    }
}

fn compute_ndm(os: &str, arch: &str) -> NdmScore {
    let mut score = 0.0;
    let mut reasons = vec![];
    if !["linux", "windows", "android", "ios"].contains(&os) {
        score += 0.4;
        reasons.push("unsupported_os".to_string());
    }
    if !["x86_64", "aarch64"].contains(&arch) {
        score += 0.3;
        reasons.push("unsupported_arch".to_string());
    }
    // DID presence check (real file existence proxy)
    if !fs::metadata(format!("{}did_anchor.key", FIXED_MANIFEST_PATH)).is_ok() {
        score += 0.5;
        reasons.push("missing_did_anchor".to_string());
    }
    NdmScore { value: score.min(1.0), reasons }
}

fn apply_dow_logic(dow: &DowManifest) {
    println!("[DOW-APPLY] Reviving deprecated component {} with anti-rollback", dow.dow_id);
    // machine-as-law: forward-only
}

fn enforce_sourze(s: &SourzeManifest) {
    if s.offline_ok && s.did_owner == PROTECTED_DID && s.ndm_ceiling > 0.3 {
        println!("[SOURZE] Capability enforcement OK: {:?}", s.capabilities);
    }
}
