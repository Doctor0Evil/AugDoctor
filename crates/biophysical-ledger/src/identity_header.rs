use sovereign_id::NeuroSubjectId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityHeader {
    pub subject_id: NeuroSubjectId,
    pub role: String,          // "AugmentedCitizen", "SystemDaemon", etc.
    pub knowledge_factor: f32, // 0.0 - 1.0, for risk gating
    pub sig: Vec<u8>,          // signature of (subject_id, role, kf, nonce)
}
