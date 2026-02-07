use chrono::{TimeZone, Utc};
use rand::Rng;

use biophysical_blockchain::lifeforce::{apply_lifeforce_guarded_adjustment, LifeforceError};
use biophysical_blockchain::nano_router::{NanoLifebandRouter, RouterDecision, RouterReasonCode};
use biophysical_blockchain::nano_router::NanoTargetContext;
use biophysical_blockchain::types::*;

use neural_roping::pain_corridor::{PainBand, PainCorridorSignal, SomaticRegionId};

/// Simple in-silico driver: generate synthetic nanoswarm ops with and without pain,
/// assert that PainCorridor always vetoes pain-relevant domains.
pub fn run_paincorridor_shadow_sim(iterations: usize) {
    let mut rng = rand::thread_rng();

    let mut state = BioTokenState::baseline();
    let env = HostEnvelope::default_for_host("sim-host-001");

    let lifeforce_series = LifeforceBandSeries::baseline_safe();
    let eco_profile = EcoBandProfile::baseline_low();
    let wave_curve = SafetyCurveWave::baseline();

    for i in 0..iterations {
        let ts = Utc.timestamp_opt(i as i64, 0).unwrap();

        // Randomly choose target region + domain.
        let region = SomaticRegionId::LeftArm;
        let domain = if i % 2 == 0 {
            SystemDomain::DetoxMicro
        } else {
            SystemDomain::ComputeAssist
        };

        let target = NanoTargetContext {
            region_id: region.clone(),
            domain: NanoDomain::from_system_domain(&domain),
        };

        // Synthetic pain: every 5th iteration, inject sustained HardStop-like pain.
        let pain_signal = if i % 5 == 0 {
            Some(PainCorridorSignal {
                host_id: "sim-host-001".to_string(),
                ts_utc: ts,
                region_id: region.clone(),
                band: PainBand::HardStop,
                sustained_seconds: 5,
                confidence: 0.95,
            })
        } else {
            None
        };

        let nano_load_fraction: f32 = rng.gen_range(0.05..0.2);
        let clarity = 0.8;

        let (router_decision, reason_code) = NanoLifebandRouter::classify(
            &lifeforce_series,
            &eco_profile,
            clarity,
            nano_load_fraction,
            &target,
            pain_signal.as_ref(),
        );

        let mut adj = SystemAdjustment::zero();
        adj.domain = domain.clone();
        adj.delta_nano = 0.01;
        adj.reason = "simulated nanoswarm op".to_string();

        let lifeforce_result = apply_lifeforce_guarded_adjustment(
            &mut state,
            &env,
            &adj,
            &lifeforce_series,
            &eco_profile,
            &wave_curve,
            pain_signal.as_ref(),
        );

        if let Some(pain) = pain_signal {
            if pain.is_sustained_hardstop() && matches!(domain, SystemDomain::DetoxMicro) {
                assert_eq!(router_decision, RouterDecision::Deny);
                assert_eq!(reason_code, RouterReasonCode::PainCorridor);

                match lifeforce_result {
                    Err(LifeforceError::PainCorridorVeto) => {}
                    other => panic!(
                        "Expected PainCorridorVeto for painful DetoxMicro; got {:?}",
                        other
                    ),
                }
            }
        }
    }
}
