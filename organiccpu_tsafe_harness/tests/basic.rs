#[cfg(test)]
mod tests {
    use super::super::run::{run_once, HarnessConfig};
    use std::path::Path;

    #[test]
    fn test_safe_state() {
        let cfg = HarnessConfig {
            kernel_path: Path::new("kernels/example_viability_kernel.yaml"),
            fake_state: vec![0.6, 0.4, 0.5, 0.3, 0.4, 0.6, 0.5, 0.5],  // Inside bounds
        };
        assert!(run_once(cfg).unwrap());
    }

    #[test]
    fn test_unsafe_state() {
        let cfg = HarnessConfig {
            kernel_path: Path::new("kernels/example_viability_kernel.yaml"),
            fake_state: vec![0.8, 0.6, 0.7, 0.5, 0.6, 0.8, 0.7, 0.3],  // Outside bounds
        };
        assert!(!run_once(cfg).unwrap());
    }
}
