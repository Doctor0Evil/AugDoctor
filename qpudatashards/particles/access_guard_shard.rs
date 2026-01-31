use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AccessPath {
    AllowInternal { reason: String },
    DeferAudit { audit_id: Uuid, reason: String },
    DenyExternal { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeforceBand {
    pub level: String, // "green", "yellow", "red"
    pub quota_pct: f64, // 0.0-1.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcoBudget {
    pub nj_per_flop: f64, // nanoJoules per FLOP
    pub remaining: f64, // current envelope
    pub cap: f64, // max per epoch
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScaleTurns {
    pub daily_cap: u32, // e.g., 1440 turns/day
    pub used: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessInput {
    pub did: String, // DID/ALN/Bostrom format
    pub role: String, // "augmented_citizen", "authorized_researcher"
    pub consent_valid: bool,
    pub lifeforce: LifeforceBand,
    pub eco: EcoBudget,
    pub scale: ScaleTurns,
    pub op_type: String, // e.g., "token_spend", "neural_rope_append"
}

#[derive(Debug, Error)]
pub enum GuardError {
    #[error("invalid DID format: {0}")]
    InvalidDid(String),
    #[error("unauthorized role: {0}")]
    UnauthorizedRole(String),
    #[error("consent invalid")]
    ConsentInvalid,
    #[error("lifeforce band too low: {0}")]
    LifeforceLow(String),
    #[error("eco budget exceeded: remaining {0} < required")]
    EcoExceeded(f64),
    #[error("scale turns cap hit: used {0} >= cap {1}")]
    TurnsCap(u32, u32),
}

pub struct AccessGuardShard {
    allowed_roles: HashMap<String, bool>,
}

impl AccessGuardShard {
    pub fn new() -> Self {
        let mut roles = HashMap::new();
        roles.insert("augmented_citizen".to_string(), true);
        roles.insert("authorized_researcher".to_string(), true);
        AccessGuardShard { allowed_roles }
    }

    pub fn evaluate(&self, input: &AccessInput) -> Result<AccessPath, GuardError> {
        // DID format check
        if !input.did.starts_with("did:") && !input.did.starts_with("aln:") && !input.did.starts_with("bostrom:") {
            return Err(GuardError::InvalidDid(input.did.clone()));
        }

        // Role authorization
        if !self.allowed_roles.contains_key(&input.role) {
            return Err(GuardError::UnauthorizedRole(input.role.clone()));
        }

        // Consent invariant
        if !input.consent_valid {
            return Err(GuardError::ConsentInvalid);
        }

        // Lifeforce band
        match input.lifeforce.level.as_str() {
            "green" if input.lifeforce.quota_pct >= 0.75 => Ok(()),
            "yellow" if input.lifeforce.quota_pct >= 0.40 => Ok(()),
            "red" | _ => Err(GuardError::LifeforceLow(input.lifeforce.level.clone())),
        }?;

        // Eco budget
        if input.eco.remaining < input.eco.nj_per_flop * 1.2 { // 20% safety margin
            return Err(GuardError::EcoExceeded(input.eco.remaining));
        }

        // SCALE turns cap
        if input.scale.used >= input.scale.daily_cap {
            return Err(GuardError::TurnsCap(input.scale.used, input.scale.daily_cap));
        }

        // Path decision
        if input.role == "augmented_citizen" && input.op_type.contains("self") {
            Ok(AccessPath::AllowInternal { reason: "Self-aug approved".to_string() })
        } else if input.role == "authorized_researcher" {
            Ok(AccessPath::DeferAudit { audit_id: Uuid::new_v4(), reason: "Research review queued".to_string() })
        } else {
            Ok(AccessPath::DenyExternal { reason: "External access blocked".to_string() })
        }
    }
}

// Usage: let guard = AccessGuardShard::new();
// let input = AccessInput { did: "bostrom:host123", role: "augmented_citizen", consent_valid: true, lifeforce: LifeforceBand { level: "green".to_string(), quota_pct: 0.85 }, eco: EcoBudget { nj_per_flop: 52.0, remaining: 1000.0, cap: 50000.0 }, scale: ScaleTurns { daily_cap: 1440, used: 1200 }, op_type: "token_spend_self_aug".to_string() };
// match guard.evaluate(&input) { Ok(path) => path, Err(e) => panic!("{}", e) }
