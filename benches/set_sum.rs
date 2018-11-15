#![feature(test)]

extern crate rand;
extern crate rayon;
extern crate rayon_hash;
extern crate test;

use rand::distributions::Standard;
use rand::{Rng, SeedableRng, XorShiftRng};
use rayon::prelude::*;
use rayon_hash::HashSet as RayonHashSet;
use std::collections::HashSet as StdHashSet;
use std::iter::FromIterator;
use test::Bencher;

fn default_set<C: FromIterator<u32>>(n: usize) -> C {
    let mut seed = <XorShiftRng as SeedableRng>::Seed::default();
    (0..).zip(seed.as_mut()).for_each(|(i, x)| *x = i);
    XorShiftRng::from_seed(seed)
        .sample_iter(&Standard)
        .take(n)
        .collect()
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
    };
}

bench_set_sum!{std_set_sum_serial, StdHashSet<_>, iter}
bench_set_sum!{std_set_sum_parallel, StdHashSet<_>, par_iter}
bench_set_sum!{rayon_set_sum_serial, RayonHashSet<_>, iter}
bench_set_sum!{rayon_set_sum_parallel, RayonHashSet<_>, par_iter}
