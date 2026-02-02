#[derive(Clone, Debug)]
pub struct SpectralWeights {
    pub w_nm: f32,
    pub w_cog: f32,
    pub w_int: f32,
}

#[derive(Clone, Debug)]
pub struct FearBaselines {
    pub neuromod_amp_base: f32,
    pub cog_load_base: f32,
    pub intensity_base: f32,
}

#[derive(Clone, Debug)]
pub struct Host7DState {
    pub intensity: f32,
    pub duty_cycle: f32,
    pub cumulative_load: f32,
    pub implant_power: f32,
    pub neuromod_amp: f32,
    pub cog_load: f32,
    pub legal_complexity: f32,
}

pub fn spectral_energy_scalar(
    x: &Host7DState,
    base: &FearBaselines,
    w: &SpectralWeights,
) -> f32 {
    let d_nm  = (x.neuromod_amp - base.neuromod_amp_base).max(0.0);
    let d_cog = (x.cog_load      - base.cog_load_base).max(0.0);
    let d_int = (x.intensity     - base.intensity_base).max(0.0);

    w.w_nm  * d_nm  * d_nm +
    w.w_cog * d_cog * d_cog +
    w.w_int * d_int * d_int
}
