use crate::security::CivicClass;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CivicRewardProfile {
    pub multiplier_min: f64,
    pub multiplier_max: f64,
    pub default_multiplier: f64,
    pub required_knowledge_factor: f32,

    pub heroic_tags: HashSet<String>,
    pub heroic_multiplier: f64,

    pub good_tags: HashSet<String>,
    pub good_multiplier: f64,

    pub neutral_multiplier: f64,

    pub disallowed_tags: HashSet<String>,

    pub eco_bonus_enabled: bool,
    pub eco_low_threshold: f64,
    pub eco_low_bonus: f64,
}

impl CivicRewardProfile {
    /// Load from JSON generated from the ALN qpudatashard.
    pub fn load_from_json<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let raw = fs::read_to_string(path)?;
        let mut profile: CivicRewardProfile = serde_json::from_str(&raw)?;
        profile.normalize();
        Ok(profile)
    }

    fn normalize(&mut self) {
        if self.multiplier_min < 0.0 {
            self.multiplier_min = 0.0;
        }
        if self.multiplier_max < self.multiplier_min {
            self.multiplier_max = self.multiplier_min;
        }

        self.heroic_multiplier = self
            .heroic_multiplier
            .clamp(self.multiplier_min, self.multiplier_max);
        self.good_multiplier = self
            .good_multiplier
            .clamp(self.multiplier_min, self.multiplier_max);
        self.neutral_multiplier = self
            .neutral_multiplier
            .clamp(self.multiplier_min, self.multiplier_max);

        // Eco bonus cannot push above multiplier_max.
        if self.eco_low_bonus < 1.0 {
            self.eco_low_bonus = 1.0;
        }
        let max_allowed = if self.heroic_multiplier > 0.0 {
            self.multiplier_max / self.heroic_multiplier
        } else {
            1.0
        };
        if self.eco_low_bonus > max_allowed {
            self.eco_low_bonus = max_allowed;
        }
    }

    pub fn classify(&self, tags: &[String]) -> CivicClass {
        let lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();

        if lower
            .iter()
            .any(|t| self.disallowed_tags.contains(t.as_str()))
        {
            return CivicClass::Disallowed;
        }
        if lower.iter().any(|t| self.heroic_tags.contains(t.as_str())) {
            return CivicClass::CivicHeroic;
        }
        if lower.iter().any(|t| self.good_tags.contains(t.as_str())) {
            return CivicClass::CivicGood;
        }
        CivicClass::Neutral
    }

    pub fn base_multiplier(&self, class: CivicClass) -> f64 {
        let raw = match class {
            CivicClass::CivicHeroic => self.heroic_multiplier,
            CivicClass::CivicGood => self.good_multiplier,
            CivicClass::Neutral => self.neutral_multiplier,
            CivicClass::Disallowed => 0.0,
        };
        raw.clamp(self.multiplier_min, self.multiplier_max)
    }

    pub fn eco_adjusted_multiplier(&self, class: CivicClass, eco_cost: f64) -> f64 {
        let mut m = self.base_multiplier(class);
        if self.eco_bonus_enabled && eco_cost <= self.eco_low_threshold && m > 0.0 {
            m *= self.eco_low_bonus;
        }
        m.clamp(self.multiplier_min, self.multiplier_max)
    }
}
