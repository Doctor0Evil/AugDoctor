import PolicySnapshotEngine from '../policy/PolicySnapshotEngine.js';

/**
 * @typedef {Object} KnowledgeObject
 * @property {string} id
 * @property {string} kind           - e.g. "experiment", "analysis", "chat-request"
 * @property {string} capabilityKey  - policy capability required to execute
 * @property {Object} payload        - arbitrary JSON-serializable data
 * @property {number} createdAt
 * @property {('pending-policy'|'eligible'|'executed'|'rejected')} status
 * @property {string|null} statusReason
 * @property {string} policyVersionAtCreation
 * @property {string|null} policyVersionAtExecution
 */

export class KnowledgeObjectEscrow {
  /**
   * @param {Object} options
   * @param {PolicySnapshotEngine} options.policyEngine
   * @param {Console} [options.logger]
   */
  constructor(options) {
    if (!options || !(options.policyEngine instanceof PolicySnapshotEngine)) {
      throw new Error('KnowledgeObjectEscrow requires a PolicySnapshotEngine instance.');
    }

    this.policyEngine = options.policyEngine;
    this.logger = options.logger || console;

    /** @type {Map<string, KnowledgeObject>} */
    this._objects = new Map();
    this._idCounter = 0;
  }

  /**
   * Create a new knowledge object.
   * Creation is never blocked: object is at least stored as "pending-policy".
   */
  create({ kind, capabilityKey, payload }) {
    if (!kind || !capabilityKey) {
      throw new Error('create requires kind and capabilityKey.');
    }

    const id = this._nextId();
    const snapshot = this.policyEngine.getSnapshot();
    const allowed = this.policyEngine.isCapabilityAllowed(capabilityKey);

    const ko = {
      id,
      kind,
      capabilityKey,
      payload: payload || {},
      createdAt: Date.now(),
      status: allowed ? 'eligible' : 'pending-policy',
      statusReason: allowed
        ? 'Capability currently allowed at creation.'
        : 'Capability not yet allowed; awaiting policy change.',
      policyVersionAtCreation: snapshot.version,
      policyVersionAtExecution: null
    };

    this._objects.set(id, ko);

    this.logger.info('[KO-ESCROW] created', {
      id,
      kind,
      capabilityKey,
      policyVersion: snapshot.version,
      status: ko.status
    });

    return ko;
  }

  /**
   * Re-evaluate all pending objects under the latest policies.
   * Called whenever you update the PolicySnapshotEngine.
   */
  reconcileWithCurrentPolicy() {
    const snapshot = this.policyEngine.getSnapshot();
    let promoted = 0;

    for (const ko of this._objects.values()) {
      if (ko.status !== 'pending-policy') continue;

      const allowed = this.policyEngine.isCapabilityAllowed(ko.capabilityKey);
      if (allowed) {
        ko.status = 'eligible';
        ko.statusReason = `Promoted to eligible under policy version ${snapshot.version}.`;
        this.logger.info('[KO-ESCROW] promoted', { id: ko.id, capabilityKey: ko.capabilityKey });
        promoted += 1;
      }
    }

    return { promoted, policyVersion: snapshot.version };
  }

  /**
   * Get objects by status.
   */
  listByStatus(status) {
    const items = [];
    for (const ko of this._objects.values()) {
      if (ko.status === status) items.push({ ...ko });
    }
    return items;
  }

  /**
   * Mark an object as executed (after executor runs it).
   */
  markExecuted(id) {
    const ko = this._objects.get(id);
    if (!ko) return;
    ko.status = 'executed';
    ko.statusReason = 'Executed successfully.';
    ko.policyVersionAtExecution = this.policyEngine.getSnapshot().version;
  }

  /**
   * Mark an object as rejected (e.g., permanently disallowed).
   */
  markRejected(id, reason) {
    const ko = this._objects.get(id);
    if (!ko) return;
    ko.status = 'rejected';
    ko.statusReason = reason || 'Rejected by executor/policy.';
    ko.policyVersionAtExecution = this.policyEngine.getSnapshot().version;
  }

  _nextId() {
    this._idCounter += 1;
    return `ko_${this._idCounter.toString(16)}_${Date.now().toString(16)}`;
  }
}

export default KnowledgeObjectEscrow;
