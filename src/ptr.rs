// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ops::CoerceUnsized;
use std::fmt;
use std::marker::{PhantomData, Unsize};
use std::mem;
use nonzero::NonZero;

/// A wrapper around a raw non-null `*mut T` that indicates that the possessor
/// of this wrapper owns the referent. Useful for building abstractions like
/// `Box<T>`, `Vec<T>`, `String`, and `HashMap<K, V>`.
///
/// Unlike `*mut T`, `Unique<T>` behaves "as if" it were an instance of `T`.
/// It implements `Send`/`Sync` if `T` is `Send`/`Sync`. It also implies
/// the kind of strong aliasing guarantees an instance of `T` can expect:
/// the referent of the pointer should not be modified without a unique path to
/// its owning Unique.
///
/// If you're uncertain of whether it's correct to use `Unique` for your purposes,
/// consider using `Shared`, which has weaker semantics.
///
/// Unlike `*mut T`, the pointer must always be non-null, even if the pointer
/// is never dereferenced. This is so that enums may use this forbidden value
/// as a discriminant -- `Option<Unique<T>>` has the same size as `Unique<T>`.
/// However the pointer may still dangle if it isn't dereferenced.
///
/// Unlike `*mut T`, `Unique<T>` is covariant over `T`. This should always be correct
/// for any type which upholds Unique's aliasing requirements.
#[allow(missing_debug_implementations)]
// #[unstable(feature = "unique", reason = "needs an RFC to flesh out design",
//            issue = "27730")]
pub struct Unique<T: ?Sized> {
    pointer: NonZero<*const T>,
    // NOTE: this marker has no consequences for variance, but is necessary
    // for dropck to understand that we logically own a `T`.
    //
    // For details, see:
    // https://github.com/rust-lang/rfcs/blob/master/text/0769-sound-generic-drop.md#phantom-data
    _marker: PhantomData<T>,
}

/// `Unique` pointers are `Send` if `T` is `Send` because the data they
/// reference is unaliased. Note that this aliasing invariant is
/// unenforced by the type system; the abstraction using the
/// `Unique` must enforce it.
// #[unstable(feature = "unique", issue = "27730")]
unsafe impl<T: Send + ?Sized> Send for Unique<T> { }

/// `Unique` pointers are `Sync` if `T` is `Sync` because the data they
/// reference is unaliased. Note that this aliasing invariant is
/// unenforced by the type system; the abstraction using the
/// `Unique` must enforce it.
// #[unstable(feature = "unique", issue = "27730")]
unsafe impl<T: Sync + ?Sized> Sync for Unique<T> { }

// #[unstable(feature = "unique", issue = "27730")]
impl<T: Sized> Unique<T> {
    /// Creates a new `Unique` that is dangling, but well-aligned.
    ///
    /// This is useful for initializing types which lazily allocate, like
    /// `Vec::new` does.
    pub fn empty() -> Self {
        unsafe {
            let ptr = mem::align_of::<T>() as *mut T;
            Unique::new_unchecked(ptr)
        }
    }
}

// #[unstable(feature = "unique", issue = "27730")]
impl<T: ?Sized> Unique<T> {
    /// Creates a new `Unique`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null.
    // #[unstable(feature = "unique", issue = "27730")]
    // #[rustc_const_unstable(feature = "const_unique_new")]
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Unique { pointer: NonZero::new_unchecked(ptr), _marker: PhantomData }
    }

    /// Creates a new `Unique` if `ptr` is non-null.
    pub fn new(ptr: *mut T) -> Option<Self> {
        NonZero::new(ptr as *const T).map(|nz| Unique { pointer: nz, _marker: PhantomData })
    }

    /// Acquires the underlying `*mut` pointer.
    pub fn as_ptr(self) -> *mut T {
        self.pointer.get() as *mut T
    }

    /// Dereferences the content.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use `&*my_ptr.ptr()`.
    pub unsafe fn as_ref(&self) -> &T {
        &*self.as_ptr()
    }

    /// Mutably dereferences the content.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use `&mut *my_ptr.ptr()`.
    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.as_ptr()
    }
}

// #[unstable(feature = "unique", issue = "27730")]
impl<T: ?Sized> Clone for Unique<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// #[unstable(feature = "unique", issue = "27730")]
impl<T: ?Sized> Copy for Unique<T> { }

// #[unstable(feature = "unique", issue = "27730")]
impl<T: ?Sized, U: ?Sized> CoerceUnsized<Unique<U>> for Unique<T> where T: Unsize<U> { }

// #[unstable(feature = "unique", issue = "27730")]
impl<T: ?Sized> fmt::Pointer for Unique<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

// #[unstable(feature = "unique", issue = "27730")]
impl<'a, T: ?Sized> From<&'a mut T> for Unique<T> {
    fn from(reference: &'a mut T) -> Self {
        Unique { pointer: NonZero::from(reference), _marker: PhantomData }
    }
}

// #[unstable(feature = "unique", issue = "27730")]
impl<'a, T: ?Sized> From<&'a T> for Unique<T> {
    fn from(reference: &'a T) -> Self {
        Unique { pointer: NonZero::from(reference), _marker: PhantomData }
    }
}

/// A wrapper around a raw `*mut T` that indicates that the possessor
/// of this wrapper has shared ownership of the referent. Useful for
/// building abstractions like `Rc<T>`, `Arc<T>`, or doubly-linked lists, which
/// internally use aliased raw pointers to manage the memory that they own.
///
/// This is similar to `Unique`, except that it doesn't make any aliasing
/// guarantees, and doesn't derive Send and Sync. Note that unlike `&T`,
/// Shared has no special mutability requirements. Shared may mutate data
/// aliased by other Shared pointers. More precise rules require Rust to
/// develop an actual aliasing model.
///
/// Unlike `*mut T`, the pointer must always be non-null, even if the pointer
/// is never dereferenced. This is so that enums may use this forbidden value
/// as a discriminant -- `Option<Shared<T>>` has the same size as `Shared<T>`.
/// However the pointer may still dangle if it isn't dereferenced.
///
/// Unlike `*mut T`, `Shared<T>` is covariant over `T`. If this is incorrect
/// for your use case, you should include some PhantomData in your type to
/// provide invariance, such as `PhantomData<Cell<T>>` or `PhantomData<&'a mut T>`.
/// Usually this won't be necessary; covariance is correct for Rc, Arc, and LinkedList
/// because they provide a public API that follows the normal shared XOR mutable
/// rules of Rust.
#[allow(missing_debug_implementations)]
// #[unstable(feature = "shared", reason = "needs an RFC to flesh out design",
//            issue = "27730")]
pub struct Shared<T: ?Sized> {
    pointer: NonZero<*const T>,
    // NOTE: this marker has no consequences for variance, but is necessary
    // for dropck to understand that we logically own a `T`.
    //
    // For details, see:
    // https://github.com/rust-lang/rfcs/blob/master/text/0769-sound-generic-drop.md#phantom-data
    _marker: PhantomData<T>,
}

/// `Shared` pointers are not `Send` because the data they reference may be aliased.
// NB: This impl is unnecessary, but should provide better error messages.
// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized> !Send for Shared<T> { }

/// `Shared` pointers are not `Sync` because the data they reference may be aliased.
// NB: This impl is unnecessary, but should provide better error messages.
// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized> !Sync for Shared<T> { }

// #[unstable(feature = "shared", issue = "27730")]
impl<T: Sized> Shared<T> {
    /// Creates a new `Shared` that is dangling, but well-aligned.
    ///
    /// This is useful for initializing types which lazily allocate, like
    /// `Vec::new` does.
    pub fn empty() -> Self {
        unsafe {
            let ptr = mem::align_of::<T>() as *mut T;
            Shared::new_unchecked(ptr)
        }
    }
}

// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized> Shared<T> {
    /// Creates a new `Shared`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null.
    // #[unstable(feature = "shared", issue = "27730")]
    // #[rustc_const_unstable(feature = "const_shared_new")]
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Shared { pointer: NonZero::new_unchecked(ptr), _marker: PhantomData }
    }

    /// Creates a new `Shared` if `ptr` is non-null.
    pub fn new(ptr: *mut T) -> Option<Self> {
        NonZero::new(ptr as *const T).map(|nz| Shared { pointer: nz, _marker: PhantomData })
    }

    /// Acquires the underlying `*mut` pointer.
    pub fn as_ptr(self) -> *mut T {
        self.pointer.get() as *mut T
    }

    /// Dereferences the content.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use `&*my_ptr.ptr()`.
    pub unsafe fn as_ref(&self) -> &T {
        &*self.as_ptr()
    }

    /// Mutably dereferences the content.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use `&mut *my_ptr.ptr_mut()`.
    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.as_ptr()
    }

    /// Acquires the underlying pointer as a `*mut` pointer.
    // #[rustc_deprecated(since = "1.19", reason = "renamed to `as_ptr` for ergonomics/consistency")]
    // #[unstable(feature = "shared", issue = "27730")]
    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        self.as_ptr()
    }
}

// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized> Copy for Shared<T> { }

// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized, U: ?Sized> CoerceUnsized<Shared<U>> for Shared<T> where T: Unsize<U> { }

// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized> fmt::Pointer for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.as_ptr(), f)
    }
}

// #[unstable(feature = "shared", issue = "27730")]
impl<T: ?Sized> From<Unique<T>> for Shared<T> {
    fn from(unique: Unique<T>) -> Self {
        Shared { pointer: unique.pointer, _marker: PhantomData }
    }
}

// #[unstable(feature = "shared", issue = "27730")]
impl<'a, T: ?Sized> From<&'a mut T> for Shared<T> {
    fn from(reference: &'a mut T) -> Self {
        Shared { pointer: NonZero::from(reference), _marker: PhantomData }
    }
}

// #[unstable(feature = "shared", issue = "27730")]
impl<'a, T: ?Sized> From<&'a T> for Shared<T> {
    fn from(reference: &'a T) -> Self {
        Shared { pointer: NonZero::from(reference), _marker: PhantomData }
    }
}
