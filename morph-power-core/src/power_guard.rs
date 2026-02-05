use serde::{Deserialize, Serialize};

use crate::types::{
    PowerBudget, PowerContext, PowerDecision, PowerProhibitionReason,
};

/// Static configuration for POWER behavior, compiled into inner-core crates.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PowerGuardConfig {
    pub eco_score_floor_for_high_power: f32,   // e.g. 0.6
    pub max_corridor_soft: f32,               // e.g. 0.8
    pub max_corridor_hard: f32,               // e.g. 1.0
    pub civic_impact_tighten_threshold: f32,  // e.g. 0.7
    pub eco_load_tighten_threshold: f32,      // e.g. 0.7
    pub high_impact_actions_threshold: u32,   // e.g. 5
}

/// POWER governor: pure function, no side-effects.
pub struct PowerGovernor {
    cfg: PowerGuardConfig,
}

impl PowerGovernor {
    pub fn new(cfg: PowerGuardConfig) -> Self {
        Self { cfg }
    }

    /// Decide POWER for this turn based on host context.
    ///
    /// This function MUST be called before any agentic AI action is executed.
    /// It must not mutate BioTokenState or inner-ledger; it only returns limits.
    pub fn decide(&self, ctx: &PowerContext) -> PowerDecision {
        let mut reasons = Vec::new();
        let mut corridor = self.cfg.max_corridor_hard;

        // 1. Lifeforce first (HardStop, SoftWarn).
        match ctx.lifeforce_band_code {
            2 => {
                // HardStop: zero POWER, propose-only.
                reasons.push(PowerProhibitionReason::HardStopLifeforce);
                return PowerDecision {
                    allowed_corridor: 0.0,
                    reasons,
                    propose_only: true,
                    max_documents: 0,
                    max_external_calls: 0,
                };
            }
            1 => {
                reasons.push(PowerProhibitionReason::SoftWarnLifeforce);
                corridor = corridor.min(0.2);
            }
            _ => {}
        }

        // 2. Tighten based on recent civic / cryptographic impact.
        if ctx.civic_impact >= self.cfg.civic_impact_tighten_threshold {
            reasons.push(PowerProhibitionReason::HighCivicImpact);
            corridor = corridor.min(0.4);
        }

        // 3. Tighten based on eco load.
        if ctx.eco_load >= self.cfg.eco_load_tighten_threshold {
            reasons.push(PowerProhibitionReason::HighEcoLoad);
            corridor = corridor.min(0.3);
        }

        // 4. Tighten when recent high-impact actions are dense.
        if ctx.recent_high_impact_actions > self.cfg.high_impact_actions_threshold {
            reasons.push(PowerProhibitionReason::TooManyRecentHighImpactActions);
            corridor = corridor.min(0.2);
        }

        // 5. Tie maximum POWER to longitudinal eco behavior.
        if ctx.eco_score_longitudinal < self.cfg.eco_score_floor_for_high_power {
            reasons.push(PowerProhibitionReason::PoorEcoBehavior);
            corridor = corridor.min(0.3);
        }

        // Clamp to [0, max_corridor_soft]; never exceed compiled soft max.
        if corridor > self.cfg.max_corridor_soft {
            corridor = self.cfg.max_corridor_soft;
        }
        if corridor < 0.0 {
            corridor = 0.0;
        }

        if reasons.is_empty() {
            reasons.push(PowerProhibitionReason::DefaultFloor);
        }

        // Map corridor into discrete limits for this turn.
        let max_documents = if corridor == 0.0 {
            0
        } else if corridor < 0.25 {
            2
        } else if corridor < 0.5 {
            5
        } else if corridor < 0.75 {
            10
        } else {
            20
        };

        let max_external_calls = if corridor == 0.0 {
            0
        } else if corridor < 0.25 {
            1
        } else if corridor < 0.5 {
            2
        } else if corridor < 0.75 {
            3
        } else {
            5
        };

        let propose_only = corridor <= 0.4;

        PowerDecision {
            allowed_corridor: corridor,
            reasons,
            propose_only,
            max_documents,
            max_external_calls,
        }
    }
}
