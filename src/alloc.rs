#[cfg(rayon_hash_unstable)] pub use std::alloc::oom;
#[cfg(not(rayon_hash_unstable))] pub use std::process::abort as oom;

/// Augments `AllocErr` with a CapacityOverflow variant.
#[derive(Clone, PartialEq, Eq, Debug)]
// #[unstable(feature = "try_reserve", reason = "new API", issue="48043")]
pub enum CollectionAllocErr {
    /// Error due to the computed capacity exceeding the collection's maximum
    /// (usually `isize::MAX` bytes).
    CapacityOverflow,
    /// Error due to the allocator (see the `AllocErr` type's docs).
    AllocErr,
}
