use crate::types::{
    AssistantAutonomyDecision,
    AssistantAutonomyProfile,
    AssistantAutonomyReason,
};
use crate::recorder::BiomarkerAggregation;
use serde::{Deserialize, Serialize};

/// State needed to track identity drift and action rates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanionAutonomyState {
    pub host_id: String,
    pub profile_id: String,
    pub actions_last_minute: u32,
    pub identity_drift_today: f32, // e.g., KL accumulator
}

#[derive(Clone, Debug)]
pub struct CompanionAutonomyGovernor;

impl CompanionAutonomyGovernor {
    pub fn decide(
        profile: &AssistantAutonomyProfile,
        state: &CompanionAutonomyState,
        agg: &BiomarkerAggregation,
        host_consent_active: bool,
    ) -> AssistantAutonomyDecision {
        // 1. Hard gates.
        if !host_consent_active && !profile.allow_self_tuning_with_consent {
            return Self::deny(profile, AssistantAutonomyReason::HostConsentMissing);
        }
        if agg.worst_lifeforce_scalar < profile.min_lifeforce_scalar {
            return Self::deny(profile, AssistantAutonomyReason::LifeforceTooLow);
        }
        if agg.highest_risk_score > profile.max_risk_score {
            return Self::deny(profile, AssistantAutonomyReason::RiskTooHigh);
        }
        if agg.avg_eco_energy_nj > profile.max_eco_energy_nj_per_minute {
            return Self::downgrade(profile, 0.2, AssistantAutonomyReason::EcoBudgetExceeded);
        }
        if state.actions_last_minute >= profile.max_autonomous_actions_per_minute {
            return Self::downgrade(profile, 0.1, AssistantAutonomyReason::TooManyActionsRecently);
        }
        if state.identity_drift_today > profile.max_identity_drift_per_day {
            return Self::downgrade(profile, 0.0, AssistantAutonomyReason::IdentityDriftLimit);
        }

        // 2. Soft shaping: more autonomy when biomarker envelope is healthy.
        let stress = agg.avg_stress;
        let fatigue = agg.avg_fatigue;
        let reward = agg.avg_reward;
        let safety = agg.avg_safety_margin;

        let health = (1.0 - stress).max(0.0) * 0.4
            + (1.0 - fatigue).max(0.0) * 0.2
            + reward.max(0.0) * 0.2
            + safety.max(0.0) * 0.2;

        let mut autonomy_level = health.clamp(0.0, 1.0);

        // 3. Apply small penalty if eco cost is high but still in-band.
        if agg.avg_eco_energy_nj > 0.7 * profile.max_eco_energy_nj_per_minute {
            autonomy_level *= 0.7;
        }

        // 4. Shot-level routing suggestion.
        let shot_label = if agg.highest_risk_score > 0.7 {
            "FewShot".to_string()
        } else {
            "ZeroShot".to_string()
        };

        AssistantAutonomyDecision {
            profile_id: profile.profile_id.clone(),
            host_id: profile.host_id.clone(),
            autonomy_level,
            shot_level_label: shot_label,
            may_act_without_explicit_confirm: autonomy_level > 0.5,
            primary_reason: AssistantAutonomyReason::WithinAllBudgets,
        }
    }

    fn deny(profile: &AssistantAutonomyProfile, reason: AssistantAutonomyReason) -> AssistantAutonomyDecision {
        AssistantAutonomyDecision {
            profile_id: profile.profile_id.clone(),
            host_id: profile.host_id.clone(),
            autonomy_level: 0.0,
            shot_level_label: "ZeroShot".to_string(),
            may_act_without_explicit_confirm: false,
            primary_reason: reason,
        }
    }

    fn downgrade(
        profile: &AssistantAutonomyProfile,
        level: f32,
        reason: AssistantAutonomyReason,
    ) -> AssistantAutonomyDecision {
        AssistantAutonomyDecision {
            profile_id: profile.profile_id.clone(),
            host_id: profile.host_id.clone(),
            autonomy_level: level.clamp(0.0, 1.0),
            shot_level_label: if level > 0.3 { "Hybrid".to_string() } else { "ZeroShot".to_string() },
            may_act_without_explicit_confirm: level > 0.5,
            primary_reason: reason,
        }
    }
}
