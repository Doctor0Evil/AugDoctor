use crate::trace::AutonomyTraceAttributes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyProfile {
    pub profile_id: String,
    pub host_id: String,
    pub max_eco_energy_nj_per_minute: f32,
    pub max_autonomous_actions_per_minute: u32,
    pub max_risk_score: f32,
    pub min_lifeforce_scalar: f32,
    pub max_identity_drift_per_day: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyDecision {
    pub autonomy_level: f32,
    pub shot_level_label: String,
}

pub struct AutonomyGovernor;

impl AutonomyGovernor {
    /// Hard-ordered decision logic: identity → consent → lifeforce → eco → SCALE/turns.
    pub fn decide(
        profile: &AutonomyProfile,
        trace: &AutonomyTraceAttributes,
        host_consent_active: bool,
    ) -> AutonomyDecision {
        // Temporal-logic style runtime assertions (LTL fragments).
        debug_assert!(
            !(trace.constraint.highest_risk_score > 0.7 && trace.decision_autonomy_level > 0.8),
            "G(risk_high -> autonomy <= θ) violated"
        );
        debug_assert!(
            trace.lifeforce_scalar >= 0.0 && trace.lifeforce_scalar <= 1.0,
            "lifeforce_scalar must be normalized"
        );

        // 1. Identity / consent (upstream identity header already checked in inner-ledger).[file:41]
        if !host_consent_active {
            return AutonomyDecision {
                autonomy_level: 0.0,
                shot_level_label: "ZeroShot".into(),
            };
        }

        // 2. Lifeforce (hard veto).[file:42][file:41]
        if trace.lifeforce_scalar < profile.min_lifeforce_scalar {
            return AutonomyDecision {
                autonomy_level: 0.0,
                shot_level_label: "ZeroShot".into(),
            };
        }

        // 3. Eco budget.[file:42]
        if trace.eco_energy_nj > profile.max_eco_energy_nj_per_minute {
            return AutonomyDecision {
                autonomy_level: 0.2,
                shot_level_label: "ZeroShot".into(),
            };
        }

        // 4. Risk.[file:41]
        if trace.risk > profile.max_risk_score {
            return AutonomyDecision {
                autonomy_level: 0.0,
                shot_level_label: "ZeroShot".into(),
            };
        }

        // 5. SCALE / turns (approximated by actions_last_minute and identity drift).[file:42][file:41]
        if trace.actions_last_minute >= profile.max_autonomous_actions_per_minute {
            return AutonomyDecision {
                autonomy_level: 0.1,
                shot_level_label: "ZeroShot".into(),
            };
        }
        if trace.identity_drift_today > profile.max_identity_drift_per_day {
            return AutonomyDecision {
                autonomy_level: 0.0,
                shot_level_label: "ZeroShot".into(),
            };
        }

        // Smooth assist ramp (Lipschitz-bounded).[file:41]
        let mut autonomy = {
            let s = (1.0 - trace.stress).max(0.0);
            let f = (1.0 - trace.fatigue).max(0.0);
            let r = (trace.reward + 1.0) / 2.0; // [-1,1] → [0,1]
            let sa = trace.safety.max(0.0);
            (0.4 * s + 0.2 * f + 0.2 * r + 0.2 * sa).clamp(0.0, 1.0)
        };

        if trace.eco_energy_nj > 0.7 * profile.max_eco_energy_nj_per_minute {
            autonomy *= 0.7;
        }

        let shot = if trace.risk > 0.7 { "FewShot" } else { "ZeroShot" };

        AutonomyDecision {
            autonomy_level: autonomy,
            shot_level_label: shot.into(),
        }
    }
}
