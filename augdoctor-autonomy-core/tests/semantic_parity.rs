use augdoctor_autonomy_core::{AutonomyGovernor, AutonomyProfile, AutonomyTraceAttributes};

fn golden_vectors() -> Vec<(AutonomyProfile, AutonomyTraceAttributes, bool, f32, &'static str)> {
    vec![
        // calm, low eco, low risk → high autonomy, ZeroShot
        (
            AutonomyProfile {
                profile_id: "test".into(),
                host_id: "host".into(),
                max_eco_energy_nj_per_minute: 5.0,
                max_autonomous_actions_per_minute: 10,
                max_risk_score: 0.65,
                min_lifeforce_scalar: 0.35,
                max_identity_drift_per_day: 0.15,
            },
            AutonomyTraceAttributes {
                schemaversion: "autonomy-trace.v1".into(),
                host_id: "host".into(),
                session_id: "s1".into(),
                environment_id: "env".into(),
                plane: "chat.only".into(),
                stress: 0.2,
                fatigue: 0.1,
                reward: 0.5,
                safety: 0.9,
                lifeforce_scalar: 0.8,
                eco_energy_nj: 1.0,
                risk: 0.2,
                actions_last_minute: 1,
                identity_drift_today: 0.05,
                decision_autonomy_level: 0.0,
                decision_shot_level_label: "ZeroShot".into(),
                constraint: crate::trace::AutonomyConstraint {
                    highest_risk_score: 0.2,
                    worst_lifeforce_scalar: 0.8,
                },
            },
            true,
            0.7,
            "ZeroShot",
        ),
        // high risk → autonomy 0.0
        (
            AutonomyProfile {
                profile_id: "test".into(),
                host_id: "host".into(),
                max_eco_energy_nj_per_minute: 5.0,
                max_autonomous_actions_per_minute: 10,
                max_risk_score: 0.65,
                min_lifeforce_scalar: 0.35,
                max_identity_drift_per_day: 0.15,
            },
            AutonomyTraceAttributes {
                schemaversion: "autonomy-trace.v1".into(),
                host_id: "host".into(),
                session_id: "s2".into(),
                environment_id: "env".into(),
                plane: "bci.hci.eeg".into(),
                stress: 0.8,
                fatigue: 0.7,
                reward: -0.2,
                safety: 0.3,
                lifeforce_scalar: 0.6,
                eco_energy_nj: 1.0,
                risk: 0.9,
                actions_last_minute: 2,
                identity_drift_today: 0.05,
                decision_autonomy_level: 0.0,
                decision_shot_level_label: "ZeroShot".into(),
                constraint: crate::trace::AutonomyConstraint {
                    highest_risk_score: 0.9,
                    worst_lifeforce_scalar: 0.6,
                },
            },
            true,
            0.0,
            "ZeroShot",
        ),
    ]
}

#[test]
fn rust_reference_matches_golden_vectors() {
    for (profile, trace, consent, expected_level, expected_shot) in golden_vectors() {
        let d = AutonomyGovernor::decide(&profile, &trace, consent);
        assert!((d.autonomy_level - expected_level).abs() < 1e-6);
        assert_eq!(d.shot_level_label, expected_shot);
    }
}
