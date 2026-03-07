fn main() {
    // Simulated CozoDB particle DB from supplied export
    let mut db: HashMap<String, ParticleRecord> = HashMap::new();
    db.insert("QmdmqT6YALao6KBYgecYxrJEiBjfNQ8Gwe9tJ6J9keaAzY".to_string(), ParticleRecord { cid: "QmdmqT6YALao6KBYgecYxrJEiBjfNQ8Gwe9tJ6J9keaAzY".into(), mime: "text/plain".into(), text: "The Donut of Knowledge".into(), blocks: 666, size: 666, size_local: 666, r#type: "file".into() });

    let lifeband = NanoLifebandRouter { blood_oxygen: 0.48, brain_wave: 0.52, eco_flops: 0.65 };
    let eeg_json = r#"{"issuer_did":"did:ion:...","subject_role":"augmented_citizen","biophysical_chain_allowed":true,"network_tier":"core"}"#;

    match process_reward_claim("cybernetic-evolution-reward-guard", "QmdmqT6YALao6KBYgecYxrJEiBjfNQ8Gwe9tJ6J9keaAzY", eeg_json, lifeband, db) {
        Ok((claim, path)) => println!("Reward routed: {:?} | EcoPoints: {} | KnowledgeFactor: {}", path, claim.eco_points, claim.knowledge_factor),
        Err(e) => println!("Blocked: {}", e),
    }
}
