extern crate rayon_hash;

fn main() {
    // The standard collections can have type parameters with
    // the same lifetime as the collection itself:
    let (x, mut set) = (0, std::collections::HashSet::new());
    set.insert(&x);

    // FIXME: We can't do this without `#[may_dangle]` on `Drop for RawTable`:
    // let (x, mut set) = (0, rayon_hash::HashSet::new());
    // set.insert(&x);

    // Instead, we our type parameters must strictly outlive the collection.
    let x = 0;
    let mut set = rayon_hash::HashSet::new();
    set.insert(&x);
}
