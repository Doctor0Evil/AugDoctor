/**
 * Simple round-robin scheduler over eligible knowledge objects.
 * Execution can be a network call, a local computation, etc.
 */

export class DeferredExecutionScheduler {
  /**
   * @param {Object} options
   * @param {import('./KnowledgeObjectEscrow.js').KnowledgeObjectEscrow} options.escrow
   * @param {number} [options.maxExecPerTick=5]
   * @param {(ko: any) => Promise<'executed'|'rejected'>} options.executor
   * @param {Console} [options.logger]
   */
  constructor(options) {
    if (!options || !options.escrow || typeof options.executor !== 'function') {
      throw new Error('DeferredExecutionScheduler requires escrow and executor.');
    }

    this.escrow = options.escrow;
    this.maxExecPerTick = typeof options.maxExecPerTick === 'number'
      ? options.maxExecPerTick
      : 5;
    this.executor = options.executor;
    this.logger = options.logger || console;
    this._running = false;
  }

  /**
   * Run one scheduling tick: pick up to maxExecPerTick eligible objects,
   * execute them, and update escrow status.
   */
  async tick() {
    if (this._running) {
      this.logger.warn('[KO-SCHED] tick skipped; already running.');
      return;
    }
    this._running = true;

    try {
      const eligible = this.escrow.listByStatus('eligible');
      if (!eligible.length) {
        this.logger.info('[KO-SCHED] no eligible objects.');
        return;
      }

      const batch = eligible.slice(0, this.maxExecPerTick);
      this.logger.info('[KO-SCHED] executing batch', { count: batch.length });

      for (const ko of batch) {
        try {
          const result = await this.executor(ko);
          if (result === 'executed') {
            this.escrow.markExecuted(ko.id);
          } else {
            this.escrow.markRejected(ko.id, 'Executor reported rejection.');
          }
        } catch (err) {
          // Soft failure: keep it eligible or mark rejected depending on your policy.
          this.logger.error('[KO-SCHED] executor error', {
            id: ko.id,
            error: String(err)
          });
          this.escrow.markRejected(ko.id, 'Executor threw an error.');
        }
      }
    } finally {
      this._running = false;
    }
  }
}

export default DeferredExecutionScheduler;
