pub struct RoHState {
    pub last_value: f32,
    pub threshold: f32, // e.g. 0.3
    pub non_increasing: bool,
}

pub enum RoHUpdateResult {
    Ok { new_value: f32 },
    ThresholdExceeded,
    IncreasedWhenPinned,
}

pub fn update_roh(state: &mut RoHState, proposed: f32) -> RoHUpdateResult {
    if proposed > state.threshold {
        return RoHUpdateResult::ThresholdExceeded;
    }
    if state.non_increasing && proposed > state.last_value {
        return RoHUpdateResult::IncreasedWhenPinned;
    }
    state.last_value = proposed;
    RoHUpdateResult::Ok { new_value: proposed }
}
