# Rayon Hash

[![rayon-hash crate](https://img.shields.io/crates/v/rayon-hash.svg)](https://crates.io/crates/rayon-hash)
[![rayon-hash documentation](https://docs.rs/rayon-hash/badge.svg)](https://docs.rs/rayon-hash)
[![Travis Status](https://travis-ci.org/rayon-rs/rayon-hash.svg?branch=master)](https://travis-ci.org/rayon-rs/rayon-hash)

The `rayon-hash` crate duplicates the standard `HashMap` and `HashSet`, adding
native support for Rayon parallel iterators.

Rayon does provide iterators for these standard types already, but since it
can't access internal fields, it has to collect to an intermediate vector to be
split into parallel jobs.  With the custom types in `rayon-hash`, we can
instead read the raw hash table directly, for much better performance.

```text
test rayon_set_sum_parallel ... bench:   1,077,602 ns/iter (+/- 50,610)
test rayon_set_sum_serial   ... bench:   6,363,125 ns/iter (+/- 101,513)
test std_set_sum_parallel   ... bench:   8,519,683 ns/iter (+/- 219,785)
test std_set_sum_serial     ... bench:   6,295,263 ns/iter (+/- 98,600)
```

This crate currently requires `rustc 1.28.0` or greater.

## Known limitations

Some compromises may be made to let this work on stable Rust, compared to the
standard types that may use unstable features.  There is an example included
which demonstrates one difference.

- [`examples/may_dangle.rs`](examples/may_dangle.rs): Since we don't use the
  unstable `#[may_dangle]` attributes, the type parameters of `HashMap<K, V>`
  and `HashSet<T>` must strictly outlive the container itself.

## Unstable features

Some of the features copied from `std` would be guarded with `#[unstable]`
attributes, but this isn't available to general crates.  Instead, we guard
these features with a config flag `rayon_hash_unstable`.  The easiest way to
use this is to set the `RUSTFLAGS` environment variable:

```
RUSTFLAGS='--cfg rayon_hash_unstable' cargo build
```

Note that this must not only be done for your crate, but for any crate that
depends on your crate.  This infectious nature is intentional, as it serves as
a reminder that you are outside of the normal semver guarantees.  These
features may also require a nightly Rust compiler.

When such features are stabilized in the standard library, we will remove the
`rayon_hash_unstable` guard here too.

## License

Rayon-hash is distributed under the terms of both the MIT license and the
Apache License (Version 2.0). See [LICENSE-APACHE](LICENSE-APACHE) and
[LICENSE-MIT](LICENSE-MIT) for details. Opening a pull requests is
assumed to signal agreement with these licensing terms.
