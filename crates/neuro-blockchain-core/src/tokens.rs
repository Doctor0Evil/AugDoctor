#[derive(Clone, Debug)]
pub enum TokenType {
    Brain,
    Evolve,
    Smart,
    Wave,
    Instinct,
}

#[derive(Clone, Debug)]
pub struct BioTokenState {
    pub host_did: String,
    pub token: TokenType,
    pub balance: f64,
}

pub trait InnerLedger {
    fn apply_system_adjustment(&mut self, adj: SystemAdjustment) -> Result<(), LedgerError>;
}

// No public API for send/transfer/stake; SystemAdjustment only mutates host-local state.
pub struct SystemAdjustment {
    pub delta_brain: f64,
    pub delta_wave: f64,
    pub delta_smart: f64,
    pub delta_evolve: f64,
    pub delta_instinct: f64,
    pub delta_scale: f64,
    pub ecoimpact_delta: f64,
}
