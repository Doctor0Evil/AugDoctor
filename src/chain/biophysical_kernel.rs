use quantum_hash::*;
use aln_did::*;
use lifeforce_safety::*;

#[derive(Clone)]
pub struct BioTokenState {
    pub brain: f64,
    pub wave: f64,
    pub blood: f64,
    pub oxygen: f64,
    pub nano: f64,
    pub smart: f64,
    pub host_id: ALNDID,
}

impl BioTokenState {
    pub fn enforce_invariants(&mut self) -> Result<(), &'static str> {
        if self.brain < 0.0 { return Err("Forbidden: negative BRAIN -> death condition."); }
        if self.blood <= 0.0 || self.oxygen <= 0.0 { return Err("Unsafe depletion: consciousness inactive."); }
        if self.smart > self.brain { self.smart = self.brain; } // SMART <= BRAIN safety binding
        Ok(())
    }

    pub fn system_adjust(&mut self, delta_brain: f64, delta_wave: f64) {
        self.brain += delta_brain;
        self.wave  += delta_wave.min(self.brain / 2.0); // restrict wave by current compute health
        self.enforce_invariants().unwrap();
    }
}

pub fn consensus_attest(state: &BioTokenState) -> QuantumHash {
    QuantumHash::digest(&format!("{:?}{:?}", state.host_id, state))
}
