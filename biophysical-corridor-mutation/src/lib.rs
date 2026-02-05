//! Corridor-checked mutation layer for biophysical-blockchain.
//!
//! Forces MORPH / POWER / ALN / consent / SCALE / daily-turn gates
//! before any inner-ledger mutation, while keeping everything
//! per-host, non-financial, and sealed.[file:42][file:47]

mod sealed;
pub mod gates;
pub mod corridor;

pub use crate::corridor::{CorridorCheckedMutation, CorridorMutationError};
pub use crate::gates::{CorridorContext, CorridorError, CorridorGate, CorridorProfile};
