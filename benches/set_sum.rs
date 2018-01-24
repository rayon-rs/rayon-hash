#![feature(test)]

extern crate fnv;
extern crate rand;
extern crate rayon;
extern crate rayon_hash;
extern crate test;

use rand::{Rng, SeedableRng, XorShiftRng};
use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};
use rayon_hash::{HashMap as RayonHashMap, HashSet as RayonHashSet};
use std::iter::FromIterator;
use rayon::prelude::*;
use test::Bencher;
use fnv::FnvBuildHasher;


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

macro_rules! bench_collect {
    ($id:ident, $ty:ty, $iter:ident) => {
        #[bench]
        fn $id(b: &mut Bencher) {
            b.iter(|| {
                let set: $ty = (0u32 .. 1<<20).$iter().map(|x| (x >> 1, ())).collect();
                assert_eq!(1<<19, set.len());
            })
        }
    }
}

bench_collect!{std_collect_serial, StdHashMap<_, _>, into_iter}
bench_collect!{std_collect_parallel, StdHashMap<_, _>, into_par_iter}
bench_collect!{rayon_collect_serial, RayonHashMap<_, _>, into_iter}
bench_collect!{rayon_collect_parallel, RayonHashMap<_, _>, into_par_iter}

bench_collect!{rayon_collect_fnv, RayonHashMap<_, _, FnvBuildHasher>, into_iter}
bench_collect!{std_collect_fnv, StdHashMap<_, _, FnvBuildHasher>, into_iter}