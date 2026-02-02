pub enum FearTransitionKind {
    Enter,
    Exit,
    AbortByRollback,
}

#[derive(Clone, Debug)]
pub struct FearTransitionEvent {
    pub host_id: String,
    pub session_id: String,
    pub kind: FearTransitionKind,
    pub at_utc: i64,
    pub initiated_by_host: bool,
}

