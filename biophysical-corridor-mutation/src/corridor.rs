//! Corridor-checked mutation trait that wraps the inner sealed LedgerMutator.

use crate::gates::{CorridorContext, CorridorError, CorridorGate};
use crate::sealed::inner::Sealed;
use aln_did_access::IdentityHeader;
use biophysical_blockchain::mutation::LedgerMutator;
use biophysical_blockchain::types::{
    EcoBandProfile, LifeforceBandSeries, SafetyCurveWave, SystemAdjustment,
};
use biophysical_blockchain::{InnerLedger, InnerLedgerError};
use consent_governance::ConsentVerifier;
use biophysical_blockchain::consensus::LedgerEvent;

/// Public trait exposed to orchestrators: they can *use* this
/// but cannot implement it for new types.
pub trait CorridorCheckedMutation: Sealed {
    /// Apply a SystemAdjustment only if the corridor gates pass, then
    /// delegate to the sealed inner-ledger mutation.
    fn apply_corridor_checked(
        &mut self,
        corridor: &CorridorContext,
        consent_verifier: &dyn ConsentVerifier,
        required_knowledge_factor: f32,
        adjustment: SystemAdjustment,
        timestamputc: &str,
        lifeforce_series: LifeforceBandSeries,
        eco_profile: EcoBandProfile,
        wave_curve: SafetyCurveWave,
    ) -> Result<LedgerEvent, CorridorMutationError>;
}

/// Unified error type for corridor + inner-ledger.
#[derive(Debug)]
pub enum CorridorMutationError {
    Corridor(CorridorError),
    Inner(InnerLedgerError),
}

impl From<CorridorError> for CorridorMutationError {
    fn from(e: CorridorError) -> Self {
        CorridorMutationError::Corridor(e)
    }
}

impl From<InnerLedgerError> for CorridorMutationError {
    fn from(e: InnerLedgerError) -> Self {
        CorridorMutationError::Inner(e)
    }
}

/// The only concrete implementation: InnerLedger.
/// No other type can be a corridor-checked mutator.
impl Sealed for InnerLedger {}

impl CorridorCheckedMutation for InnerLedger {
    fn apply_corridor_checked(
        &mut self,
        corridor: &CorridorContext,
        consent_verifier: &dyn ConsentVerifier,
        required_knowledge_factor: f32,
        adjustment: SystemAdjustment,
        timestamputc: &str,
        lifeforce_series: LifeforceBandSeries,
        eco_profile: EcoBandProfile,
        wave_curve: SafetyCurveWave,
    ) -> Result<LedgerEvent, CorridorMutationError> {
        // 1. MORPH / POWER / ALN / consent gates.
        corridor.check(consent_verifier).map_err(CorridorMutationError::from)?;

        // 2. Inner-ledger identity + lifeforce/eco/WAVE invariants via sealed LedgerMutator.
        let id_header: IdentityHeader = corridor.identity.clone();
        let event = <InnerLedger as LedgerMutator>::system_apply_guarded(
            self,
            id_header,
            required_knowledge_factor,
            adjustment,
            timestamputc,
            lifeforce_series,
            eco_profile,
            wave_curve,
        )?;

        Ok(event)
    }
}
