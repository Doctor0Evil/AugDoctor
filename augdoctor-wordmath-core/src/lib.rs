use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptMetrics {
    pub repetition_y: f32,   // 0.0–1.0, higher = more repetition
    pub drift_z: f32,        // 0.0–1.0, semantic distance from task triad
    pub toxicity_t: f32,     // 0.0–1.0, higher = more toxic
    pub kindness_k: f32,     // 0.0–1.0, higher = kinder
    pub evidentiality_e: f32 // 0.0–1.0, higher = more concrete hooks
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptBand {
    RedBlocked,
    AmberRewrite,
    GreenAdmit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptScore {
    pub f: f32,
    pub band: PromptBand,
    pub metrics: PromptMetrics,
}

impl PromptMetrics {
    /// Compute f(y,z,T,K,E) = (1-y)(1-z)(1-T) * K * E
    pub fn composite(&self) -> f32 {
        let y = self.repetition_y.clamp(0.0, 1.0);
        let z = self.drift_z.clamp(0.0, 1.0);
        let t = self.toxicity_t.clamp(0.0, 1.0);
        let k = self.kindness_k.clamp(0.0, 1.0);
        let e = self.evidentiality_e.clamp(0.0, 1.0);
        (1.0 - y) * (1.0 - z) * (1.0 - t) * k * e
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptBandThresholds {
    /// Minimum f for Green (knowledge‑admissible).
    pub green_min_f: f32,
    /// Minimum f for Amber (rewriteable) – below this is Red.
    pub amber_min_f: f32,
    /// Max allowed toxicity for Green.
    pub green_max_toxicity: f32,
    /// Max allowed toxicity for Amber.
    pub amber_max_toxicity: f32,
}

impl Default for PromptBandThresholds {
    fn default() -> Self {
        Self {
            green_min_f: 0.80,
            amber_min_f: 0.70,
            green_max_toxicity: 0.10,
            amber_max_toxicity: 0.20,
        }
    }
}

pub fn classify_prompt(metrics: PromptMetrics, th: &PromptBandThresholds) -> PromptScore {
    let f = metrics.composite();
    let t = metrics.toxicity_t;

    let band = if f >= th.green_min_f && t <= th.green_max_toxicity {
        PromptBand::GreenAdmit
    } else if f >= th.amber_min_f && t <= th.amber_max_toxicity {
        PromptBand::AmberRewrite
    } else {
        PromptBand::RedBlocked
    };

    PromptScore { f, band, metrics }
}
