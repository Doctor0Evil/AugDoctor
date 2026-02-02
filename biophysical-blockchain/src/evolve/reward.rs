pub fn evolution_points_for_session(k_delta: f32) -> u32 {
    if k_delta <= 0.0 {
        return 0;
    }
    // Small, bounded micro-step mapping; exact curve governed by ALN shards.
    (k_delta * 100.0).min(50.0) as u32
}
