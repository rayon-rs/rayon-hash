/// Rayon extensions for `HashSet`

use rayon::iter::{ParallelIterator, IntoParallelIterator, FromParallelIterator};
use rayon::iter::plumbing::UnindexedConsumer;

use super::{Hash, HashSet, BuildHasher, map};


pub struct ParIntoIter<T: Send> {
    inner: map::ParIntoIter<T, ()>,
}

pub struct ParIter<'a, T: Sync + 'a> {
    inner: map::ParKeys<'a, T, ()>,
}

pub struct ParDifference<'a, T: Sync + 'a, S: Sync + 'a> {
    a: &'a HashSet<T, S>,
    b: &'a HashSet<T, S>,
}

pub struct ParSymmetricDifference<'a, T: Sync + 'a, S: Sync + 'a> {
    a: &'a HashSet<T, S>,
    b: &'a HashSet<T, S>,
}

pub struct ParIntersection<'a, T: Sync + 'a, S: Sync + 'a> {
    a: &'a HashSet<T, S>,
    b: &'a HashSet<T, S>,
}

pub struct ParUnion<'a, T: Sync + 'a, S: Sync + 'a> {
    a: &'a HashSet<T, S>,
    b: &'a HashSet<T, S>,
}


impl<T, S> HashSet<T, S>
    where T: Eq + Hash + Sync,
          S: BuildHasher + Sync
{
    pub fn par_difference<'a>(&'a self, other: &'a Self) -> ParDifference<'a, T, S> {
        ParDifference {
            a: self,
            b: other,
        }
    }

    pub fn par_symmetric_difference<'a>(&'a self,
                                        other: &'a Self)
                                        -> ParSymmetricDifference<'a, T, S> {
        ParSymmetricDifference {
            a: self,
            b: other,
        }
    }

    pub fn par_intersection<'a>(&'a self, other: &'a Self) -> ParIntersection<'a, T, S> {
        ParIntersection {
            a: self,
            b: other,
        }
    }

    pub fn par_union<'a>(&'a self, other: &'a Self) -> ParUnion<'a, T, S> {
        ParUnion {
            a: self,
            b: other,
        }
    }

    pub fn par_is_disjoint(&self, other: &Self) -> bool {
        self.into_par_iter().all(|x| !other.contains(x))
    }

    pub fn par_is_subset(&self, other: &Self) -> bool {
        self.into_par_iter().all(|x| other.contains(x))
    }

    pub fn par_is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    }

    pub fn par_eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.par_is_subset(other)
    }
}


impl<T: Send, S> IntoParallelIterator for HashSet<T, S> {
    type Item = T;
    type Iter = ParIntoIter<T>;

    fn into_par_iter(self) -> Self::Iter {
        ParIntoIter { inner: self.map.into_par_iter() }
    }
}

impl<'a, T: Sync, S> IntoParallelIterator for &'a HashSet<T, S> {
    type Item = &'a T;
    type Iter = ParIter<'a, T>;

    fn into_par_iter(self) -> Self::Iter {
        ParIter { inner: self.map.par_keys() }
    }
}


// This is equal to the normal `HashSet` -- no custom advantage.
impl<T, S> FromParallelIterator<T> for HashSet<T, S>
    where T: Eq + Hash + Send,
          S: BuildHasher + Default + Send
{
    fn from_par_iter<P>(par_iter: P) -> Self
        where P: IntoParallelIterator<Item = T>
    {
        use std::collections::LinkedList;

        let list: LinkedList<_> = par_iter.into_par_iter()
            .fold(Vec::new, |mut vec, elem| {
                vec.push(elem);
                vec
            })
            .collect();

        let len = list.iter().map(Vec::len).sum();
        let start = HashSet::with_capacity_and_hasher(len, Default::default());
        list.into_iter().fold(start, |mut coll, vec| {
            coll.extend(vec);
            coll
        })
    }
}


impl<T: Send> ParallelIterator for ParIntoIter<T> {
    type Item = T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.map(|(k, _)| k).drive_unindexed(consumer)
    }
}


impl<'a, T: Sync> ParallelIterator for ParIter<'a, T> {
    type Item = &'a T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.drive_unindexed(consumer)
    }
}


impl<'a, T, S> ParallelIterator for ParDifference<'a, T, S>
    where T: Eq + Hash + Sync,
          S: BuildHasher + Sync
{
    type Item = &'a T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.a
            .into_par_iter()
            .filter(|&x| !self.b.contains(x))
            .drive_unindexed(consumer)
    }
}


impl<'a, T, S> ParallelIterator for ParSymmetricDifference<'a, T, S>
    where T: Eq + Hash + Sync,
          S: BuildHasher + Sync
{
    type Item = &'a T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.a
            .par_difference(self.b)
            .chain(self.b.par_difference(self.a))
            .drive_unindexed(consumer)
    }
}


impl<'a, T, S> ParallelIterator for ParIntersection<'a, T, S>
    where T: Eq + Hash + Sync,
          S: BuildHasher + Sync
{
    type Item = &'a T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.a
            .into_par_iter()
            .filter(|&x| self.b.contains(x))
            .drive_unindexed(consumer)
    }
}


impl<'a, T, S> ParallelIterator for ParUnion<'a, T, S>
    where T: Eq + Hash + Sync,
          S: BuildHasher + Sync
{
    type Item = &'a T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.a
            .into_par_iter()
            .chain(self.b.par_difference(self.a))
            .drive_unindexed(consumer)
    }
}


#[cfg(test)]
mod test_par_set {
    use super::HashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use rayon::prelude::*;

    #[test]
    fn test_disjoint() {
        let mut xs = HashSet::new();
        let mut ys = HashSet::new();
        assert!(xs.par_is_disjoint(&ys));
        assert!(ys.par_is_disjoint(&xs));
        assert!(xs.insert(5));
        assert!(ys.insert(11));
        assert!(xs.par_is_disjoint(&ys));
        assert!(ys.par_is_disjoint(&xs));
        assert!(xs.insert(7));
        assert!(xs.insert(19));
        assert!(xs.insert(4));
        assert!(ys.insert(2));
        assert!(ys.insert(-11));
        assert!(xs.par_is_disjoint(&ys));
        assert!(ys.par_is_disjoint(&xs));
        assert!(ys.insert(7));
        assert!(!xs.par_is_disjoint(&ys));
        assert!(!ys.par_is_disjoint(&xs));
    }

    #[test]
    fn test_subset_and_superset() {
        let mut a = HashSet::new();
        assert!(a.insert(0));
        assert!(a.insert(5));
        assert!(a.insert(11));
        assert!(a.insert(7));

        let mut b = HashSet::new();
        assert!(b.insert(0));
        assert!(b.insert(7));
        assert!(b.insert(19));
        assert!(b.insert(250));
        assert!(b.insert(11));
        assert!(b.insert(200));

        assert!(!a.par_is_subset(&b));
        assert!(!a.par_is_superset(&b));
        assert!(!b.par_is_subset(&a));
        assert!(!b.par_is_superset(&a));

        assert!(b.insert(5));

        assert!(a.par_is_subset(&b));
        assert!(!a.par_is_superset(&b));
        assert!(!b.par_is_subset(&a));
        assert!(b.par_is_superset(&a));
    }

    #[test]
    fn test_iterate() {
        let mut a = HashSet::new();
        for i in 0..32 {
            assert!(a.insert(i));
        }
        let observed = AtomicUsize::new(0);
        a.par_iter().for_each(|k| {
            observed.fetch_or(1 << *k, Ordering::Relaxed);
        });
        assert_eq!(observed.into_inner(), 0xFFFF_FFFF);
    }

    #[test]
    fn test_intersection() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(11));
        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(77));
        assert!(a.insert(103));
        assert!(a.insert(5));
        assert!(a.insert(-5));

        assert!(b.insert(2));
        assert!(b.insert(11));
        assert!(b.insert(77));
        assert!(b.insert(-9));
        assert!(b.insert(-42));
        assert!(b.insert(5));
        assert!(b.insert(3));

        let expected = [3, 5, 11, 77];
        let i = a.par_intersection(&b).map(|x| {
            assert!(expected.contains(x));
            1
        }).sum::<usize>();
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_difference() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));

        assert!(b.insert(3));
        assert!(b.insert(9));

        let expected = [1, 5, 11];
        let i = a.par_difference(&b).map(|x| {
            assert!(expected.contains(x));
            1
        }).sum::<usize>();
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_symmetric_difference() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));

        assert!(b.insert(-2));
        assert!(b.insert(3));
        assert!(b.insert(9));
        assert!(b.insert(14));
        assert!(b.insert(22));

        let expected = [-2, 1, 5, 11, 14, 22];
        let i = a.par_symmetric_difference(&b).map(|x| {
            assert!(expected.contains(x));
            1
        }).sum::<usize>();
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_union() {
        let mut a = HashSet::new();
        let mut b = HashSet::new();

        assert!(a.insert(1));
        assert!(a.insert(3));
        assert!(a.insert(5));
        assert!(a.insert(9));
        assert!(a.insert(11));
        assert!(a.insert(16));
        assert!(a.insert(19));
        assert!(a.insert(24));

        assert!(b.insert(-2));
        assert!(b.insert(1));
        assert!(b.insert(5));
        assert!(b.insert(9));
        assert!(b.insert(13));
        assert!(b.insert(19));

        let expected = [-2, 1, 3, 5, 9, 11, 13, 16, 19, 24];
        let i = a.par_union(&b).map(|x| {
            assert!(expected.contains(x));
            1
        }).sum::<usize>();
        assert_eq!(i, expected.len());
    }

    #[test]
    fn test_from_iter() {
        let xs = [1, 2, 3, 4, 5, 6, 7, 8, 9];

        let set: HashSet<_> = xs.iter().cloned().collect();

        for x in &xs {
            assert!(set.contains(x));
        }
    }

    #[test]
    fn test_move_iter() {
        let hs = {
            let mut hs = HashSet::new();

            hs.insert('a');
            hs.insert('b');

            hs
        };

        let v = hs.into_par_iter().collect::<Vec<char>>();
        assert!(v == ['a', 'b'] || v == ['b', 'a']);
    }

    #[test]
    fn test_eq() {
        // These constants once happened to expose a bug in insert().
        // I'm keeping them around to prevent a regression.
        let mut s1 = HashSet::new();

        s1.insert(1);
        s1.insert(2);
        s1.insert(3);

        let mut s2 = HashSet::new();

        s2.insert(1);
        s2.insert(2);

        assert!(!s1.par_eq(&s2));

        s2.insert(3);

        assert!(s1.par_eq(&s2));
    }
}
