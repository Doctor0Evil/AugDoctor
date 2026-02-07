use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::nano_router::{RouterDecision, RouterReasonCode};
use crate::types::{EcoBandProfile, LifeforceBand, NanoDomain};

use neural_roping::pain_corridor::SomaticRegionId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NanoRouteDecisionLog {
    pub decision_id: uuid::Uuid,
    pub host_id: String,
    pub ts_utc: DateTime<Utc>,
    pub packet_hash: String,
    pub router_decision: RouterDecision,
    pub reason_code: RouterReasonCode,
    pub nano_domain: NanoDomain,
    pub region_id: SomaticRegionId,
    pub current_lifeforce_band: LifeforceBand,
    pub current_eco_band: EcoBandProfile,
    pub civic_audit_ref: String,
}
