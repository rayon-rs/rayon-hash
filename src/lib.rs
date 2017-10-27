#![feature(alloc)]
#![feature(allocator_api)]
#![feature(dropck_eyepatch)]
#![feature(fused)]
#![feature(generic_param_attrs)]
#![feature(placement_new_protocol)]
#![feature(shared)]
#![feature(sip_hash_13)]
#![feature(unique)]

#![cfg_attr(test, feature(placement_in_syntax))]
#![cfg_attr(test, feature(test))]

extern crate alloc;
extern crate rand;
extern crate rayon;

use std::borrow;
use std::cell;
use std::cmp;
use std::fmt;
use std::hash;
use std::iter;
use std::marker;
use std::mem;
use std::ops;
use std::ptr;

#[cfg(test)] use std::panic;

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
