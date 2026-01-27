const ShotLevel = {
  ZeroShot: "ZeroShot",
  FewShot: "FewShot",
};

/**
 * Decide shot level on the JS side (mirrors Rust ShotLevelPolicy).
 */
function decideShotLevel(signal, cfg) {
  let useFewShot = false;
  const reasons = [];

  if (signal.requiresExamples) {
    useFewShot = true;
    reasons.push("task explicitly requires examples");
  }
  if (signal.riskScore >= cfg.riskThresholdForFewShot) {
    useFewShot = true;
    reasons.push(
      `risk_score ${signal.riskScore.toFixed(3)} >= threshold ${cfg.riskThresholdForFewShot.toFixed(3)}`
    );
  }
  if (signal.historicalErrorRate >= cfg.errorRateThresholdForFewShot) {
    useFewShot = true;
    reasons.push(
      `historical_error_rate ${signal.historicalErrorRate.toFixed(3)} >= threshold ${cfg.errorRateThresholdForFewShot.toFixed(3)}`
    );
  }
  if (signal.latencyBudgetMs < cfg.minLatencyForFewShotMs) {
    useFewShot = false;
    reasons.push(
      `latency_budget_ms ${signal.latencyBudgetMs} < minLatencyForFewShotMs ${cfg.minLatencyForFewShotMs}`
    );
  }
  if (signal.tokenBudget < cfg.minTokenBudgetForFewShot) {
    useFewShot = false;
    reasons.push(
      `token_budget ${signal.tokenBudget} < minTokenBudgetForFewShot ${cfg.minTokenBudgetForFewShot}`
    );
  }

  const chosenLevel = useFewShot ? ShotLevel.FewShot : ShotLevel.ZeroShot;
  const maxExamples = chosenLevel === ShotLevel.ZeroShot ? 0 : cfg.maxExamplesFewShot;

  return {
    chosenLevel,
    maxExamples,
    explanation: reasons.join("; "),
  };
}

/**
 * Build a final prompt string for AI-Chats using neural-rope examples.
 */
function buildPrompt(baseInstruction, selectorResult) {
  if (
    selectorResult.shot_level === "ZeroShot" ||
    !selectorResult.examples ||
    selectorResult.examples.length === 0
  ) {
    return baseInstruction;
  }

  const lines = [];
  lines.push(baseInstruction.trim());
  lines.push("");
  lines.push("Here are task-specific examples for higher accuracy:");
  selectorResult.examples.forEach((ex, idx) => {
    lines.push(`Example ${idx + 1}:`);
    lines.push(ex.text.trim());
    lines.push(`(reward=${ex.reward_score}, safety=${ex.safety_decision})`);
    lines.push("");
  });
  return lines.join("\n");
}

/**
 * Simple JS model of the NeuroHandshakeOrchestrator's phases, suitable for
 * front-end flows or multi-platform agent UIs.
 */
const HandshakePhase = {
  Safety: "Safety",
  Calibration: "Calibration",
  Operation: "Operation",
};

function nextHandshakeActions(state) {
  if (state.phase === HandshakePhase.Safety) {
    if (!state.safetyConfirmed) {
      return ["PromptUserConsent", "ShowSafetySummary"];
    }
    return ["CollectCalibrationSample:eeg"];
  }

  if (state.phase === HandshakePhase.Calibration) {
    if (state.calibrationSamplesCollected < state.requiredCalibrationSamples) {
      return ["CollectCalibrationSample:eeg", "CollectCalibrationSample:emg"];
    }
    return ["TransitionToOperation"];
  }

  return [];
}

module.exports = {
  ShotLevel,
  decideShotLevel,
  buildPrompt,
  HandshakePhase,
  nextHandshakeActions,
};
