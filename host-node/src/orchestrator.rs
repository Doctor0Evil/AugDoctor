use biophysical_corridor_mutation::{CorridorContext, CorridorProfile};
use aln_did_access::IdentityHeader;
use consent_governance::DemonstratedConsentShard;

fn build_corridor_context_from_doctrine(
    id: IdentityHeader,
    consent: DemonstratedConsentShard,
    derived: CorridorProfile,
    requested_morph: f32,
    requested_power: f32,
) -> CorridorContext {
    CorridorContext {
        identity: id,
        profile: derived,
        consent: Some(consent),
        requested_morph,
        requested_power,
    }
}
