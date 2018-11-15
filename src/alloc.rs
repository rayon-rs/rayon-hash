// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Memory allocation APIs (forked from `libcore/alloc.rs`)

// #![stable(feature = "alloc_module", since = "1.28.0")]

use std::cmp;
use std::mem;
use std::usize;
use std::alloc::{self, LayoutErr};

#[inline]
fn layout_err() -> LayoutErr {
    alloc::Layout::from_size_align(usize::MAX, usize::MAX).unwrap_err()
}

/// Layout of a block of memory.
///
/// An instance of `Layout` describes a particular layout of memory.
/// You build a `Layout` up as an input to give to an allocator.
///
/// All layouts have an associated non-negative size and a
/// power-of-two alignment.
///
/// (Note however that layouts are *not* required to have positive
/// size, even though many allocators require that all memory
/// requests have positive size. A caller to the `Alloc::alloc`
/// method must either ensure that conditions like this are met, or
/// use specific allocators with looser requirements.)
// #[stable(feature = "alloc_layout", since = "1.28.0")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
// #[lang = "alloc_layout"]
pub(crate) struct Layout {
    inner: alloc::Layout,
}

impl Layout {
    /// Constructs a `Layout` from a given `size` and `align`,
    /// or returns `LayoutErr` if either of the following conditions
    /// are not met:
    ///
    /// * `align` must not be zero,
    ///
    /// * `align` must be a power of two,
    ///
    /// * `size`, when rounded up to the nearest multiple of `align`,
    ///    must not overflow (i.e. the rounded value must be less than
    ///    `usize::MAX`).
    // #[stable(feature = "alloc_layout", since = "1.28.0")]
    #[inline]
    pub(crate) fn from_size_align(size: usize, align: usize) -> Result<Self, LayoutErr> {
        alloc::Layout::from_size_align(size, align).map(Layout::from)
    }

    /// Creates a layout, bypassing all checks.
    ///
    /// # Safety
    ///
    /// This function is unsafe as it does not verify the preconditions from
    /// [`Layout::from_size_align`](#method.from_size_align).
    // #[stable(feature = "alloc_layout", since = "1.28.0")]
    #[inline]
    pub(crate) unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Self {
        Layout::from(alloc::Layout::from_size_align_unchecked(size, align))
    }

    /// The minimum size in bytes for a memory block of this layout.
    // #[stable(feature = "alloc_layout", since = "1.28.0")]
    #[inline]
    pub(crate) fn size(&self) -> usize { self.inner.size() }

    /// The minimum byte alignment for a memory block of this layout.
    // #[stable(feature = "alloc_layout", since = "1.28.0")]
    #[inline]
    pub(crate) fn align(&self) -> usize { self.inner.align() }

    /// Constructs a `Layout` suitable for holding a value of type `T`.
    // #[stable(feature = "alloc_layout", since = "1.28.0")]
    #[inline]
    pub(crate) fn new<T>() -> Self {
        Layout::from(alloc::Layout::new::<T>())
    }

    /// Returns the amount of padding we must insert after `self`
    /// to ensure that the following address will satisfy `align`
    /// (measured in bytes).
    ///
    /// E.g. if `self.size()` is 9, then `self.padding_needed_for(4)`
    /// returns 3, because that is the minimum number of bytes of
    /// padding required to get a 4-aligned address (assuming that the
    /// corresponding memory block starts at a 4-aligned address).
    ///
    /// The return value of this function has no meaning if `align` is
    /// not a power-of-two.
    ///
    /// Note that the utility of the returned value requires `align`
    /// to be less than or equal to the alignment of the starting
    /// address for the whole allocated block of memory. One way to
    /// satisfy this constraint is to ensure `align <= self.align()`.
    // #[unstable(feature = "alloc_layout_extra", issue = "55724")]
    #[inline]
    pub(crate) fn padding_needed_for(&self, align: usize) -> usize {
        let len = self.size();

        // Rounded up value is:
        //   len_rounded_up = (len + align - 1) & !(align - 1);
        // and then we return the padding difference: `len_rounded_up - len`.
        //
        // We use modular arithmetic throughout:
        //
        // 1. align is guaranteed to be > 0, so align - 1 is always
        //    valid.
        //
        // 2. `len + align - 1` can overflow by at most `align - 1`,
        //    so the &-mask wth `!(align - 1)` will ensure that in the
        //    case of overflow, `len_rounded_up` will itself be 0.
        //    Thus the returned padding, when added to `len`, yields 0,
        //    which trivially satisfies the alignment `align`.
        //
        // (Of course, attempts to allocate blocks of memory whose
        // size and padding overflow in the above manner should cause
        // the allocator to yield an error anyway.)

        let len_rounded_up = len.wrapping_add(align).wrapping_sub(1)
            & !align.wrapping_sub(1);
        len_rounded_up.wrapping_sub(len)
    }

    /// Creates a layout describing the record for `n` instances of
    /// `self`, with a suitable amount of padding between each to
    /// ensure that each instance is given its requested size and
    /// alignment. On success, returns `(k, offs)` where `k` is the
    /// layout of the array and `offs` is the distance between the start
    /// of each element in the array.
    ///
    /// On arithmetic overflow, returns `LayoutErr`.
    // #[unstable(feature = "alloc_layout_extra", issue = "55724")]
    #[inline]
    pub(crate) fn repeat(&self, n: usize) -> Result<(Self, usize), LayoutErr> {
        let padded_size = self.size().checked_add(self.padding_needed_for(self.align()))
            .ok_or(layout_err())?;
        let alloc_size = padded_size.checked_mul(n)
            .ok_or(layout_err())?;

        unsafe {
            // self.align is already known to be valid and alloc_size has been
            // padded already.
            Ok((Layout::from_size_align_unchecked(alloc_size, self.align()), padded_size))
        }
    }

    /// Creates a layout describing the record for `self` followed by
    /// `next`, including any necessary padding to ensure that `next`
    /// will be properly aligned. Note that the result layout will
    /// satisfy the alignment properties of both `self` and `next`.
    ///
    /// The resulting layout will be the same as that of a C struct containing
    /// two fields with the layouts of `self` and `next`, in that order.
    ///
    /// Returns `Some((k, offset))`, where `k` is layout of the concatenated
    /// record and `offset` is the relative location, in bytes, of the
    /// start of the `next` embedded within the concatenated record
    /// (assuming that the record itself starts at offset 0).
    ///
    /// On arithmetic overflow, returns `LayoutErr`.
    // #[unstable(feature = "alloc_layout_extra", issue = "55724")]
    #[inline]
    pub(crate) fn extend(&self, next: Self) -> Result<(Self, usize), LayoutErr> {
        let new_align = cmp::max(self.align(), next.align());
        let pad = self.padding_needed_for(next.align());

        let offset = self.size().checked_add(pad)
            .ok_or(layout_err())?;
        let new_size = offset.checked_add(next.size())
            .ok_or(layout_err())?;

        let layout = Layout::from_size_align(new_size, new_align)?;
        Ok((layout, offset))
    }

    /// Creates a layout describing the record for a `[T; n]`.
    ///
    /// On arithmetic overflow, returns `LayoutErr`.
    // #[unstable(feature = "alloc_layout_extra", issue = "55724")]
    #[inline]
    pub(crate) fn array<T>(n: usize) -> Result<Self, LayoutErr> {
        Layout::new::<T>()
            .repeat(n)
            .map(|(k, offs)| {
                debug_assert!(offs == mem::size_of::<T>());
                k
            })
    }
}

impl From<alloc::Layout> for Layout {
    #[inline]
    fn from(inner: alloc::Layout) -> Layout {
        Layout { inner }
    }
}

impl From<Layout> for alloc::Layout {
    #[inline]
    fn from(layout: Layout) -> alloc::Layout {
        layout.inner
    }
}

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

// #[unstable(feature = "try_reserve", reason = "new API", issue="48043")]
impl From<LayoutErr> for CollectionAllocErr {
    #[inline]
    fn from(_: LayoutErr) -> Self {
        CollectionAllocErr::CapacityOverflow
    }
}
