//! Corridor-checked mutation layer for biophysical-blockchain.
//! Forces MORPH/POWER/ALN gates before any inner-ledger mutation.

mod sealed;
pub mod gates;
pub mod corridor;

pub use crate::corridor::{CorridorCheckedMutation, CorridorMutationError};
pub use crate::gates::{CorridorContext, CorridorError, CorridorGate, CorridorProfile};
