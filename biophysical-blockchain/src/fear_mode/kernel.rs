pub struct KernelBounds {
    pub a_mode: [[f32; 7]; 7],
    pub b_mode: [f32; 7],
    pub spectral_max: f32,
    pub spectral_weights: SpectralWeights,
    pub spectral_baselines: FearBaselines,
}

pub fn state_in_fear_kernel(bounds: &KernelBounds, x: &Host7DState) -> bool {
    // Linear viability constraints
    for i in 0..7 {
        let mut acc = 0.0;
        let vec = [
            x.intensity,
            x.duty_cycle,
            x.cumulative_load,
            x.implant_power,
            x.neuromod_amp,
            x.cog_load,
            x.legal_complexity,
        ];
        for j in 0..7 {
            acc += bounds.a_mode[i][j] * vec[j];
        }
        if acc > bounds.b_mode[i] {
            return false;
        }
    }

    // Spectral energy ceiling
    let e_spec = crate::fear_mode::spectral_energy::spectral_energy_scalar(
        x,
        &bounds.spectral_baselines,
        &bounds.spectral_weights,
    );
    e_spec <= bounds.spectral_max
}
