use crate::domain::{NeuromorphPlane, NeuromorphDomain, NeuromorphPlaneTag};

#[derive(Clone, Debug)]
pub enum NeuromorphEventKind {
    EmgOnset { channel: String, strength: f32 },
    EmgOffset { channel: String },
    MotionBurst { joint: String, magnitude: f32 },
    PostureShift { pattern_id: String },
    BciState { label: String, confidence: f32 },
    SystemOverload { error_rate: f32 },
    SystemRecovery { pattern_id: String },
}

#[derive(Clone, Debug)]
pub struct NeuromorphEvent {
    pub plane: NeuromorphPlaneTag,        // always NeuromorphReflex
    pub timestamp_ms_utc: i64,
    pub kind: NeuromorphEventKind,
    pub source_session: String,           // host session / environment id
    pub riskscore: f32,                   // copied from BCI/EMG classifier
}

pub trait NeuromorphEventSink {
    fn record(&mut self, event: NeuromorphEvent);
}
