extern crate rayon_hash;

use std::mem::size_of;

fn main() {
    // With a proper `NonZero` implementation, `Option` takes no extra space.
    println!("size_of::<std::collections::HashSet<i32, i32>>() -> {}",
              size_of::<std::collections::HashSet<i32, i32>>());
    println!("size_of::<Option<std::collections::HashSet<i32, i32>>>() -> {}",
              size_of::<Option<std::collections::HashSet<i32, i32>>>());

    // With stable rust, our `NonZero` doesn't do anything.
    println!("size_of::<rayon_hash::HashSet<i32, i32>>() -> {}",
              size_of::<rayon_hash::HashSet<i32, i32>>());
    println!("size_of::<Option<rayon_hash::HashSet<i32, i32>>>() -> {}",
              size_of::<Option<rayon_hash::HashSet<i32, i32>>>());
}
