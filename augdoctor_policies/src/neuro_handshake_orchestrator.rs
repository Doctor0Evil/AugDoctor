use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HandshakePhase {
    Safety,
    Calibration,
    Operation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuroHandshakeState {
    pub session_id: String,
    pub phase: HandshakePhase,
    pub safety_confirmed: bool,
    pub calibration_samples_collected: u32,
    pub required_calibration_samples: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NeuroHandshakeAction {
    PromptUserConsent,
    ShowSafetySummary,
    CollectCalibrationSample { channel: String },
    TransitionToOperation,
    DenyOperation { reason: String },
}

pub struct NeuroHandshakeOrchestrator;

impl NeuroHandshakeOrchestrator {
    pub fn initial(session_id: &str, required_calibration_samples: u32) -> NeuroHandshakeState {
        NeuroHandshakeState {
            session_id: session_id.to_string(),
            phase: HandshakePhase::Safety,
            safety_confirmed: false,
            calibration_samples_collected: 0,
            required_calibration_samples,
        }
    }

    pub fn next_actions(state: &NeuroHandshakeState) -> Vec<NeuroHandshakeAction> {
        match state.phase {
            HandshakePhase::Safety => {
                if !state.safety_confirmed {
                    vec![
                        NeuroHandshakeAction::PromptUserConsent,
                        NeuroHandshakeAction::ShowSafetySummary,
                    ]
                } else {
                    vec![NeuroHandshakeAction::CollectCalibrationSample {
                        channel: String::from("eeg"),
                    }]
                }
            }
            HandshakePhase::Calibration => {
                if state.calibration_samples_collected < state.required_calibration_samples {
                    vec![
                        NeuroHandshakeAction::CollectCalibrationSample {
                            channel: String::from("eeg"),
                        },
                        NeuroHandshakeAction::CollectCalibrationSample {
                            channel: String::from("emg"),
                        },
                    ]
                } else {
                    vec![NeuroHandshakeAction::TransitionToOperation]
                }
            }
            HandshakePhase::Operation => Vec::new(),
        }
    }

    pub fn apply_event(
        mut state: NeuroHandshakeState,
        event: &str,
    ) -> NeuroHandshakeState {
        match event {
            "user_consented" => {
                state.safety_confirmed = true;
                state.phase = HandshakePhase::Calibration;
            }
            "calibration_sample_recorded" => {
                if state.phase == HandshakePhase::Calibration {
                    state.calibration_samples_collected += 1;
                }
            }
            "force_deny" => {
                state.phase = HandshakePhase::Safety;
                state.safety_confirmed = false;
                state.calibration_samples_collected = 0;
            }
            _ => {}
        }
        state
    }
}
