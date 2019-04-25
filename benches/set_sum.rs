#![feature(test)]

extern crate rand;
extern crate rand_xorshift;
extern crate rayon;
extern crate rayon_hash;
extern crate test;

use hashbrown::HashSet as HashBrownSet;
use rand::distributions::Standard;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use rayon::prelude::*;
use rayon_hash::HashSet as RayonHashSet;
use std::collections::HashSet as StdHashSet;
use std::collections::hash_map::RandomState;
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

bench_set_sum!{std_hash_set_sum_serial, StdHashSet<_, RandomState>, iter}
bench_set_sum!{std_hash_set_sum_parallel, StdHashSet<_, RandomState>, par_iter}
bench_set_sum!{rayon_hash_set_sum_serial, RayonHashSet<_, RandomState>, iter}
bench_set_sum!{rayon_hash_set_sum_parallel, RayonHashSet<_, RandomState>, par_iter}
bench_set_sum!{hashbrown_set_sum_serial, HashBrownSet<_, RandomState>, iter}
bench_set_sum!{hashbrown_set_sum_parallel, HashBrownSet<_, RandomState>, par_iter}
