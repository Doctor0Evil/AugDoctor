use serde::{Deserialize, Serialize};

use crate::lifeforce::{LifeforceBand, LifeforceBandSeries};
use crate::types::{EcoBandProfile, NanoDomain};

use neural_roping::pain_corridor::{PainCorridorSignal, SomaticRegionId};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RouterDecision {
    Safe,
    Defer,
    Deny,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RouterReasonCode {
    None,
    HardStop,
    EcoHigh,
    PainCorridor,
    ClarityLow,
    Unknown,
}

/// Minimal view of the current nanoswarm target.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoTargetContext {
    pub region_id: SomaticRegionId,
    pub domain: NanoDomain, // e.g. DetoxMicro, Radiology, RepairMicro, SensorHousekeeping
}

/// Existing router, extended with pain veto.
pub struct NanoLifebandRouter;

impl NanoLifebandRouter {
    #[allow(clippy::too_many_arguments)]
    pub fn classify(
        lifeforce_series: &LifeforceBandSeries,
        eco_profile: &EcoBandProfile,
        clarity_index: f32,
        nano_load_fraction: f32,
        target: &NanoTargetContext,
        pain_signal: Option<&PainCorridorSignal>,
    ) -> (RouterDecision, RouterReasonCode) {
        let lifeband = lifeforce_series.current_band();

        // 1. Classic HardStop invariant: physiology veto.
        if matches!(lifeband, LifeforceBand::HardStop) {
            return (RouterDecision::Deny, RouterReasonCode::HardStop);
        }

        // 2. Subjective veto: sustained pain in the target somatic region.
        if let Some(pain) = pain_signal {
            if pain.is_sustained_hardstop() && pain.region_id == target.region_id {
                // For pain-relevant domains we *must* deny.
                match target.domain {
                    NanoDomain::DetoxMicro
                    | NanoDomain::Radiology
                    | NanoDomain::RepairMicro => {
                        return (RouterDecision::Deny, RouterReasonCode::PainCorridor);
                    }
                    // For non-invasive domains we could Defer instead of Deny;
                    // here we still choose Deny to keep early experiments maximally safe.
                    _ => {
                        return (RouterDecision::Deny, RouterReasonCode::PainCorridor);
                    }
                }
            }
        }

        // 3. Eco band & clarity checks as in existing design.
        let eco_band = eco_profile.band;
        if matches!(eco_band, crate::types::EcoBand::High) && nano_load_fraction > 0.5 {
            return (RouterDecision::Defer, RouterReasonCode::EcoHigh);
        }

        if clarity_index < 0.3 {
            return (RouterDecision::Defer, RouterReasonCode::ClarityLow);
        }

        (RouterDecision::Safe, RouterReasonCode::None)
    }
}
