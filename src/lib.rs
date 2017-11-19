#![cfg_attr(rayon_hash_unstable, feature(fused))]
#![cfg_attr(rayon_hash_unstable, feature(placement_new_protocol))]

#![cfg_attr(all(rayon_hash_unstable, test), feature(placement_in_syntax))]
#![cfg_attr(test, feature(test))]

extern crate rayon;

#[cfg(test)] extern crate rand;

mod heap;
mod nonzero;
mod ptr;

#[cfg(all(rayon_hash_unstable, test))] use std::panic;
#[cfg(test)] use std::cell;

// #[stable(feature = "rust1", since = "1.0.0")]
pub use self::hash_map::HashMap;
// #[stable(feature = "rust1", since = "1.0.0")]
pub use self::hash_set::HashSet;

mod std_hash;

// #[stable(feature = "rust1", since = "1.0.0")]
pub mod hash_map {
    //! A hash map implemented with linear probing and Robin Hood bucket stealing.
    // #[stable(feature = "rust1", since = "1.0.0")]
    pub use super::std_hash::map::*;
}

// #[stable(feature = "rust1", since = "1.0.0")]
pub mod hash_set {
    //! A hash set implemented as a `HashMap` where the value is `()`.
    // #[stable(feature = "rust1", since = "1.0.0")]
    pub use super::std_hash::set::*;
}
