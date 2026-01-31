class AutonomyProfile {
  constructor(payload) {
    this.profileId = payload.profileId;
    this.hostId = payload.hostId;
    this.maxEcoEnergyNjPerMinute = payload.maxEcoEnergyNjPerMinute;
    this.maxAutonomousActionsPerMinute = payload.maxAutonomousActionsPerMinute;
    this.maxRiskScore = payload.maxRiskScore;
    this.minLifeforceScalar = payload.minLifeforceScalar;
    this.maxIdentityDriftPerDay = payload.maxIdentityDriftPerDay;
  }
}

class AutonomyConstraint {
  constructor(obj) {
    this.highestRiskScore = obj.highest_risk_score;
    this.worstLifeforceScalar = obj.worst_lifeforce_scalar;
  }
}

class AutonomyTraceAttributes {
  constructor(obj) {
    this.schemaversion = obj.schemaversion;
    this.host_id = obj.host_id;
    this.session_id = obj.session_id;
    this.environment_id = obj.environment_id;
    this.plane = obj.plane;

    this.stress = obj.stress;
    this.fatigue = obj.fatigue;
    this.reward = obj.reward;
    this.safety = obj.safety;
    this.lifeforce_scalar = obj.lifeforce_scalar;
    this.eco_energy_nj = obj.eco_energy_nj;
    this.risk = obj.risk;

    this.actions_last_minute = obj.actions_last_minute;
    this.identity_drift_today = obj.identity_drift_today;

    this.decision_autonomy_level = obj.decision_autonomy_level;
    this.decision_shot_level_label = obj.decision_shot_level_label;

    this.constraint = new AutonomyConstraint(obj.constraint);
  }
}

class AutonomyDecision {
  constructor(level, label) {
    this.autonomy_level = level;
    this.shot_level_label = label;
  }
}

class AutonomyGovernor {
  static decide(profile, trace, hostConsentActive) {
    // Mirror Rust runtime assertion semantics in a dev build.[file:42][file:41]
    if (trace.constraint.highestRiskScore > 0.7 &&
        trace.decision_autonomy_level > 0.8) {
      console.warn("G(risk_high -> autonomy <= θ) violated");
    }

    if (!hostConsentActive) {
      return new AutonomyDecision(0.0, "ZeroShot");
    }
    if (trace.lifeforce_scalar < profile.minLifeforceScalar) {
      return new AutonomyDecision(0.0, "ZeroShot");
    }
    if (trace.eco_energy_nj > profile.maxEcoEnergyNjPerMinute) {
      return new AutonomyDecision(0.2, "ZeroShot");
    }
    if (trace.risk > profile.maxRiskScore) {
      return new AutonomyDecision(0.0, "ZeroShot");
    }
    if (trace.actions_last_minute >= profile.maxAutonomousActionsPerMinute) {
      return new AutonomyDecision(0.1, "ZeroShot");
    }
    if (trace.identity_drift_today > profile.maxIdentityDriftPerDay) {
      return new AutonomyDecision(0.0, "ZeroShot");
    }

    const s = Math.max(0.0, 1.0 - trace.stress);
    const f = Math.max(0.0, 1.0 - trace.fatigue);
    const r = (trace.reward + 1.0) / 2.0;
    const sa = Math.max(0.0, trace.safety);
    let autonomy = Math.min(1.0, Math.max(0.0, 0.4 * s + 0.2 * f + 0.2 * r + 0.2 * sa));

    if (trace.eco_energy_nj > 0.7 * profile.maxEcoEnergyNjPerMinute) {
      autonomy *= 0.7;
    }

    const shot = trace.risk > 0.7 ? "FewShot" : "ZeroShot";
    return new AutonomyDecision(autonomy, shot);
  }
}

module.exports = {
  AutonomyProfile,
  AutonomyTraceAttributes,
  AutonomyDecision,
  AutonomyGovernor
};
