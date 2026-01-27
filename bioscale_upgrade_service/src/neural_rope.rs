use flo_rope::{AttributedRope, RopeSlice};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Attributes attached to segments in the neural rope. These are carefully
/// non-identity-bearing and do not encode consciousness, only learning context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralRopeAttributes {
    pub plane_label: String,
    pub bioscale_upgrade_id: Option<String>,
    pub reward_score: f32,
    pub safety_decision: String,
}

/// A neural rope containing token-like units (chars or short symbols) plus
/// attributes for assisted-learning introspection.
#[derive(Clone, Debug)]
pub struct NeuralRope {
    rope: AttributedRope<char, NeuralRopeAttributes>,
}

impl NeuralRope {
    pub fn new() -> Self {
        NeuralRope {
            rope: AttributedRope::from(String::new().chars().collect::<Vec<char>>()),
        }
    }

    /// Append a new trace chunk with attributes.
    pub fn append_trace(
        &mut self,
        trace: &str,
        plane_label: &str,
        bioscale_upgrade_id: Option<String>,
        reward_score: f32,
        safety_decision: &str,
    ) {
        let start_len = self.rope.len();
        let chars: Vec<char> = trace.chars().collect();
        self.rope.insert(start_len, chars.clone());
        let end_len = start_len + chars.len();

        let attrs = NeuralRopeAttributes {
            plane_label: plane_label.to_string(),
            bioscale_upgrade_id,
            reward_score,
            safety_decision: safety_decision.to_string(),
        };

        self.rope.set_attributes(start_len..end_len, attrs);
    }

    /// Export a safe, JSON-serializable snapshot of the rope for external
    /// visualization or assisted-learning analysis. This does not expose any
    /// identity-bearing or conscious-state fields by construction.
    pub fn export_snapshot(&self, max_segments: usize) -> Vec<NeuralRopeSegmentSnapshot> {
        let mut snapshots = Vec::new();
        let len = self.rope.len();
        let mut idx = 0;
        let mut count = 0;

        while idx < len && count < max_segments {
            let next = (idx + 256).min(len);
            let slice: RopeSlice<char> = self.rope.read_cells(idx..next);
            let text: String = slice.cloned().collect();

            let attrs = self
                .rope
                .attributes(idx..next)
                .first()
                .map(|(_, a)| a.clone());

            snapshots.push(NeuralRopeSegmentSnapshot {
                id: Uuid::new_v4().to_string(),
                range_start: idx as u64,
                range_end: next as u64,
                text,
                attributes: attrs,
            });

            idx = next;
            count += 1;
        }

        snapshots
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralRopeSegmentSnapshot {
    pub id: String,
    pub range_start: u64,
    pub range_end: u64,
    pub text: String,
    pub attributes: Option<NeuralRopeAttributes>,
}
