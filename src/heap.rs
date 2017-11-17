//! Crudely approximating the `alloc::heap` API

use std::mem;

fn capacity<T>(size: usize) -> usize {
    let t_size = mem::size_of::<T>();
    assert!(t_size > 0);
    size.checked_add(t_size - 1).unwrap() / t_size
}


fn alloc1<T>(size: usize) -> *mut u8 {
    let cap = capacity::<T>(size);
    let mut v = Vec::<T>::with_capacity(cap);
    let p = v.as_mut_ptr();
    mem::forget(v);
    p as *mut u8
}

pub fn alloc<A, B>(size: usize, align: usize) -> *mut u8 {
    if mem::align_of::<A>() == align {
        alloc1::<A>(size)
    } else if mem::align_of::<B>() == align {
        alloc1::<B>(size)
    } else {
        panic!("invalid alignment: {}", align);
    }
}


unsafe fn dealloc1<T>(p: *mut u8, size: usize) {
    let cap = capacity::<T>(size);
    Vec::<T>::from_raw_parts(p as *mut T, 0, cap);
}

pub unsafe fn dealloc<A, B>(p: *mut u8, size: usize, align: usize) {
    if mem::align_of::<A>() == align {
        dealloc1::<A>(p, size)
    } else if mem::align_of::<B>() == align {
        dealloc1::<B>(p, size)
    } else {
        panic!("invalid alignment: {}", align);
    }
}
