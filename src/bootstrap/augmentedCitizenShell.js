import IntegrityScanner from '../security/IntegrityScanner.js';
import AugmentedCitizenContext from '../augmented/AugmentedCitizenCharter.js';
import createResilientSessionShell from '../ResilientSessionShell.js';
import FairUsageRateLimiter from '../fairness/FairUsageRateLimiter.js';
import SecureMessagePipeline from '../security/SecureMessagePipeline.js';
import SafeLocalConfig from '../config/SafeLocalConfig.js';

export function createAugmentedCitizenShell() {
  const logger = console;

  // 1. Integrity manifest for this frontend bundle.
  const integrityManifest = {
    allowedModules: ['ChatUI', 'Telemetry', 'AugmentedCitizen'],
    capabilities: {
      globalEventTypes: ['error', 'unhandledrejection'],
      dynamicImportTargets: ['analytics-chunk', 'optional-tools']
    }
  };

  const integrityScanner = new IntegrityScanner(integrityManifest, { logger });

  const chatShell = createResilientSessionShell({
    appId: 'augmented-citizen-chat',
    storage: 'local',
    allowPersistence: true,
    onGlobalError(evt) {
      logger.error('[GlobalError]', evt);
    }
  });

  const configSchema = {
    allowAnalytics: { type: 'boolean', default: false },
    allowTrainingUse: { type: 'boolean', default: false },
    allowCommercialUpgradeFlows: { type: 'boolean', default: false }
  };
  const config = new SafeLocalConfig(configSchema);

  const citizen = new AugmentedCitizenContext({
    citizenId: 'citizen-local-001',
    jurisdiction: 'US-AZ',
    preferences: {
      allowAnalytics: config.get('allowAnalytics'),
      allowTrainingUse: config.get('allowTrainingUse'),
      allowCommercialUpgradeFlows: config.get('allowCommercialUpgradeFlows'),
      allowExperimentTagging: true
    },
    logger
  });

  const limiter = new FairUsageRateLimiter({ maxRequests: 20, windowMs: 60_000 });
  const pipeline = new SecureMessagePipeline();

  const chatUIModule = integrityScanner.registerModule('ChatUI');

  function sendChatMessage(rawText, purpose) {
    const cleanedText = pipeline.sanitizeOutgoing(rawText);
    const session = chatShell.getClientSession();

    const rate = limiter.record(session.id);
    if (!rate.allowed) {
      citizen.logLocalEvent('rate-limit-hit', { retryAfterMs: rate.retryAfterMs });
      throw new Error(`Too many messages. Try again later.`);
    }

    const actionEval = citizen.evaluateAction({
      type: purpose === 'experiment' ? 'experiment-log' : 'collect-analytics',
      meta: { purpose }
    });

    if (!actionEval.allowed) {
      citizen.logLocalEvent('action-denied', { purpose, reasons: actionEval.reasons });
    }

    const baseHeaders = { 'Content-Type': 'application/json' };
    const headers = citizen.decorateRequestHeaders(baseHeaders, { purpose });

    const body = JSON.stringify({
      message: cleanedText,
      clientSessionId: session.id,
      experiment: purpose === 'experiment'
    });

    return fetch('/api/chat', {
      method: 'POST',
      headers,
      body
    });
  }

  function registerGlobalErrorHandlers() {
    const telemetryModule = integrityScanner.registerModule('Telemetry');

    telemetryModule.addGlobalEventListener('error', (event) => {
      citizen.logLocalEvent('global-error', {
        message: event.message || null,
        source: event.filename || null
      });
    });

    telemetryModule.addGlobalEventListener('unhandledrejection', (event) => {
      citizen.logLocalEvent('unhandled-rejection', {
        reason: String(event.reason || '')
      });
    });
  }

  return {
    sendChatMessage,
    registerGlobalErrorHandlers,
    getCharter: () => citizen.getCharter(),
    getIntegrityAuditLog: () => integrityScanner.getAuditLog(),
    getLocalCitizenLog: () => citizen.getLocalLog()
  };
}

export default createAugmentedCitizenShell;
