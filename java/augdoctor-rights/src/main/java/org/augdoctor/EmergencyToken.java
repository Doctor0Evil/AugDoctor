package org.augdoctor;

/**
 * Emergency token explicitly authored by the host to allow one-time
 * rollback or downgrade in a true emergency. This must be created,
 * signed, and stored only by you; control-plane code treats it as
 * opaque and never forges or auto-creates it.
 */
public record EmergencyToken(
        String scopeId,          // e.g. "evolution-emergency-rollback-v1"
        String transcriptHash,   // hash of the chat / consent text
        String issuedAtUtc,      // ISO-8601
        String signatureHex      // signature under your Bostrom key
) {}
