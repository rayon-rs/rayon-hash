#![feature(test)]

extern crate test;
extern crate rand;
extern crate rayon;
extern crate rayon_hash;

use rand::{Rng, SeedableRng, XorShiftRng};
use std::collections::HashSet as StdHashSet;
use rayon_hash::HashSet as RayonHashSet;
use std::iter::FromIterator;
use rayon::prelude::*;
use test::Bencher;


fn default_set<C: FromIterator<u32>>(n: usize) -> C {
    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    (0..n).map(|_| rng.next_u32()).collect()
}

macro_rules! bench_set_sum {
    ($id:ident, $ty:ty, $iter:ident) => {
        #[bench]
        fn $id(b: &mut Bencher) {
            let set: $ty = default_set(1024 * 1024);
            let sum: u64 = set.iter().map(|&x| x as u64).sum();

            b.iter(|| {
                let s: u64 = set.$iter().map(|&x| x as u64).sum();
                assert_eq!(s, sum);
            })
        }
    }
}

bench_set_sum!{std_set_sum_serial, StdHashSet<_>, iter}
bench_set_sum!{std_set_sum_parallel, StdHashSet<_>, par_iter}
bench_set_sum!{rayon_set_sum_serial, RayonHashSet<_>, iter}
bench_set_sum!{rayon_set_sum_parallel, RayonHashSet<_>, par_iter}
