pub struct FearConsentState {
    pub last_affirmation_utc: i64,
    pub max_silence_secs: i64,
}

pub fn consent_still_valid(now_utc: i64, state: &FearConsentState) -> bool {
    (now_utc - state.last_affirmation_utc) <= state.max_silence_secs
}
