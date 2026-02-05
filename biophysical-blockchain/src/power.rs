#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::sealed::inner::Sealed;
use crate::types::{HostEnvelope, LifeforceBand, EcoBandProfile};

/// Per-turn, host-bound governor on agentic AI surface.
/// Non-financial, non-transferable, propose-only.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerCorridor {
    pub host_id: String,
    /// Max documents RAG may read this turn.
    pub max_documents: u32,
    /// Max external calls (HTTP/RPC) this turn.
    pub max_external_calls: u32,
    /// Max number of distinct policy shards AI may *touch* (read/propose).
    pub max_policy_surface: u32,
    /// Max novelty score corridor (0–1) for proposals this turn.
    pub max_novelty: f32,
    /// Flag: irreversible/civic actions allowed at all in this turn.
    pub irreversible_allowed: bool,
    /// Remaining counters during validation.
    pub remaining_documents: u32,
    pub remaining_external_calls: u32,
    pub remaining_policy_surface: u32,
}

impl Sealed for PowerCorridor {}

impl PowerCorridor {
    /// Construct a corridor from host env, lifeforce, and eco band.
    /// Tightens when eco or lifeforce are stressed.
    pub fn from_state(
        host: &HostEnvelope,
        lifeforce_band: LifeforceBand,
        eco: &EcoBandProfile,
    ) -> Self {
        // Base capacities derived from eco + lifeforce, never from money.
        let (base_docs, base_calls, base_policy, base_novelty) = match lifeforce_band {
            LifeforceBand::Safe => (64, 32, 16, 1.0),
            LifeforceBand::SoftWarn => (32, 16, 8, 0.7),
            LifeforceBand::HardStop => (4, 2, 2, 0.3),
        };

        // Eco tightening: High eco band shrinks corridor, Low relaxes slightly.
        let eco_factor: f32 = match eco.ecoband {
            crate::types::EcoBand::Low => 1.0,
            crate::types::EcoBand::Medium => 0.75,
            crate::types::EcoBand::High => 0.5,
        };

        let docs = ((base_docs as f32) * eco_factor).floor() as u32;
        let calls = ((base_calls as f32) * eco_factor).floor() as u32;
        let policy = ((base_policy as f32) * eco_factor).floor() as u32;
        let novelty = base_novelty * eco_factor;

        // Irreversible only in Safe band, and eco not High.
        let irreversible_allowed =
            matches!(lifeforce_band, LifeforceBand::Safe) && eco.ecoband != crate::types::EcoBand::High;

        PowerCorridor {
            host_id: host.hostid.clone(),
            max_documents: docs,
            max_external_calls: calls,
            max_policy_surface: policy,
            max_novelty: novelty.clamp(0.0, 1.0),
            irreversible_allowed,
            remaining_documents: docs,
            remaining_external_calls: calls,
            remaining_policy_surface: policy,
        }
    }

    pub fn charge_document(&mut self) -> Result<(), PowerError> {
        if self.remaining_documents == 0 {
            return Err(PowerError::DocumentBudgetExhausted);
        }
        self.remaining_documents -= 1;
        Ok(())
    }

    pub fn charge_external_call(&mut self) -> Result<(), PowerError> {
        if self.remaining_external_calls == 0 {
            return Err(PowerError::ExternalCallBudgetExhausted);
        }
        self.remaining_external_calls -= 1;
        Ok(())
    }

    pub fn charge_policy_touch(&mut self) -> Result<(), PowerError> {
        if self.remaining_policy_surface == 0 {
            return Err(PowerError::PolicySurfaceBudgetExhausted);
        }
        self.remaining_policy_surface -= 1;
        Ok(())
    }

    /// Guard for irreversible / high-civic-risk actions.
    pub fn require_irreversible_capacity(
        &self,
        has_irreversible_token: bool,
        has_consent_bundle: bool,
    ) -> Result<(), PowerError> {
        if !self.irreversible_allowed {
            return Err(PowerError::IrreversibleForbiddenThisTurn);
        }
        if !has_irreversible_token || !has_consent_bundle {
            return Err(PowerError::MissingIrreversibleGuard);
        }
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PowerError {
    #[error("POWER: document budget exhausted for this turn")]
    DocumentBudgetExhausted,
    #[error("POWER: external-call budget exhausted for this turn")]
    ExternalCallBudgetExhausted,
    #[error("POWER: policy-surface budget exhausted for this turn")]
    PolicySurfaceBudgetExhausted,
    #[error("POWER: irreversible or civic downgrade actions are forbidden this turn")]
    IrreversibleForbiddenThisTurn,
    #[error("POWER: irreversible action missing token and consent bundle")]
    MissingIrreversibleGuard,
}
