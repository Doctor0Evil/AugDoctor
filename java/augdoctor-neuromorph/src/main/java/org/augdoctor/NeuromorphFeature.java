package org.augdoctor;

public record NeuromorphFeature(
    String sessionId,
    String environmentId,
    String intentLabel,
    int channelCount,
    int fsHz,
    float bandAlphaPower,
    float bandBetaPower,
    float bandGammaPower,
    int eventLatencyMs,
    float classifierConfidence,
    float ecoEnergyNj,
    float rewardScore,
    String safetyDecision
) {}
