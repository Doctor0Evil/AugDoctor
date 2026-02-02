pub struct StakePolicy {
    pub host_did: String,
    pub evolve_signers: Vec<String>,   // e.g., [host, organiccpu]
    pub lifeforce_scopes: Vec<String>, // e.g., ["lifeforce_alteration"]
}

pub fn verify_multisig(
    policy: &StakePolicy,
    proposal: &EvolveProposal,
    signatures: &[Signature],
) -> bool {
    if !proposal.requires_multisig() {
        return true;
    }
    let required = &policy.evolve_signers;
    let mut seen = Vec::new();
    for sig in signatures {
        if required.contains(&sig.signer_did) && sig.valid_for(&proposal.hash) {
            seen.push(sig.signer_did.clone());
        }
    }
    seen.sort();
    seen.dedup();
    seen.len() == required.len()
}
