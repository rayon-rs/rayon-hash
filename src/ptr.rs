// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::marker::PhantomData;
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
unsafe impl<T: Send + ?Sized> Send for Unique<T> { }

/// `Unique` pointers are `Sync` if `T` is `Sync` because the data they
/// reference is unaliased. Note that this aliasing invariant is
/// unenforced by the type system; the abstraction using the
/// `Unique` must enforce it.
unsafe impl<T: Sync + ?Sized> Sync for Unique<T> { }

impl<T: ?Sized> Unique<T> {
    /// Creates a new `Unique`.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null.
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Unique { pointer: NonZero::new_unchecked(ptr), _marker: PhantomData }
    }

    /// Acquires the underlying `*mut` pointer.
    pub fn as_ptr(self) -> *mut T {
        self.pointer.get() as *mut T
    }
}

impl<T: ?Sized> Clone for Unique<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Unique<T> { }


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
pub struct Shared<T: ?Sized> {
    pointer: NonZero<*const T>,
    // NOTE: this marker has no consequences for variance, but is necessary
    // for dropck to understand that we logically own a `T`.
    //
    // For details, see:
    // https://github.com/rust-lang/rfcs/blob/master/text/0769-sound-generic-drop.md#phantom-data
    _marker: PhantomData<T>,
}

// `Shared` pointers are not `Send` because the data they reference may be aliased.
// `Shared` pointers are not `Sync` because the data they reference may be aliased.

impl<T: ?Sized> Shared<T> {
    /// Acquires the underlying `*mut` pointer.
    pub fn as_ptr(self) -> *mut T {
        self.pointer.get() as *mut T
    }

    /// Mutably dereferences the content.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use `&mut *my_ptr.ptr_mut()`.
    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.as_ptr()
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Shared<T> { }

impl<'a, T: ?Sized> From<&'a mut T> for Shared<T> {
    fn from(reference: &'a mut T) -> Self {
        Shared { pointer: NonZero::from(reference), _marker: PhantomData }
    }
}
