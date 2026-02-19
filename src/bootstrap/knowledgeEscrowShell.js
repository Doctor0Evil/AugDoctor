import PolicySnapshotEngine from '../policy/PolicySnapshotEngine.js';
import KnowledgeObjectEscrow from '../knowledge/KnowledgeObjectEscrow.js';
import DeferredExecutionScheduler from '../knowledge/DeferredExecutionScheduler.js';

export function createKnowledgeEscrowShell() {
  const logger = console;

  // Initial policy: no "tool:advanced" yet, basic chat allowed.
  const policyEngine = new PolicySnapshotEngine({
    version: 'v1-basic',
    capabilities: {
      'chat:standard': true,
      'tool:advanced': false
    }
  });

  const escrow = new KnowledgeObjectEscrow({ policyEngine, logger });

  const scheduler = new DeferredExecutionScheduler({
    escrow,
    maxExecPerTick: 3,
    async executor(ko) {
      if (ko.kind === 'chat-request') {
        // Example: send to your backend.
        await fetch('/api/chat', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            capability: ko.capabilityKey,
            payload: ko.payload
          })
        });
        return 'executed';
      }

      // Unknown kinds: skip.
      return 'rejected';
    },
    logger
  });

  // Public API ----------------------------------------------------------------

  function submitChatRequest(text, capabilityKey = 'chat:standard') {
    const ko = escrow.create({
      kind: 'chat-request',
      capabilityKey,
      payload: { text }
    });
    return ko;
  }

  function updatePolicies(newSnapshot) {
    policyEngine.updateSnapshot(newSnapshot);
    const res = escrow.reconcileWithCurrentPolicy();
    logger.info('[POLICY] reconcile result', res);
  }

  async function runSchedulerTick() {
    await scheduler.tick();
  }

  return {
    submitChatRequest,
    updatePolicies,
    runSchedulerTick,
    getSnapshot: () => policyEngine.getSnapshot(),
    listPending: () => escrow.listByStatus('pending-policy'),
    listEligible: () => escrow.listByStatus('eligible'),
    listExecuted: () => escrow.listByStatus('executed')
  };
}

export default createKnowledgeEscrowShell;
