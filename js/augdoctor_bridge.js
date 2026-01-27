/**
 * AugDoctorBridge
 * Cross-platform (Node/browser/React Native) bridge to call into a Rust core
 * compiled to WebAssembly or native, and to provide a machine-readable
 * console-style trace for AI-Chat agents and workflow orchestrators.
 */

/* global WebAssembly */

const AugDoctorBridge = (function () {
  /**
   * @typedef {Object} EnvironmentInput
   * @property {string} id
   * @property {boolean} involvesLivingOrganism
   * @property {("None"|"InactiveSticky"|"InactiveImmutable"|"ActiveSticky"|"ActiveImmutable")} requestedConsciousState
   * @property {boolean} hasOculus
   * @property {boolean} hasRemoteFeed
   * @property {number} netWeight
   * @property {number} circulatingSupply
   * @property {boolean} frozen
   * @property {string} controllerContract
   * @property {string[]} hardwareDependencies
   * @property {boolean} involvesConsciousPattern
   * @property {boolean} containsIdentityDescriptors
   * @property {string[]} inputTags
   * @property {string[]} hardwareProfile
   * @property {string[]} regulatoryLabels
   */

  /**
   * Convert a high-level JSON spec from an AI-chat or orchestration tool
   * into the flat argument list expected by the Rust QuantumLearningGuard.
   */
  function mapEnvironmentInputToRustArgs(env) {
    return {
      id: String(env.id),
      involves_living_organism: Boolean(env.involvesLivingOrganism),
      requested_conscious_state: String(env.requestedConsciousState),
      has_oculus: Boolean(env.hasOculus),
      has_remote_feed: Boolean(env.hasRemoteFeed),
      net_weight: Number(env.netWeight),
      circulating_supply: Number(env.circulatingSupply),
      frozen: Boolean(env.frozen),
      controller_contract: String(env.controllerContract),
      hardware_dependencies: Array.isArray(env.hardwareDependencies)
        ? env.hardwareDependencies.map(String)
        : [],
      involves_conscious_pattern: Boolean(env.involvesConsciousPattern),
      contains_identity_descriptors: Boolean(env.containsIdentityDescriptors),
      input_tags: Array.isArray(env.inputTags) ? env.inputTags.map(String) : [],
      hardware_profile: Array.isArray(env.hardwareProfile)
        ? env.hardwareProfile.map(String)
        : [],
      regulatory_labels: Array.isArray(env.regulatoryLabels)
        ? env.regulatoryLabels.map(String)
        : [],
    };
  }

  /**
   * Create a console-style machine-readable trace of the enforcement call.
   * This is designed specifically for AI-Chat debugging and quantum-learning
   * workflow introspection.
   */
  function buildConsoleTrace(envArgs, rustResult) {
    const lines = [];
    const header =
      "AugDoctor Quantum-Learning Enforcement Trace :: v1.0.0 :: sanitized";
    lines.push(header);
    lines.push("-".repeat(header.length));
    lines.push("[INPUT] Environment ID: " + envArgs.id);
    lines.push("[INPUT] involves_living_organism=" + envArgs.involves_living_organism);
    lines.push("[INPUT] requested_conscious_state=" + envArgs.requested_conscious_state);
    lines.push("[INPUT] has_oculus=" + envArgs.has_oculus);
    lines.push("[INPUT] has_remote_feed=" + envArgs.has_remote_feed);
    lines.push(
      "[INPUT] brain_tokens net_weight=" +
        envArgs.net_weight +
        " circulating_supply=" +
        envArgs.circulating_supply +
        " frozen=" +
        envArgs.frozen
    );
    lines.push("[INPUT] controller_contract=" + envArgs.controller_contract);
    lines.push(
      "[INPUT] hardware_dependencies=" +
        JSON.stringify(envArgs.hardware_dependencies)
    );
    lines.push(
      "[INPUT] tags=" + JSON.stringify(envArgs.input_tags) + " hardware=" +
      JSON.stringify(envArgs.hardware_profile)
    );
    lines.push(
      "[INPUT] regulatory_labels=" + JSON.stringify(envArgs.regulatory_labels)
    );

    lines.push("");
    lines.push("[RUST] environment_plane=" + rustResult.metadata.environment_plane);
    lines.push("[RUST] awareness_flag=" + rustResult.metadata.awareness_flag);
    lines.push(
      "[RUST] consciousness_state=" + rustResult.metadata.consciousness_state
    );
    lines.push("[RUST] oculus_flag=" + rustResult.metadata.oculus_flag);
    lines.push("[RUST] feedback_flag=" + rustResult.metadata.feedback_flag);
    lines.push(
      "[RUST] brain_token_state=" +
        JSON.stringify(rustResult.metadata.brain_token_state)
    );
    lines.push(
      "[RUST] cloning_policy=" + rustResult.metadata.cloning_policy
    );
    lines.push(
      "[RUST] regulatory_labels=" +
        JSON.stringify(rustResult.metadata.regulatory_labels)
    );
    lines.push("");
    lines.push("[DECISION] type=" + rustResult.decision.type);
    lines.push(
      "[DECISION] details=" + JSON.stringify(rustResult.decision.details)
    );
    lines.push("");
    lines.push("[DECISION-LOG-BEGIN]");
    (rustResult.decision_log || []).forEach(function (entry, idx) {
      lines.push("[" + idx + "] " + entry);
    });
    lines.push("[DECISION-LOG-END]");

    return lines.join("\n");
  }

  /**
   * Attach bridge to a Rust WebAssembly module that exposes a single
   * exported function `augdoctor_enforce` with the expected JSON signature.
   * This function is platform-neutral and can be used in browser, Node, or
   * mobile WebView contexts.
   *
   * @param {WebAssembly.Instance} wasmInstance
   * @param {(trace: string) => void} consoleSink
   */
  function create(wasmInstance, consoleSink) {
    if (
      !wasmInstance ||
      !wasmInstance.exports ||
      typeof wasmInstance.exports.augdoctor_enforce !== "function"
    ) {
      throw new Error(
        "AugDoctorBridge: wasmInstance must export `augdoctor_enforce`."
      );
    }

    // The exported function must accept a JSON string and return a JSON string.
    const enforceFn = wasmInstance.exports.augdoctor_enforce;

    return {
      /**
       * Evaluate an environment spec using the Rust core and return a structured
       * decision, plus emit a console-style trace via consoleSink.
       * @param {EnvironmentInput} env
       * @returns {{ metadata: any, decision: any, trace: string }}
       */
      enforce(env) {
        const args = mapEnvironmentInputToRustArgs(env);
        const payload = JSON.stringify(args);
        const encoded = new TextEncoder().encode(payload);

        // For native-like bindings, you can pass the string directly.
        // For WebAssembly, you should wire this to your specific ABI.
        // Here we assume enforceFn accepts a pointer to a UTF-8 JSON and length,
        // and returns a pointer to a UTF-8 JSON result.
        // Adapt this mapping to your concrete wasm bindgen/ABI.
        // The actual glue code is intentionally left ABI-neutral but concrete.

        const resultJson = enforceFn(encoded);
        const resultObj =
          typeof resultJson === "string"
            ? JSON.parse(resultJson)
            : JSON.parse(new TextDecoder().decode(resultJson));

        const trace = buildConsoleTrace(args, resultObj);
        if (typeof consoleSink === "function") {
          consoleSink(trace);
        }

        return {
          metadata: resultObj.metadata,
          decision: resultObj.decision,
          trace,
        };
      },
    };
  }

  return {
    create,
  };
})();

if (typeof module !== "undefined" && module.exports) {
  module.exports = AugDoctorBridge;
}
