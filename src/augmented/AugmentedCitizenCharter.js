const RIGHTS_CHARTER = [
  {
    id: 'transparency',
    title: 'Right to Transparent Operation',
    description: 'The citizen has the right to know when and why AI-chat actions occur, including logging significant events locally in a human-readable form.'
  },
  {
    id: 'consent-data-use',
    title: 'Right to Purpose-Bound Data Use',
    description: 'The citizen may choose whether their chat content is used for analytics or training, beyond what is strictly necessary to provide the service.'
  },
  {
    id: 'no-forced-upgrade',
    title: 'Right to Non-Greedy Upgrades',
    description: 'The system must not auto-enroll the citizen into paid tiers or expanded billing without explicit, informed, local consent.'
  },
  {
    id: 'experiment-freedom',
    title: 'Right to Run Local Experiments',
    description: 'The citizen may run non-destructive experiments in AI chats, log their own results, and adjust local policies, provided they comply with applicable law and platform terms.'
  },
  {
    id: 'minimal-tracking',
    title: 'Right to Minimal Tracking',
    description: 'Only metrics necessary for stability and security may be collected locally by default; additional analytics require opt-in.'
  },
  {
    id: 'revocable-consent',
    title: 'Right to Revoke Consent',
    description: 'Any granted consent for expanded data use or experimentation metadata is revocable, and revocation must be respected immediately by local tools.'
  }
];

/**
 * @typedef {Object} AugmentedCitizenPreferences
 * @property {boolean} allowAnalytics
 * @property {boolean} allowTrainingUse
 * @property {boolean} allowCommercialUpgradeFlows
 * @property {boolean} allowExperimentTagging
 */

/**
 * A local, citizen-centric policy engine for AI-chat usage.
 * This does not replace law or platform policy; it structures your own constraints.
 */
export class AugmentedCitizenContext {
  /**
   * @param {Object} options
   * @param {string} options.citizenId - Local, opaque identifier (not PII).
   * @param {string} options.jurisdiction - e.g. "US-AZ".
   * @param {AugmentedCitizenPreferences} options.preferences
   * @param {Console} [options.logger]
   */
  constructor(options) {
    if (!options || typeof options.citizenId !== 'string') {
      throw new Error('AugmentedCitizenContext requires a citizenId string.');
    }
    if (!options.preferences) {
      throw new Error('AugmentedCitizenContext requires preferences.');
    }

    this.citizenId = options.citizenId;
    this.jurisdiction = options.jurisdiction || 'US-UNKNOWN';
    this.preferences = {
      allowAnalytics: !!options.preferences.allowAnalytics,
      allowTrainingUse: !!options.preferences.allowTrainingUse,
      allowCommercialUpgradeFlows: !!options.preferences.allowCommercialUpgradeFlows,
      allowExperimentTagging: options.preferences.allowExperimentTagging !== false
    };
    this.logger = options.logger || console;
    this.localLog = [];
  }

  /**
   * Return the rights charter, for UI display or documentation.
   */
  getCharter() {
    return RIGHTS_CHARTER.slice();
  }

  /**
   * Evaluate a proposed action against local preferences and charter.
   * @param {Object} action
   * @param {string} action.type - e.g. "collect-analytics", "training-use", "upgrade-offer", "experiment-log".
   * @param {Object} [action.meta]
   * @returns {{allowed: boolean, reasons: string[]}}
   */
  evaluateAction(action) {
    if (!action || typeof action.type !== 'string') {
      return { allowed: false, reasons: ['Invalid action definition.'] };
    }

    const reasons = [];

    switch (action.type) {
      case 'collect-analytics':
        if (!this.preferences.allowAnalytics) {
          reasons.push('Analytics collection is disabled by citizen preference.');
          return { allowed: false, reasons };
        }
        break;

      case 'training-use':
        if (!this.preferences.allowTrainingUse) {
          reasons.push('Use of chat content for training is disabled by citizen preference.');
          return { allowed: false, reasons };
        }
        break;

      case 'upgrade-offer':
        if (!this.preferences.allowCommercialUpgradeFlows) {
          reasons.push('Commercial upgrade flows are disabled by citizen preference.');
          return { allowed: false, reasons };
        }
        break;

      case 'experiment-log':
        if (!this.preferences.allowExperimentTagging) {
          reasons.push('Experiment tagging is disabled by citizen preference.');
          return { allowed: false, reasons };
        }
        break;

      default:
        reasons.push('Unknown action type; defaulting to deny for safety.');
        return { allowed: false, reasons };
    }

    reasons.push('Action complies with local preferences and charter.');
    return { allowed: true, reasons };
  }

  /**
   * Decorate outgoing request headers with non-PII metadata that expresses
   * the citizens experimental intent and local policy state.
   *
   * @param {Object} baseHeaders
   * @param {Object} meta
   * @param {string} [meta.purpose] - e.g. "chat", "experiment", "analytics".
   * @returns {Object} headers
   */
  decorateRequestHeaders(baseHeaders, meta) {
    const headers = { ...(baseHeaders || {}) };
    const purpose = (meta && meta.purpose) || 'chat';

    headers['X-Augmented-Citizen'] = this.citizenId;
    headers['X-Augmented-Jurisdiction'] = this.jurisdiction;
    headers['X-Augmented-Purpose'] = purpose;

    headers['X-Augmented-Analytics-Allowed'] = String(this.preferences.allowAnalytics);
    headers['X-Augmented-Training-Allowed'] = String(this.preferences.allowTrainingUse);

    return headers;
  }

  /**
   * Log a significant local event to a human-readable buffer, in line with
   * the transparency right. This never leaves the browser unless you choose to export it.
   *
   * @param {string} kind
   * @param {Object} [detail]
   */
  logLocalEvent(kind, detail) {
    const entry = {
      at: new Date().toISOString(),
      kind,
      detail: detail || null
    };
    this.localLog.push(entry);
    if (this.preferences.allowAnalytics) {
      this.logger.info('[AugmentedCitizen]', kind, detail || '');
    }
  }

  /**
   * Export the local log for inspection or archiving.
   */
  getLocalLog() {
    return this.localLog.slice();
  }
}

export default AugmentedCitizenContext;
