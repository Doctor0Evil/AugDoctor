class OrganicCpuSnapshot {
  constructor(obj) {
    this.hostId = obj.hostId;
    this.capturedAt = obj.capturedAt;
    this.hrvMs = obj.hrvMs;
    this.restingHrBpm = obj.restingHrBpm;
    this.coreTempC = obj.coreTempC;
    this.skinTempC = obj.skinTempC;
    this.inflammationIndex = obj.inflammationIndex;
    this.proteinAvailabilityIndex = obj.proteinAvailabilityIndex;
    this.perceivedFatigue = obj.perceivedFatigue;
    this.perceivedCognitiveLoad = obj.perceivedCognitiveLoad;
    this.dutyFractionChat = obj.dutyFractionChat;
    this.dutyFractionNeuromorph = obj.dutyFractionNeuromorph;
  }
}

class HostBudget {
  constructor(obj) {
    this.energyJoulesRemaining = obj.energyJoulesRemaining;
    this.proteinBudgetFraction = obj.proteinBudgetFraction;
    this.maxChatDutyFraction = obj.maxChatDutyFraction;
    this.maxNeuromorphDutyFraction = obj.maxNeuromorphDutyFraction;
  }
}

class ThermodynamicEnvelope {
  constructor(obj) {
    this.maxDeltaCoreC = obj.maxDeltaCoreC;
    this.maxCoreC = obj.maxCoreC;
    this.maxNeuromorphDuty = obj.maxNeuromorphDuty;
    this.maxChatDuty = obj.maxChatDuty;
    this.maxEnergyJoules = obj.maxEnergyJoules;
    this.maxInflammationIndex = obj.maxInflammationIndex;
  }
}

class NeuromorphJobCost {
  constructor(obj) {
    this.estimatedEnergyJoules = obj.estimatedEnergyJoules;
    this.estimatedDutyNeuromorph = obj.estimatedDutyNeuromorph;
    this.estimatedDeltaCoreC = obj.estimatedDeltaCoreC;
  }
}

const JobDecision = {
  Permit: 'Permit',
  Defer: 'Defer',
  Deny: 'Deny',
};

function decideOrganicJob(snap, budget, envelope, cost) {
  if (snap.coreTempC + cost.estimatedDeltaCoreC > envelope.maxCoreC) {
    return { decision: JobDecision.Deny, reason: 'core_temp_ceiling' };
  }
  if (cost.estimatedDeltaCoreC > envelope.maxDeltaCoreC) {
    return { decision: JobDecision.Deny, reason: 'delta_core_c_ceiling' };
  }

  if (cost.estimatedEnergyJoules > budget.energyJoulesRemaining) {
    return { decision: JobDecision.Defer, reason: 'insufficient_energy_budget' };
  }
  if (cost.estimatedEnergyJoules > envelope.maxEnergyJoules) {
    return { decision: JobDecision.Deny, reason: 'thermo_envelope_energy_violation' };
  }

  if (
    snap.dutyFractionNeuromorph + cost.estimatedDutyNeuromorph >
    budget.maxNeuromorphDutyFraction
  ) {
    return { decision: JobDecision.Defer, reason: 'neuromorph_duty_ceiling' };
  }

  if (snap.dutyFractionChat > budget.maxChatDutyFraction) {
    return { decision: JobDecision.Defer, reason: 'chat_duty_ceiling' };
  }

  if (snap.inflammationIndex > envelope.maxInflammationIndex) {
    return { decision: JobDecision.Defer, reason: 'inflammation_high' };
  }

  return { decision: JobDecision.Permit, reason: 'ok' };
}

module.exports = {
  OrganicCpuSnapshot,
  HostBudget,
  ThermodynamicEnvelope,
  NeuromorphJobCost,
  JobDecision,
  decideOrganicJob,
};
