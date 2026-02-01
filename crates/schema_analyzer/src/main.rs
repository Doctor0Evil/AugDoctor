use serde_json::Value;
use std::fs::File;
use std::io::{self, BufRead};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyzerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing relation: {0}")]
    MissingRelation(String),
    #[error("Invalid column type: {0}")]
    InvalidType(String),
}

fn main() -> Result<(), AnalyzerError> {
    let file = File::open("export.json.txt.txt")?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();

    // Parse schema init
    let mut schema_json = String::new();
    while let Some(line) = lines.next()? {
        schema_json.push_str(&line);
        if line.contains("]}") { break; } // Approximate end of JSON
    }

    let schema: Value = serde_json::from_str(&schema_json)?;
    let relations = schema["data"][0].as_object().ok_or(AnalyzerError::InvalidType("data[0]".to_string()))?;

    // Console output: relations and columns
    println!("Functions/Calls:");
    println!("load_schema: parsed {} relations", relations.len());
    for (rel_name, rel) in relations {
        let keys = rel["keys"].as_array().ok_or(AnalyzerError::InvalidType(rel_name.to_string()))?;
        let values = rel["values"].as_array().ok_or(AnalyzerError::InvalidType(rel_name.to_string()))?;
        println!("Relation: {} | Keys: {:?} | Values: {:?}", rel_name, keys, values);
    }

    // Particle list analysis
    println!("\nParticle List:");
    let mut particle_count = 0;
    while let Some(line) = lines.next()? {
        if line.contains("Qm") { // CID marker
            particle_count += 1;
            println!("Line {}: {}", particle_count, line);
        }
    }
    println!("Total Particles: {}", particle_count);

    Ok(())
}
