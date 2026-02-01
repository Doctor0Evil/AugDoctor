use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ViabilityKernelSpec {
    pub state_dim: usize,
    pub A: Vec<Vec<f64>>,
    pub b: Vec<f64>,
    pub labels: Option<Vec<String>>,
    pub meta: Option<serde_yaml::Value>,
}

impl ViabilityKernelSpec {
    pub fn validate(&self) -> Result<(), String> {
        if self.A.is_empty() {
            return Err("A must have at least one row".to_string());
        }
        let n = self.state_dim;
        for (i, row) in self.A.iter().enumerate() {
            if row.len() != n {
                return Err(format!("row {} of A has length {}, expected {}", i, row.len(), n));
            }
        }
        if self.b.len() != self.A.len() {
            return Err(format!("b has length {}, expected {} (one per row of A)", self.b.len(), self.A.len()));
        }
        Ok(())
    }
}
