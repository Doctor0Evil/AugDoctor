// biophysical-blockchain/src/sealed.rs
//
// Sealed trait infrastructure to ensure that only types defined inside the
// `biophysical-blockchain` crate can implement core mutation behaviors.
// This formalizes "no external extension of core mechanics".

pub(crate) mod inner {
    /// Marker trait used to seal mutation-related traits so they cannot be
    /// implemented outside this crate.
    ///
    /// Any type that implements `Sealed` must be defined in this crate.
    pub trait Sealed {}
}
