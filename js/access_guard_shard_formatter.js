const { v4: uuidv4 } = require('uuid');

class AccessGuardShardFormatter {
  constructor() {
    this.allowedRoles = new Map([['augmented_citizen', true], ['authorized_researcher', true]]);
  }

  formatAndValidate(input) {
    // Filled defaults if partial
    const filled = {
      did: input.did || 'bostrom:default-host',
      role: input.role || 'augmented_citizen',
      consent_valid: input.consent_valid !== undefined ? input.consent_valid : true,
      lifeforce: input.lifeforce || { level: 'green', quota_pct: 0.85 },
      eco: input.eco || { nj_per_flop: 52.0, remaining: 10000.0, cap: 50000.0 },
      scale: input.scale || { daily_cap: 1440, used: 0 },
      op_type: input.op_type || 'token_spend_self_aug',
    };

    // DID validation
    if (!filled.did.startsWith('did:') && !filled.did.startsWith('aln:') && !filled.did.startsWith('bostrom:')) {
      throw new Error(`Invalid DID: ${filled.did}`);
    }

    // Role check
    if (!this.allowedRoles.has(filled.role)) {
      throw new Error(`Unauthorized role: ${filled.role}`);
    }

    // Consent
    if (!filled.consent_valid) {
      throw new Error('Consent invalid');
    }

    // Lifeforce
    if (filled.lifeforce.level === 'red' || filled.lifeforce.quota_pct < 0.40) {
      throw new Error(`Lifeforce too low: ${filled.lifeforce.level}`);
    }

    // Eco
    if (filled.eco.remaining < filled.eco.nj_per_flop * 1.2) {
      throw new Error(`Eco exceeded: remaining ${filled.eco.remaining}`);
    }

    // SCALE
    if (filled.scale.used >= filled.scale.daily_cap) {
      throw new Error(`Turns cap hit: used ${filled.scale.used} >= ${filled.scale.daily_cap}`);
    }

    // Format JSON output
    return JSON.stringify(filled, null, 2);
  }
}

// Usage: const formatter = new AccessGuardShardFormatter();
// const validJson = formatter.formatAndValidate({ did: 'bostrom:host456', role: 'augmented_citizen', op_type: 'neural_rope_append' });
// console.log(validJson);
