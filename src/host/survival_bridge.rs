use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Coarse origin of the detected threat.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThreatSource {
    Physiological,
    Environmental,
    Social,
}

/// Mirror of your INSTINCT guard plane, with an added SURVIVAL band.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstinctState {
    Safe,
    Defer,
    Block,
    Survival,
}

/// Snapshot of host physiology from outer-domain sensors.
/// All values are normalized 0.0..1.0 using host-local baselines.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysioSnapshot {
    pub heart_rate_norm: f32,   // 0 = resting, 1 = critical
    pub hrv_strain: f32,        // 0 = ideal variability, 1 = dangerously low
    pub bci_index: f32,         // composite BCI index, 0..1
    pub distress: f32,          // subjective band if available, 0 calm, 1 severe
}

/// Snapshot of outer-domain risk (no mental-state inference).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub crowd_risk: f32,        // 0..1 (e.g., density, exit blockage)
    pub impact_risk: f32,       // 0..1 (vehicles, falling objects)
    pub policing_intensity: f32,// 0..1 (number of armed actors, loud commands)
    pub weapon_noise_score: f32,// 0..1 (gunfire-like acoustic patterns)
}

/// Minimal survival policy parameters, host-tunable and non-financial.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurvivalPolicy {
    /// Hard BCI ceiling at or below the global 0.30 limit.
    pub bci_hard_ceiling: f32,
    /// Physiological alarm threshold for HR and HRV strain.
    pub hr_alarm: f32,
    pub hrv_alarm: f32,
    /// Social/environmental thresholds.
    pub policing_alarm: f32,
    pub weapon_noise_alarm: f32,
    /// Require INSTINCT to be in SURVIVAL or BLOCK to fire bridge.
    pub require_instinct_survival: bool,
}

/// Severity of the survival alert.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

/// Host-visible survival event, suitable for inner-ledger logging.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurvivalEvent {
    pub severity: AlertSeverity,
    pub threat_source: ThreatSource,
    pub timestamp_ms_utc: u64,
    /// Coarse, rights-safe English message for humans and AI-chats.
    pub message: String,
}

/// Alert for other augmented citizens over host-node / network.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostAlert {
    pub severity: AlertSeverity,
    /// Very coarse location label (corridor, block, beacon id) without PII.
    pub location_label: String,
    /// High-level context e.g., "crowd-crush-risk", "weapon-noise", "physio-collapse".
    pub context_label: String,
}

/// Species-safe cue pattern for trained animals (e.g., canines).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimalCue {
    /// Pattern identifier ("dog_help_me", "dog_avoid_person", etc.).
    pub pattern_id: String,
    /// Duration in milliseconds.
    pub duration_ms: u32,
    /// Relative intensity 0.0..1.0 for sound/light/haptics.
    pub intensity: f32,
}

/// Combined output of the Survival Bridge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurvivalBridgeOutput {
    pub event: SurvivalEvent,
    pub host_alert: HostAlert,
    pub animal_cue: Option<AnimalCue>,
}

/// Core evaluation function: decides if the bridge should fire and
/// constructs human and animal-safe outputs. Pure, testable logic.
pub fn evaluate_survival_bridge(
    policy: &SurvivalPolicy,
    instinct: InstinctState,
    physio: &PhysioSnapshot,
    env: &EnvironmentSnapshot,
    location_label: &str,
) -> Option<SurvivalBridgeOutput> {
    // Clamp inputs to 0.0..1.0 for safety.
    let bci = physio.bci_index.clamp(0.0, 1.0);
    let hr = physio.heart_rate_norm.clamp(0.0, 1.0);
    let hrv = physio.hrv_strain.clamp(0.0, 1.0);
    let policing = env.policing_intensity.clamp(0.0, 1.0);
    let weapon = env.weapon_noise_score.clamp(0.0, 1.0);

    // Hard physiological danger.
    let phys_danger =
        bci >= policy.bci_hard_ceiling ||
        (hr >= policy.hr_alarm && hrv >= policy.hrv_alarm);

    // Environmental / social danger.
    let env_danger =
        env.crowd_risk >= 0.8 ||
        env.impact_risk >= 0.8 ||
        policing >= policy.policing_alarm ||
        weapon >= policy.weapon_noise_alarm;

    if !phys_danger && !env_danger {
        return None;
    }

    if policy.require_instinct_survival {
        match instinct {
            InstinctState::Survival | InstinctState::Block => {}
            _ => return None,
        }
    }

    // Decide severity and threat source.
    let severity = if phys_danger && env_danger {
        AlertSeverity::Critical
    } else {
        AlertSeverity::Warning
    };

    let threat_source = if phys_danger && !env_danger {
        ThreatSource::Physiological
    } else if env_danger && !phys_danger {
        ThreatSource::Environmental
    } else {
        ThreatSource::Social
    };

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let message = match (&severity, &threat_source) {
        (AlertSeverity::Critical, ThreatSource::Physiological) =>
            "Critical physiological distress detected; survival support requested.".to_string(),
        (AlertSeverity::Critical, ThreatSource::Environmental) =>
            "Critical environmental danger detected; survival support requested.".to_string(),
        (AlertSeverity::Critical, ThreatSource::Social) =>
            "Critical combined physiological and social danger detected; survival support requested.".to_string(),
        (AlertSeverity::Warning, ThreatSource::Physiological) =>
            "Elevated physiological stress; monitoring and assistance recommended.".to_string(),
        (AlertSeverity::Warning, ThreatSource::Environmental) =>
            "Elevated environmental risk; monitoring and assistance recommended.".to_string(),
        (AlertSeverity::Warning, ThreatSource::Social) =>
            "Elevated combined risk; monitoring and assistance recommended.".to_string(),
    };

    let context_label = if weapon >= policy.weapon_noise_alarm {
        "weapon-noise"
    } else if env.crowd_risk >= 0.8 {
        "crowd-crush-risk"
    } else if policing >= policy.policing_alarm {
        "policing-intensity"
    } else if phys_danger {
        "physio-collapse"
    } else {
        "unspecified-danger"
    }.to_string();

    let event = SurvivalEvent {
        severity: severity.clone(),
        threat_source,
        timestamp_ms_utc: now_ms,
        message,
    };

    let host_alert = HostAlert {
        severity: severity.clone(),
        location_label: location_label.to_string(),
        context_label,
    };

    // Optional animal cue: only for certain patterns, and always friendly.
    let animal_cue = match severity {
        AlertSeverity::Critical => Some(AnimalCue {
            pattern_id: "dog_help_me".to_string(),
            duration_ms: 3000,
            intensity: 0.7,
        }),
        AlertSeverity::Warning => Some(AnimalCue {
            pattern_id: "dog_stay_close".to_string(),
            duration_ms: 1500,
            intensity: 0.4,
        }),
    };

    Some(SurvivalBridgeOutput {
        event,
        host_alert,
        animal_cue,
    })
}
