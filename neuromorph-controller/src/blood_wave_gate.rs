use biophysical_blockchain::types::LifeforceBand;
use host_bioledger::metabolic_drops_view::MetabolicDropsView;

/// Policy output for a neuromorph chipset: how much instantaneous draw is allowed.
#[derive(Clone, Debug)]
pub struct NeuromorphBudget {
    /// 1.0 = full designed output, 0.0 = no extra load allowed.
    pub power_scale_0_1: f32,
}

pub fn neuromorph_budget_from_blood(view: &MetabolicDropsView) -> NeuromorphBudget {
    let d = view.drops_0_100 as f32;

    let scale = match view.band {
        LifeforceBand::HardStop => 0.0,
        LifeforceBand::SoftWarn => {
            if d < 40.0 {
                0.0
            } else if d < 70.0 {
                0.4
            } else {
                0.7
            }
        }
        LifeforceBand::Safe => {
            if d < 40.0 {
                0.2
            } else if d < 70.0 {
                0.6
            } else {
                1.0
            }
        }
    };

    NeuromorphBudget {
        power_scale_0_1: scale,
    }
}
