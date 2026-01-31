use crate::types::{AssistantAutonomyDecision, AssistantAutonomyReason};
use crate::recorder::BiomarkerAggregation;
use serde::{Deserialize, Serialize};

/// Minimal attribute payload that will ride on the NeuralRope segment.
/// This stays strictly non-identity-bearing: no DID, no consciousness flags.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutonomyTraceAttributes {
    pub planelabel: String,          // e.g., "neuromorph.softwareonly", "bci.hci.eeg"
    pub session_id: String,          // ephemeral, not a DID
    pub environment_id: String,      // device / OS plane, not a human ID
    pub autonomy_level: f32,         // [0,1]
    pub shot_level_label: String,    // "ZeroShot" | "FewShot" | "Hybrid"
    pub may_act_without_confirm: bool,
    pub primary_reason: String,      // stringified AssistantAutonomyReason

    // Collapsed biomarker + eco envelope used for learning assistive policies.
    pub avg_stress: f32,
    pub avg_fatigue: f32,
    pub avg_cognitive_load: f32,
    pub avg_eco_energy_nj: f32,
    pub avg_reward: f32,
    pub avg_safety_margin: f32,
    pub worst_lifeforce_scalar: f32,
    pub highest_risk_score: f32,
}

/// Trait alias so we do not depend on a specific neuralrope crate directly.
/// Any rope type that implements this surface can be used.
pub trait NeuralRopeLike {
    /// Append a textual trace with attributes over its span.
    ///
    /// Signature intentionally mirrors the existing NeuralRope::append_trace:
    ///   fn append_trace(
    ///       &mut self,
    ///       trace: &str,
    ///       plane_label: &str,
    ///       bioscale_upgrade_id: Option<String>,
    ///       reward_score: f32,
    ///       safety_decision: &str,
    ///   );
    fn append_trace(
        &mut self,
        trace: &str,
        plane_label: &str,
        bioscale_upgrade_id: Option<String>,
        reward_score: f32,
        safety_decision: &str,
    );
}

/// Helper that knows how to format autonomy decisions + biomarkers
/// into short, non-identity rope segments for assisted-policy learning.
pub struct AutonomyNeuralRopeHelper;

impl AutonomyNeuralRopeHelper {
    /// Append a single autonomy decision + biomarker aggregate into the rope.
    ///
    /// `plane_label` is the environment plane for this decision
    ///   e.g., "neuromorph.softwareonly", "bci.hci.eeg", "system.runtime".
    /// `bioscale_upgrade_id` is optional and may be None for pure software-only decisions.
    pub fn append_decision_trace<R: NeuralRopeLike>(
        rope: &mut R,
        plane_label: &str,
        bioscale_upgrade_id: Option<String>,
        decision: &AssistantAutonomyDecision,
        agg: &BiomarkerAggregation,
    ) {
        let attrs = AutonomyTraceAttributes {
            planelabel: plane_label.to_string(),
            session_id: agg.session_id.clone(),
            environment_id: format!("env:{}", decision.host_id), // host_id is a DID, but we only keep an abstract env tag.
            autonomy_level: decision.autonomy_level,
            shot_level_label: decision.shot_level_label.clone(),
            may_act_without_confirm: decision.may_act_without_explicit_confirm,
            primary_reason: format!("{:?}", decision.primary_reason),
            avg_stress: agg.avg_stress,
            avg_fatigue: agg.avg_fatigue,
            avg_cognitive_load: agg.avg_cognitive_load,
            avg_eco_energy_nj: agg.avg_eco_energy_nj,
            avg_reward: agg.avg_reward,
            avg_safety_margin: agg.avg_safety_margin,
            worst_lifeforce_scalar: agg.worst_lifeforce_scalar,
            highest_risk_score: agg.highest_risk_score,
        };

        // Format a compact, text-only trace that all AI-chats can consume.
        let trace_text = format!(
            "AUTONOMYTRACE session={} plane={} level={:.3} shot={} reason={:?} \
             stress={:.3} fatigue={:.3} cog={:.3} eco_nj={:.3} reward={:.3} safety={:.3} \
             lifeforce_min={:.3} risk_max={:.3}",
            attrs.session_id,
            attrs.planelabel,
            attrs.autonomy_level,
            attrs.shot_level_label,
            decision.primary_reason,
            attrs.avg_stress,
            attrs.avg_fatigue,
            attrs.avg_cognitive_load,
            attrs.avg_eco_energy_nj,
            attrs.avg_reward,
            attrs.avg_safety_margin,
            attrs.worst_lifeforce_scalar,
            attrs.highest_risk_score,
        );

        // Reward signal is taken directly from the biomarker aggregation average;
        // safety decision is the autonomy reason string.
        let reward_for_learning = attrs.avg_reward;
        let safety_decision = attrs.primary_reason.clone();

        rope.append_trace(
            &trace_text,
            plane_label,
            bioscale_upgrade_id,
            reward_for_learning,
            &safety_decision,
        );
    }

    /// Convenience: compute an abstract plane label from the decision reason.
    /// This lets external agents group traces without seeing raw host or consciousness state.
    pub fn plane_from_reason(reason: &AssistantAutonomyReason) -> &'static str {
        match reason {
            AssistantAutonomyReason::WithinAllBudgets => "neuromorph.softwareonly",
            AssistantAutonomyReason::EcoBudgetExceeded => "eco.high",
            AssistantAutonomyReason::LifeforceTooLow => "lifeforce.low",
            AssistantAutonomyReason::RiskTooHigh => "risk.high",
            AssistantAutonomyReason::IdentityDriftLimit => "identity.guard",
            AssistantAutonomyReason::TooManyActionsRecently => "rate.limited",
            AssistantAutonomyReason::HostConsentMissing => "consent.missing",
        }
    }
}
