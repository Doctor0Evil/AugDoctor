#[derive(Clone, Debug)]
pub struct CyberRank {
    pub safety: f32,
    pub legal: f32,
    pub biomech: f32,
    pub psych: f32,
    pub rollback: f32,
}

#[derive(Clone, Debug)]
pub struct CyberRankBands {
    pub min_safety: f32,
    pub min_legal: f32,
    pub min_biomech: f32,
    pub min_psych: f32,
    pub min_rollback: f32,
}

pub fn cyberrank_within_bands(rank: &CyberRank, bands: &CyberRankBands) -> bool {
    rank.safety   >= bands.min_safety   &&
    rank.legal    >= bands.min_legal    &&
    rank.biomech  >= bands.min_biomech  &&
    rank.psych    >= bands.min_psych    &&
    rank.rollback >= bands.min_rollback
}
