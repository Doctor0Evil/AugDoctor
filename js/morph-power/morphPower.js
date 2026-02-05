class MorphVector {
  constructor({ eco, cyber, neuro, smart }) {
    this.eco = eco;
    this.cyber = cyber;
    this.neuro = neuro;
    this.smart = smart;
  }

  l1Norm() {
    return (
      Math.abs(this.eco) +
      Math.abs(this.cyber) +
      Math.abs(this.neuro) +
      Math.abs(this.smart)
    );
  }
}

class PowerDecision {
  constructor(payload) {
    this.allowedCorridor = payload.allowed_corridor;
    this.reasons = payload.reasons || [];
    this.proposeOnly = payload.propose_only;
    this.maxDocuments = payload.max_documents;
    this.maxExternalCalls = payload.max_external_calls;
  }

  canReadMoreDocs(currentCount) {
    return currentCount < this.maxDocuments;
  }

  canCallTool(currentCount) {
    return currentCount < this.maxExternalCalls;
  }
}

module.exports = {
  MorphVector,
  PowerDecision,
};
