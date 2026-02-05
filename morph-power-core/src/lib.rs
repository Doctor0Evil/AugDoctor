pub mod types;
pub mod invariants;
pub mod power_guard;

pub use types::{
    MorphVector, MorphDimension, MorphUsage, MorphEvidenceTag, MorphEvidenceBundle,
    PowerBudget, PowerContext, PowerDecision, PowerProhibitionReason,
};
pub use invariants::{MorphBudgetCorridor, MorphSafetyError};
pub use power_guard::{PowerGovernor, PowerGuardConfig};
