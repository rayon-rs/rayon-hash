# Rayon Hash

[![rayon-hash crate](https://img.shields.io/crates/v/rayon-hash.svg)](https://crates.io/crates/rayon-hash)
[![rayon-hash documentation](https://docs.rs/rayon-hash/badge.svg)](https://docs.rs/rayon-hash)
[![Travis Status](https://travis-ci.org/rayon-rs/rayon-hash.svg?branch=master)](https://travis-ci.org/rayon-rs/rayon-hash)
![deprecated](https://img.shields.io/badge/maintenance-deprecated-red.svg)

This crate is now **deprecated**, because the [new implementation in `std`]
also exists as the [`hashbrown`] crate with its own "rayon" feature.

[new implementation in `std`]: https://github.com/rust-lang/rust/pull/58623
[`hashbrown`]: https://crates.io/crates/hashbrown

The `rayon-hash` crate duplicates the _former_ standard `HashMap` and
`HashSet`, adding native support for Rayon parallel iterators.

Rayon does provide iterators for these standard types already, but since it
can't access internal fields, it has to collect to an intermediate vector to be
split into parallel jobs.  With the custom types in `rayon-hash`, we can
instead read the raw hash table directly, for much better performance.

Benchmarks using `rustc 1.36.0-nightly (e938c2b9a 2019-04-23)`, before the
`hashbrown` implementation had merged into `std`:

```text
test hashbrown_set_sum_parallel  ... bench:     617,405 ns/iter (+/- 58,565)
test hashbrown_set_sum_serial    ... bench:   2,655,882 ns/iter (+/- 15,104)
test rayon_hash_set_sum_parallel ... bench:   1,368,058 ns/iter (+/- 75,984)
test rayon_hash_set_sum_serial   ... bench:   7,558,175 ns/iter (+/- 190,545)
test std_hash_set_sum_parallel   ... bench:   6,869,490 ns/iter (+/- 47,897)
test std_hash_set_sum_serial     ... bench:   7,591,704 ns/iter (+/- 154,438)
```

This crate currently requires `rustc 1.31.0` or greater.

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
