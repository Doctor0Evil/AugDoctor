#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Minimal view of evolution-emergency-audit-profilev1.aln.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionEmergencyRightsProfile {
    pub host_id: String,
    pub profile_id: String,
    pub scope_id: String,
    pub max_emergencies_per_day: u32,
    pub require_transcripthash: bool,
    pub require_human_explanation: bool,
    pub explanation_min_words: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EvolutionEmergencyRightsStatus {
    RightsSafe,
    ViolatesInvariant(Vec<String>),
}

impl EvolutionEmergencyRightsProfile {
    /// Mechanical invariant checks, analogous to DeepDomainRightsProfile::verify_rights_safe.[file:39]
    pub fn verify_rights_safe(&self) -> EvolutionEmergencyRightsStatus {
        let mut errors = Vec::new();

        if self.host_id != "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7" {
            errors.push("host_id must be your sovereign host DID".into());
        }
        if self.scope_id != "evolution-emergency-rollback-v1" {
            errors.push("scope_id must be evolution-emergency-rollback-v1".into());
        }
        if self.max_emergencies_per_day == 0 {
            errors.push("max_emergencies_per_day must be >= 1".into());
        }
        if !self.require_transcripthash {
            errors.push("require_transcripthash must be true".into());
        }
        if !self.require_human_explanation {
            errors.push("require_human_explanation must be true".into());
        }
        if self.explanation_min_words < 25 {
            errors.push("explanation_min_words must be >= 25".into());
        }

        if errors.is_empty() {
            EvolutionEmergencyRightsStatus::RightsSafe
        } else {
            EvolutionEmergencyRightsStatus::ViolatesInvariant(errors)
        }
    }
}

/// Per-turn context subset needed to audit emergency rollback use.
#[derive(Clone, Debug)]
pub struct EmergencyTurnContext {
    pub turn_id: String,
    pub block_id: String,
    pub aichatplatform: String,
    pub host_did: String,
    pub transcripthash: Option<String>,
    pub human_explanation: Option<String>,
    pub emergency_scope_id: Option<String>,
    pub emergency_issued_at_utc: Option<String>,
    pub validator_alias: String,
    pub is_rollback: bool,
}

/// Result kind reused from your per-turn validation pattern.[file:39]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmergencyValidationKind {
    Passed,
    Failed,
    Skipped,
}

#[derive(Clone, Debug)]
pub struct EmergencyValidationResult {
    pub kind: EmergencyValidationKind,
    pub messages: Vec<String>,
}

/// Check invariants when an EmergencyOverride is present and a rollback is requested.
pub fn validate_emergency_override_turn(
    rights: &EvolutionEmergencyRightsProfile,
    ctx: &EmergencyTurnContext,
    emergencies_used_today: u32,
) -> EmergencyValidationResult {
    let mut msgs = Vec::new();

    if !ctx.is_rollback {
        return EmergencyValidationResult {
            kind: EmergencyValidationKind::Skipped,
            messages: vec!["no rollback requested this turn".into()],
        };
    }

    // Profile must itself be RightsSafe (verified at startup as with DeepDomainRights).[file:39]
    if let EvolutionEmergencyRightsStatus::ViolatesInvariant(errs) = rights.verify_rights_safe() {
        return EmergencyValidationResult {
            kind: EmergencyValidationKind::Failed,
            messages: vec![
                "EvolutionEmergencyRightsProfile violates invariants".into(),
                format!("errors: {:?}", errs),
            ],
        };
    }

    // Scope and host binding.
    match &ctx.emergency_scope_id {
        Some(scope) if scope == &rights.scope_id => {}
        Some(scope) => {
            msgs.push(format!("emergency scope_id {} does not match rights scope_id {}", scope, rights.scope_id));
            return EmergencyValidationResult { kind: EmergencyValidationKind::Failed, messages: msgs };
        }
        None => {
            msgs.push("rollback requested but no emergency scope_id present".into());
            return EmergencyValidationResult { kind: EmergencyValidationKind::Failed, messages: msgs };
        }
    }

    if ctx.host_did != rights.host_id {
        msgs.push("host_did must equal rights.host_id for emergency rollback".into());
        return EmergencyValidationResult { kind: EmergencyValidationKind::Failed, messages: msgs };
    }

    // Transcript hash and explanation requirements.
    if rights.require_transcripthash {
        if ctx.transcripthash.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
            msgs.push("transcripthash must be non-empty for emergency rollback turns".into());
            return EmergencyValidationResult { kind: EmergencyValidationKind::Failed, messages: msgs };
        }
    }

    if rights.require_human_explanation {
        match &ctx.human_explanation {
            Some(exp) => {
                let words = exp.split_whitespace().count() as u32;
                if words < rights.explanation_min_words {
                    msgs.push(format!(
                        "human explanation too short: {} words (min {})",
                        words, rights.explanation_min_words
                    ));
                    return EmergencyValidationResult { kind: EmergencyValidationKind::Failed, messages: msgs };
                }
            }
            None => {
                msgs.push("missing human explanation for emergency rollback turn".into());
                return EmergencyValidationResult { kind: EmergencyValidationKind::Failed, messages: msgs };
            }
        }
    }

    // Daily cap enforcement; above-cap uses become log-only.[file:42]
    if emergencies_used_today >= rights.max_emergencies_per_day {
        msgs.push(format!(
            "max_emergencies_per_day {} exceeded; treating rollback as log-only",
            rights.max_emergencies_per_day
        ));
        return EmergencyValidationResult {
            kind: EmergencyValidationKind::Failed,
            messages: msgs,
        };
    }

    EmergencyValidationResult {
        kind: EmergencyValidationKind::Passed,
        messages: vec!["emergency override invariants satisfied for this rollback".into()],
    }
}
