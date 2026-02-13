pub struct EnvelopeCtx {
    // existing fields…
    has_downgrade_cap: bool,
}

impl EnvelopeCtx {
    /// Only inner-ledger core can create contexts.
    pub(crate) fn new(/* args */) -> Self {
        Self {
            // …
            has_downgrade_cap: false,
        }
    }

    /// Called *only* from host-local, self-consented logic.
    pub fn request_capability_downgrade(&mut self) {
        // This method itself must be gated by:
        // - host DID == subject of ledger
        // - valid DemonstratedConsentShard for downgrade path
        // Enforcement happens in the inner-ledger runtime, not here.
        self.has_downgrade_cap = true;
    }

    pub fn can_downgrade(&self) -> bool {
        self.has_downgrade_cap
    }
}
