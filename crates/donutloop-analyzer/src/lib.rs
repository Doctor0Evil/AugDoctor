use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::fs::File;

#[derive(Deserialize, Clone)]
pub struct DonutloopEntry {
    pub proposal_id: String,
    pub seq: u64,
    pub host_did: String,
    pub roh_before: f64,
    pub roh_after: f64,
    pub knowledge_factor_before: f64,
    pub knowledge_factor_after: f64,
    pub cybostate_before: CyboState,
    pub cybostate_after: CyboState,
    pub decision: String,
    pub scope: String,
    pub token_type: String,
    pub personal_envelope_event: PersonalEnvelopeEvent,
}

#[derive(Deserialize, Clone)]
pub struct CyboState {
    pub c_value: f32,
    pub band: String,
}

#[derive(Deserialize, Clone)]
pub struct PersonalEnvelopeEvent {
    pub pain_band: String,
    pub fear_band: String,
    pub cognitive_band: String,
    pub approaching_threshold: bool,
    pub instinct_vetoed: bool,
    pub explicit_spend: bool,
}

pub struct Summary {
    pub n_entries: u64,
    pub roh_violations: u64,
    pub k_total_delta: f64,
    pub n_k_positive: u64,
    pub n_high_envelope_events: u64,
    pub n_instinct_vetoes: u64,
}

pub fn analyze_donutloop(path: &str) -> Result<Summary, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let mut summary = Summary {
        n_entries: 0,
        roh_violations: 0,
        k_total_delta: 0.0,
        n_k_positive: 0,
        n_high_envelope_events: 0,
        n_instinct_vetoes: 0,
    };

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() { continue; }

        let entry: DonutloopEntry =
            serde_json::from_str(&line).map_err(|e| format!("parse error: {e}"))?;

        summary.n_entries += 1;

        if entry.roh_after > 0.3 {
            summary.roh_violations += 1;
        }

        let k_delta = entry.knowledge_factor_after - entry.knowledge_factor_before;
        summary.k_total_delta += k_delta;
        if k_delta > 0.0 {
            summary.n_k_positive += 1;
        }

        let high_envelope =
            entry.personal_envelope_event.pain_band == "HIGH" ||
            entry.personal_envelope_event.fear_band == "HIGH";

        if high_envelope && entry.personal_envelope_event.approaching_threshold {
            summary.n_high_envelope_events += 1;
        }
        if entry.personal_envelope_event.instinct_vetoed {
            summary.n_instinct_vetoes += 1;
        }
    }

    Ok(summary)
}
