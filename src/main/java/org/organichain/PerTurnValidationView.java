package org.organichain;

import java.util.Map;

public interface PerTurnValidationView {
    // Map from action id to {kind, messages}, exported over FFI or gRPC.
    Map<String, ActionValidationStatus> getLastTurnValidationReport();

    final class ActionValidationStatus {
        public final String actionId;      // e.g., "action.intent-detection"
        public final String resultKind;    // "Passed" | "Failed" | "Skipped"
        public final String[] messages;

        public ActionValidationStatus(String actionId, String resultKind, String[] messages) {
            this.actionId = actionId;
            this.resultKind = resultKind;
            this.messages = messages;
        }
    }
}
