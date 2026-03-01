use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DonutEntry {
    pub hexstamp: String,          // 64-char hex SHA256
    pub prev_hexstamp: String,
    pub evolve_ref: String,
    pub roh_slice: f64,            // current RoH value
    pub evolution_points: u64,     // new quantum-learning reward
    pub soul_nontradeable: bool,
    pub neurorights_flags: Vec<String>, // e.g. ["mental_privacy", "cognitive_liberty"]
}

pub fn verify_chain(path: &str) -> io::Result<Vec<DonutEntry>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut chain: Vec<DonutEntry> = vec![];
    let mut prev = "0x0000000000000000000000000000000000000000000000000000000000000000".to_string();

    for line in reader.lines() {
        let line = line?;
        let entry: DonutEntry = serde_json::from_str(&line)?;
        if entry.prev_hexstamp != prev {
            panic!("Donutloop breakage at hexstamp {}", entry.hexstamp);
        }
        if entry.roh_slice > 0.3 {
            panic!("RoH ceiling 0.3 violated: {}", entry.roh_slice);
        }
        // New BioHashQuantum – 5D biophysical scoring
        let mut hasher = Sha256::new();
        hasher.update(entry.hexstamp.as_bytes());
        hasher.update(entry.roh_slice.to_le_bytes());
        let bio_entropy = hasher.finalize();
        let points = (bio_entropy[0] as u64) % 100 + 10; // lab-grade deterministic reward
        println!("[BioHashQuantum] evolution_points_granted={} for entry {}", points, entry.hexstamp);
        
        prev = entry.hexstamp.clone();
        chain.push(entry);
    }
    Ok(chain)
}

pub fn append_entry(path: &str, evolve_ref: &str, roh: f64) -> io::Result<()> {
    let mut chain = verify_chain(path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(format!("{:?}", chain.last()));
    let new_hash = format!("0x{:x}", hasher.finalize());
    let entry = DonutEntry {
        hexstamp: new_hash.clone(),
        prev_hexstamp: chain.last().map(|e| e.hexstamp.clone()).unwrap_or_else(|| "0x0000...".to_string()),
        evolve_ref: evolve_ref.to_string(),
        roh_slice: roh,
        evolution_points: 42,
        soul_nontradeable: true,
        neurorights_flags: vec!["mental_privacy".to_string(), "cognitive_liberty".to_string()],
    };
    let mut file = File::options().append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(&entry)?)?;
    println!("[DONUTLOOP] Appended entry {} | RoH={} | Points=42", new_hash, roh);
    Ok(())
}
