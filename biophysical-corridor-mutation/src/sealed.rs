//! Internal sealing so only this crate can implement the mutation corridor traits.

pub(crate) mod inner {
    /// Marker trait to seal corridor traits.
    pub trait Sealed {}
}
