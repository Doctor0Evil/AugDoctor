class NeuromorphFeature {
  constructor(payload) {
    this.sessionId = payload.sessionId;
    this.environmentId = payload.environmentId;
    this.intentLabel = payload.intentLabel;
    this.channelCount = payload.channelCount;
    this.fsHz = payload.fsHz;
    this.bandAlphaPower = payload.bandAlphaPower;
    this.bandBetaPower = payload.bandBetaPower;
    this.bandGammaPower = payload.bandGammaPower;
    this.eventLatencyMs = payload.eventLatencyMs;
    this.classifierConfidence = payload.classifierConfidence;
    this.ecoEnergyNj = payload.ecoEnergyNj;
    this.rewardScore = payload.rewardScore;
    this.safetyDecision = payload.safetyDecision;
  }
}

class NeuromorphRouter {
  constructor(cfg) {
    this.minConfidence = cfg.minConfidence;
    this.maxLatencyMs = cfg.maxLatencyMs;
    this.maxEcoEnergyNj = cfg.maxEcoEnergyNj;
  }

  route(feature) {
    if (feature.safetyDecision.startsWith("Deny")) return "Deny";
    if (
      feature.classifierConfidence < this.minConfidence ||
      feature.eventLatencyMs > this.maxLatencyMs ||
      feature.ecoEnergyNj > this.maxEcoEnergyNj
    ) {
      return "Defer";
    }
    return "Safe";
  }
}

module.exports = { NeuromorphFeature, NeuromorphRouter };
