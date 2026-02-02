#[derive(Clone, Debug)]
pub struct FearRollbackSnapshot {
    pub session_id: String,
    pub pre_cyberrank: CyberRank,
    pub pre_bands: LifeforceBandSeries,
}

pub fn capture_snapshot(session_id: &str, rank: &CyberRank, bands: &LifeforceBandSeries)
    -> FearRollbackSnapshot
{
    FearRollbackSnapshot {
        session_id: session_id.to_string(),
        pre_cyberrank: rank.clone(),
        pre_bands: bands.clone(),
    }
}
