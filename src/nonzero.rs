// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Exposes the NonZero lang item which provides optimization hints.
//! (but not really, because now we're in a non-core stable crate...)

/// Unsafe trait to indicate what types are usable with the NonZero struct
pub unsafe trait Zeroable {}

unsafe impl<T: ?Sized> Zeroable for *const T {}
unsafe impl<T: ?Sized> Zeroable for *mut T {}

/// A wrapper type for raw pointers and integers that will never be
/// NULL or 0 that might allow certain optimizations.
/// (but not really, because now we're in a non-core stable crate...)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct NonZero<T: Zeroable>(T);

impl<T: Zeroable> NonZero<T> {
    /// Creates an instance of NonZero with the provided value.
    /// You must indeed ensure that the value is actually "non-zero".
    #[inline]
    pub unsafe fn new_unchecked(inner: T) -> Self {
        NonZero(inner)
    }

    /// Gets the inner value.
    pub fn get(self) -> T {
        self.0
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for NonZero<*mut T> {
    fn from(reference: &'a mut T) -> Self {
        NonZero(reference)
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for NonZero<*const T> {
    fn from(reference: &'a mut T) -> Self {
        let ptr: *mut T = reference;
        NonZero(ptr)
    }
}

impl<'a, T: ?Sized> From<&'a T> for NonZero<*const T> {
    fn from(reference: &'a T) -> Self {
        NonZero(reference)
    }
}
