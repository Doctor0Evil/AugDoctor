pub struct KernelBounds {
    pub a_mode: [[f32; 7]; 7], // Amode
    pub b_mode: [f32; 7],      // bmode
}

pub fn state_in_kernel(bounds: &KernelBounds, x: [f32; 7]) -> bool {
    for i in 0..7 {
        let mut acc = 0.0;
        for j in 0..7 {
            acc += bounds.a_mode[i][j] * x[j];
        }
        if acc > bounds.b_mode[i] {
            return false;
        }
    }
    true
}
