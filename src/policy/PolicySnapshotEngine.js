export class PolicySnapshotEngine {
  /**
   * @param {Object} initialSnapshot
   * @param {string} initialSnapshot.version
   * @param {Object<string, boolean>} initialSnapshot.capabilities
   */
  constructor(initialSnapshot) {
    if (
      !initialSnapshot ||
      typeof initialSnapshot.version !== 'string' ||
      typeof initialSnapshot.capabilities !== 'object'
    ) {
      throw new Error('PolicySnapshotEngine requires { version, capabilities }.');
    }

    this._current = {
      version: initialSnapshot.version,
      capabilities: { ...initialSnapshot.capabilities },
      updatedAt: Date.now()
    };

    this._history = [this._current];
  }

  /**
   * Replace snapshot with a new one (e.g., from backend, local config, or manual change).
   */
  updateSnapshot(next) {
    if (
      !next ||
      typeof next.version !== 'string' ||
      typeof next.capabilities !== 'object'
    ) {
      throw new Error('updateSnapshot requires { version, capabilities }.');
    }

    this._current = {
      version: next.version,
      capabilities: { ...next.capabilities },
      updatedAt: Date.now()
    };
    this._history.push(this._current);
  }

  /**
   * Check whether a capability key is currently allowed.
   */
  isCapabilityAllowed(key) {
    return !!this._current.capabilities[key];
  }

  /**
   * Optionally check whether a capability *might* be allowed later, given
   * a policy hint. (Here we just mirror current state; you can extend.)
   */
  willLikelyBeAllowedLater(key) {
    // Simple heuristic: if present but false, assume "maybe later".
    if (Object.prototype.hasOwnProperty.call(this._current.capabilities, key)) {
      return !this._current.capabilities[key];
    }
    // Unknown capability: treat as uncertain but potentially future-valid.
    return true;
  }

  getSnapshot() {
    return { ...this._current, capabilities: { ...this._current.capabilities } };
  }

  getHistory() {
    return this._history.slice();
  }
}

export default PolicySnapshotEngine;
