export class IntegrityScanner {
  /**
   * @param {Object} manifest
   * @param {string[]} manifest.allowedModules
   * @param {Object} [manifest.capabilities]
   * @param {string[]} [manifest.capabilities.globalEventTypes]
   * @param {string[]} [manifest.capabilities.dynamicImportTargets]
   * @param {Object} [options]
   * @param {Console} [options.logger]
   */
  constructor(manifest, options) {
    if (!manifest || !Array.isArray(manifest.allowedModules)) {
      throw new Error('IntegrityScanner requires a manifest.allowedModules array.');
    }

    this.allowedModules = new Set(manifest.allowedModules);
    this.capabilities = {
      globalEventTypes: new Set(
        (manifest.capabilities && manifest.capabilities.globalEventTypes) || []
      ),
      dynamicImportTargets: new Set(
        (manifest.capabilities && manifest.capabilities.dynamicImportTargets) || []
      )
    };

    this.logger = (options && options.logger) || console;
    this.auditLog = [];
  }

  /**
   * Returns a module-scoped guard API which must be used by that module
   * when performing sensitive actions (global handlers, dynamic imports).
   */
  registerModule(moduleId) {
    if (!this.allowedModules.has(moduleId)) {
      const msg = `IntegrityScanner: module "${moduleId}" is not in allowedModules.`;
      this._record('module-denied', { moduleId, message: msg });
      throw new Error(msg);
    }

    const self = this;

    return {
      moduleId,

      addGlobalEventListener(type, handler, options) {
        return self._addGlobalEventListener(moduleId, type, handler, options);
      },

      removeGlobalEventListener(type, handler, options) {
        return self._removeGlobalEventListener(moduleId, type, handler, options);
      },

      /**
       * Controlled dynamic import.
       * @param {string} targetName - Logical name, e.g. "analytics-chunk".
       * @param {() => Promise<any>} importFactory - Function that calls import('...').
       */
      dynamicImport(targetName, importFactory) {
        return self._dynamicImport(moduleId, targetName, importFactory);
      }
    };
  }

  getAuditLog() {
    return this.auditLog.slice();
  }

  // Internal ---------------------------------------------------------------

  _addGlobalEventListener(moduleId, type, handler, options) {
    if (!this.capabilities.globalEventTypes.has(type)) {
      const msg = `IntegrityScanner: module "${moduleId}" attempted to register unsupported global event type "${type}".`;
      this._record('global-handler-denied', { moduleId, type, message: msg });
      this.logger.warn(msg);
      return;
    }
    if (typeof window === 'undefined' || typeof window.addEventListener !== 'function') {
      const msg = 'IntegrityScanner: window.addEventListener is not available.';
      this._record('global-handler-env-missing', { moduleId, type, message: msg });
      this.logger.warn(msg);
      return;
    }

    window.addEventListener(type, handler, options);
    this._record('global-handler-allowed', { moduleId, type });
  }

  _removeGlobalEventListener(moduleId, type, handler, options) {
    if (typeof window === 'undefined' || typeof window.removeEventListener !== 'function') {
      const msg = 'IntegrityScanner: window.removeEventListener is not available.';
      this._record('global-handler-remove-env-missing', { moduleId, type, message: msg });
      this.logger.warn(msg);
      return;
    }
    window.removeEventListener(type, handler, options);
    this._record('global-handler-removed', { moduleId, type });
  }

  _dynamicImport(moduleId, targetName, importFactory) {
    if (!this.capabilities.dynamicImportTargets.has(targetName)) {
      const msg = `IntegrityScanner: module "${moduleId}" attempted dynamicImport of disallowed target "${targetName}".`;
      this._record('dynamic-import-denied', { moduleId, targetName, message: msg });
      this.logger.error(msg);
      return Promise.reject(new Error(msg));
    }
    if (typeof importFactory !== 'function') {
      const msg = `IntegrityScanner: importFactory for target "${targetName}" must be a function.`;
      this._record('dynamic-import-error', { moduleId, targetName, message: msg });
      return Promise.reject(new Error(msg));
    }

    this._record('dynamic-import-start', { moduleId, targetName });

    try {
      const result = importFactory();
      if (!result || typeof result.then !== 'function') {
        const msg = `IntegrityScanner: importFactory for "${targetName}" did not return a Promise.`;
        this._record('dynamic-import-error', { moduleId, targetName, message: msg });
        return Promise.reject(new Error(msg));
      }

      return result.then((mod) => {
        this._record('dynamic-import-success', { moduleId, targetName });
        return mod;
      }).catch((err) => {
        this._record('dynamic-import-failure', { moduleId, targetName, error: String(err) });
        throw err;
      });
    } catch (err) {
      this._record('dynamic-import-error', { moduleId, targetName, error: String(err) });
      return Promise.reject(err);
    }
  }

  _record(eventType, payload) {
    const entry = {
      at: new Date().toISOString(),
      eventType,
      ...payload
    };
    this.auditLog.push(entry);
  }
}

export default IntegrityScanner;
