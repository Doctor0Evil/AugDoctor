const { ShotLevel, decideShotLevel, buildPrompt } = require("./augdoctor_policies");

/**
 * Convert UI/agent context into a shot-level signal.
 */
function makeShotSignal(intentLabel, riskScore, latencyBudgetMs, tokenBudget) {
  return {
    taskId: `eeg-${intentLabel}`,
    planeLabel: "bci/hci/eeg",
    riskScore,
    latencyBudgetMs,
    tokenBudget,
    historicalErrorRate: 0.05,
    requiresExamples: intentLabel === "fine_grip",
  };
}

/**
 * Build a few-shot prompt using EEG examples fetched from a WASM-backed
 * neural-rope snapshot.
 *
 * @param {string} intentLabel
 * @param {number} riskScore
 * @param {number} latencyBudgetMs
 * @param {number} tokenBudget
 * @param {Array<{ text: string, reward_score: number, safety_decision: string, plane_label: string }>} examples
 */
function buildEegFewShotPrompt(intentLabel, riskScore, latencyBudgetMs, tokenBudget, examples) {
  const cfg = {
    maxExamplesFewShot: 4,
    riskThresholdForFewShot: 0.4,
    errorRateThresholdForFewShot: 0.15,
    minLatencyForFewShotMs: 250,
    minTokenBudgetForFewShot: 512,
  };

  const signal = makeShotSignal(intentLabel, riskScore, latencyBudgetMs, tokenBudget);
  const decision = decideShotLevel(
    {
      taskId: signal.taskId,
      planeLabel: signal.planeLabel,
      riskScore: signal.riskScore,
      latencyBudgetMs: signal.latencyBudgetMs,
      tokenBudget: signal.tokenBudget,
      historicalErrorRate: signal.historicalErrorRate,
      requiresExamples: signal.requiresExamples,
    },
    cfg
  );

  const baseInstruction =
    `You are an EEG-driven assistive controller for intent '${intentLabel}'. ` +
    `Map EEG-derived features to safe, low-force control actions.`;

  const selectorResult = {
    task_id: signal.taskId,
    plane_label: signal.planeLabel,
    shot_level: decision.chosenLevel,
    examples:
      decision.chosenLevel === ShotLevel.FewShot
        ? examples.slice(0, decision.maxExamples)
        : [],
  };

  const finalPrompt = buildPrompt(baseInstruction, selectorResult);
  return { decision, finalPrompt };
}

module.exports = {
  buildEegFewShotPrompt,
};
