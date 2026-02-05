//! CI helper for ALN autonomy graph shards on GitHub.
//! - Scans a directory or glob for *.alnshard
//! - Verifies Ed25519 signatures (public key must be provided)
//! - Computes a simple eco-impact summary for reporting

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::{Arg, Command};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct ShardRecord {
    path: PathBuf,
    valid_signature: bool,
    eco_score: Option<f32>,
}

fn load_key(path: &Path) -> VerifyingKey {
    let mut f = File::open(path).unwrap_or_else(|e| panic!("Failed to open key {}: {}", path.display(), e));
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap_or_else(|e| panic!("Failed to read key {}: {}", path.display(), e));
    VerifyingKey::from_bytes(
        buf.as_slice()
            .try_into()
            .unwrap_or_else(|_| panic!("Expected 32-byte Ed25519 public key in {}", path.display())),
    )
    .expect("Invalid Ed25519 public key")
}

/// Very small convention: last 64 bytes are Ed25519 signature, everything
/// before is signed payload. ALN header encodes eco_impact_score as a fixed
/// ASCII field `eco:` with a float.
fn verify_shard(path: &Path, key: &VerifyingKey) -> ShardRecord {
    let mut f = File::open(path).unwrap_or_else(|e| panic!("Failed to open shard {}: {}", path.display(), e));
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap_or_else(|e| panic!("Failed to read shard {}: {}", path.display(), e));

    if buf.len() < 64 {
        return ShardRecord {
            path: path.to_path_buf(),
            valid_signature: false,
            eco_score: None,
        };
    }

    let (payload, sig_bytes) = buf.split_at(buf.len() - 64);
    let sig = match Signature::from_bytes(sig_bytes.try_into().unwrap()) {
        Ok(s) => s,
        Err(_) => {
            return ShardRecord {
                path: path.to_path_buf(),
                valid_signature: false,
                eco_score: None,
            }
        }
    };

    let valid = key.verify(payload, &sig).is_ok();

    // Extract eco_impact_score if present in payload as ASCII "eco:<float>\n"
    let eco_score = String::from_utf8(payload.to_vec())
        .ok()
        .and_then(|s| {
            for line in s.lines() {
                if let Some(rest) = line.strip_prefix("eco:") {
                    return rest.trim().parse::<f32>().ok();
                }
            }
            None
        });

    ShardRecord {
        path: path.to_path_buf(),
        valid_signature: valid,
        eco_score,
    }
}

fn main() {
    let matches = Command::new("autonomy-graph-github")
        .about("ALN autonomy graph shard verifier for GitHub CI")
        .arg(
            Arg::new("shards-dir")
                .long("shards-dir")
                .required(true)
                .value_name("DIR")
                .help("Directory containing *.alnshard files"),
        )
        .arg(
            Arg::new("pubkey")
                .long("pubkey")
                .required(true)
                .value_name("PATH")
                .help("Path to Ed25519 public key for shard verification"),
        )
        .arg(
            Arg::new("verify-signatures")
                .long("verify-signatures")
                .action(clap::ArgAction::SetTrue)
                .help("Enable signature verification for shards"),
        )
        .get_matches();

    let dir = Path::new(matches.get_one::<String>("shards-dir").unwrap());
    let pubkey_path = Path::new(matches.get_one::<String>("pubkey").unwrap());
    let verify = matches.get_flag("verify-signatures");

    if !dir.is_dir() {
        panic!("shards-dir {} is not a directory", dir.display());
    }

    let key = load_key(pubkey_path);
    let mut records: Vec<ShardRecord> = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("alnshard") {
            let rec = if verify {
                verify_shard(&path, &key)
            } else {
                ShardRecord {
                    path: path.clone(),
                    valid_signature: false,
                    eco_score: None,
                }
            };
            records.push(rec);
        }
    }

    if records.is_empty() {
        eprintln!("autonomy-graph-github: no .alnshard files found in {}", dir.display());
        std::process::exit(1);
    }

    // Integrity summary (JSON + hash)
    let mut any_invalid = false;
    let mut eco_sum = 0.0f32;
    let mut eco_count = 0u32;

    for r in &records {
        if verify && !r.valid_signature {
            any_invalid = true;
        }
        if let Some(e) = r.eco_score {
            eco_sum += e;
            eco_count += 1;
        }
    }

    let avg_eco = if eco_count > 0 {
        eco_sum / eco_count as f32
    } else {
        0.0
    };

    let mut hasher = Sha256::new();
    for r in &records {
        hasher.update(r.path.to_string_lossy().as_bytes());
        hasher.update(&[0u8]);
    }
    let hash = hasher.finalize();
    let hash_hex = format!("0x{}", hex::encode(hash));

    if verify && any_invalid {
        eprintln!("autonomy-graph-github: invalid shard signatures detected");
        for r in &records {
            if !r.valid_signature {
                eprintln!("- {}", r.path.display());
            }
        }
        std::process::exit(1);
    }

    println!(
        "{{\"tool\":\"autonomy-graph-github\",\"status\":\"ok\",\"dir\":\"{}\",\"avg_eco_score\":{},\"graph_hash\":\"{}\"}}",
        dir.display(),
        avg_eco,
        hash_hex
    );
}
