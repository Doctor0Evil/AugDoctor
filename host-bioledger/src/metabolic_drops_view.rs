use serde::{Deserialize, Serialize};
use biophysical_blockchain::types::{BioTokenState, LifeforceBand};

/// Read-only, UI/neuromorph-friendly projection of BLOOD into 0–100 "drops".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetabolicDropsView {
    pub drops_0_100: u8,
    pub band: LifeforceBand,
    pub blood_raw: f64,
}

impl MetabolicDropsView {
    /// Map BLOOD in [blood_min, blood_max] into 0–100 drops.
    /// Values below blood_min clamp to 0; above blood_max clamp to 100.
    pub fn from_state(
        state: &BioTokenState,
        blood_min: f64,
        blood_max: f64,
        band: LifeforceBand,
    ) -> Self {
        let blood = state.blood;
        let span = (blood_max - blood_min).max(1e-9);
        let norm = ((blood - blood_min) / span).clamp(0.0, 1.0);
        let drops = (norm * 100.0).round() as u8;

        MetabolicDropsView {
            drops_0_100: drops,
            band,
            blood_raw: blood,
        }
    }
}
