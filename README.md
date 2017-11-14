# Rayon Hash

The `rayon-hash` crate duplicates the standard `HashMap` and `HashSet`, adding
native support for Rayon parallel iterators.

Rayon does provide iterators for these standard types already, but since it
can't access internal fields, it has to collect to an intermediate vector to be
split into parallel jobs.  With the custom types in `rayon-hash`, we can
instead read the raw hash table directly, for much better performance.
