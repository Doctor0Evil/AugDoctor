use crate::morph::{MorphDelta};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeDescriptor {
    pub id: String,
    pub version: String,
    pub category: String, // neuromorphic, eco-upgrade, cybernetic, smart, ...

    // existing fields: energy, protein, duty, thermal, evidence, etc.

    pub required_morph: MorphBudget, // minimum MORPH vector required
    pub delta_morph:    MorphDelta,  // change to MORPH if applied
}
