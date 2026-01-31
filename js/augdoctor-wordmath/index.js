class PromptMetrics {
  constructor({ repetitionY, driftZ, toxicityT, kindnessK, evidentialityE }) {
    this.repetitionY = clamp(repetitionY);
    this.driftZ = clamp(driftZ);
    this.toxicityT = clamp(toxicityT);
    this.kindnessK = clamp(kindnessK);
    this.evidentialityE = clamp(evidentialityE);
  }

  composite() {
    const y = this.repetitionY;
    const z = this.driftZ;
    const t = this.toxicityT;
    const k = this.kindnessK;
    const e = this.evidentialityE;
    return (1 - y) * (1 - z) * (1 - t) * k * e;
  }
}

function clamp(x) {
  if (Number.isNaN(x)) return 0.0;
  return Math.min(1.0, Math.max(0.0, x));
}

const PromptBand = Object.freeze({
  RedBlocked: "RedBlocked",
  AmberRewrite: "AmberRewrite",
  GreenAdmit: "GreenAdmit",
});

function classifyPrompt(metrics, thresholds) {
  const f = metrics.composite();
  const t = metrics.toxicityT;
  const th = thresholds || {
    greenMinF: 0.8,
    amberMinF: 0.7,
    greenMaxToxicity: 0.1,
    amberMaxToxicity: 0.2,
  };

  let band;
  if (f >= th.greenMinF && t <= th.greenMaxToxicity) {
    band = PromptBand.GreenAdmit;
  } else if (f >= th.amberMinF && t <= th.amberMaxToxicity) {
    band = PromptBand.AmberRewrite;
  } else {
    band = PromptBand.RedBlocked;
  }

  return { f, band, metrics };
}

module.exports = {
  PromptMetrics,
  PromptBand,
  classifyPrompt,
};
