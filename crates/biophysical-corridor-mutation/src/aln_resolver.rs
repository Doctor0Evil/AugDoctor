//! Load evolutionturnpolicy.aln + deep-domain-rights.aln and
//! derive a CorridorProfile for the current call.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::gates::CorridorProfile;

/// Minimal view of evolutionturnpolicy.aln we care about.
#[derive(Clone, Debug, Deserialize)]
pub struct EvolutionTurnPolicy {
    pub schema: String,
    pub maxdailyturns: u32,
    pub maxdailypainindex: f32,
    pub maxdailyfearindex: f32,
    pub maxdailyblooddelta: f32,
    pub microspaceprofileid: String,
    pub allowexperimentalreversible: bool,
    pub allowexperimentalirreversible: bool,
}

/// Minimal view of deep-domain-rights.aln we care about.
#[derive(Clone, Debug, Deserialize)]
pub struct DeepDomainRightsShard {
    pub host_id: String,
    pub profile_id: String,
    pub b2_max_epochs_per_day: u32,
    pub b3_max_epochs_per_day: u32,
    pub b4_max_epochs_per_day: u32,
    pub brain_tokens_daily_budget: f32,
    pub dracula_wave_daily_budget: f32,
    pub eco_nj_daily_budget: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CorridorProfileBundle {
    pub profile: CorridorProfile,
    pub source_evo_policy_id: String,
    pub source_deep_rights_id: String,
}

/// Simple ALN-as-JSON loader (assumes build step already rendered ALN to JSON).
fn load_json_from<P, T>(path: P) -> Result<T, String>
where
    P: AsRef<Path>,
    T: serde::de::DeserializeOwned,
{
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("read failed {}: {}", path.as_ref().display(), e))?;
    serde_json::from_str(&text)
        .map_err(|e| format!("parse failed {}: {}", path.as_ref().display(), e))
}

/// Derive MORPH/POWER/ALN limits from your doctrine.
/// - `morph_limit` comes from daily evolution turn envelope.
/// - `power_limit` comes from deep-domain eco/brain budgets.
/// - `aln_profile_id` is the evolution profile name to be enforced.
pub fn derive_corridor_profile_from_doctrine(
    evo_policy_json: &str,
    deep_rights_json: &str,
) -> Result<CorridorProfileBundle, String> {
    let evo: EvolutionTurnPolicy =
        serde_json::from_str(evo_policy_json).map_err(|e| format!("evo parse: {e}"))?;
    let deep: DeepDomainRightsShard =
        serde_json::from_str(deep_rights_json).map_err(|e| format!("deep parse: {e}"))?;

    // MORPH: normalized mutation depth based on how much of daily evolution budget
    // a single inner-ledger call is allowed to consume.
    // Example: one turn is at most 1 / maxdailyturns of morph scope.
    let morph_limit = if evo.maxdailyturns == 0 {
        0.0
    } else {
        (1.0 / evo.maxdailyturns as f32).min(1.0)
    };

    // POWER: eco + BRAIN/DraculaWave envelope.
    // Use a conservative normalization against daily budgets.
    let total_capacity = deep.eco_nj_daily_budget
        + deep.brain_tokens_daily_budget * 10.0
        + deep.dracula_wave_daily_budget * 10.0;

    let power_limit = if total_capacity <= 0.0 {
        0.0
    } else {
        // Allow a single mutation call to consume at most 5% of daily power.
        (0.05_f32).min(1.0)
    };

    let profile = CorridorProfile {
        morph_limit,
        power_limit,
        aln_profile_id: evo.schema.clone(), // or microspaceprofileid if you prefer
    };

    Ok(CorridorProfileBundle {
        profile,
        source_evo_policy_id: evo.schema,
        source_deep_rights_id: deep.profile_id,
    })
}
