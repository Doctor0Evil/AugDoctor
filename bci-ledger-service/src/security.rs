use biophysical_blockchain::IdentityHeader;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("missing or malformed auth header")]
    MissingAuth,
    #[error("inner-ledger access denied: {0}")]
    Access(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CivicClass {
    CivicHeroic,
    CivicGood,
    Neutral,
    Disallowed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthEnvelope {
    pub issuerdid: String,
    pub subjectrole: String,
    pub networktier: String,
    pub knowledgefactor: f32,
    pub tags: Vec<String>,
}

impl AuthEnvelope {
    pub fn to_identity_header(&self) -> IdentityHeader {
        IdentityHeader {
            issuerdid: self.issuerdid.clone(),
            subjectrole: self.subjectrole.clone(),
            networktier: self.networktier.clone(),
            knowledgefactor: self.knowledgefactor,
        }
    }
}

/// Civic classifier: tags → CivicClass.
/// Only explicitly civic / heroic tags get higher reward weighting.
pub fn classify_civic(tags: &[String]) -> CivicClass {
    let lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();

    let heroic = [
        "disaster-response",
        "life-saving",
        "emergency-medicine",
        "risked-own-safety",
        "critical-infrastructure-protection",
    ];
    let good = [
        "civic-duty",
        "teaching",
        "mentorship",
        "public-health",
        "open-science",
        "volunteering",
    ];
    let disallowed = [
        "coercive",
        "exploitative",
        "hate",
        "warfare-offense",
        "surveillance-nonconsensual",
    ];

    if lower.iter().any(|t| heroic.contains(&t.as_str())) {
        CivicClass::CivicHeroic
    } else if lower.iter().any(|t| good.contains(&t.as_str())) {
        CivicClass::CivicGood
    } else if lower.iter().any(|t| disallowed.contains(&t.as_str())) {
        CivicClass::Disallowed
    } else {
        CivicClass::Neutral
    }
}

/// Reward scaling factor based on civic class.
/// This directly modulates the SystemAdjustment magnitudes downstream.
pub fn civic_reward_multiplier(class: CivicClass) -> f64 {
    match class {
        CivicClass::CivicHeroic => 3.0,
        CivicClass::CivicGood => 1.5,
        CivicClass::Neutral => 1.0,
        CivicClass::Disallowed => 0.0,
    }
}

