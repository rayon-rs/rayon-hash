/// Rayon extensions to `HashMap`

use rayon::iter::{ParallelIterator, IntoParallelIterator, FromParallelIterator, ParallelExtend};
use rayon::iter::plumbing::UnindexedConsumer;

use super::{Hash, HashMap, BuildHasher};
use std_hash::table;


pub struct ParIntoIter<K: Send, V: Send> {
    inner: table::ParIntoIter<K, V>,
}

pub struct ParIter<'a, K: Sync + 'a, V: Sync + 'a> {
    inner: table::ParIter<'a, K, V>,
}

pub struct ParIterMut<'a, K: Sync + 'a, V: Send + 'a> {
    inner: table::ParIterMut<'a, K, V>,
}

pub struct ParKeys<'a, K: Sync + 'a, V: Sync + 'a> {
    inner: ParIter<'a, K, V>,
}

pub struct ParValues<'a, K: Sync + 'a, V: Sync + 'a> {
    inner: ParIter<'a, K, V>,
}

pub struct ParValuesMut<'a, K: Sync + 'a, V: Send + 'a> {
    inner: ParIterMut<'a, K, V>,
}


impl<K: Sync, V: Sync, S> HashMap<K, V, S> {
    pub fn par_keys(&self) -> ParKeys<K, V> {
        ParKeys { inner: self.into_par_iter() }
    }

    pub fn par_values(&self) -> ParValues<K, V> {
        ParValues { inner: self.into_par_iter() }
    }
}

impl<K, V, S> HashMap<K, V, S>
    where K: Eq + Hash + Sync,
          V: PartialEq + Sync,
          S: BuildHasher + Sync
{
    pub fn par_eq(&self, other: &Self) -> bool {
        self.len() == other.len() &&
        self.into_par_iter().all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K: Sync, V: Send, S> HashMap<K, V, S> {
    pub fn par_values_mut(&mut self) -> ParValuesMut<K, V> {
        ParValuesMut { inner: self.into_par_iter() }
    }
}


impl<K: Send, V: Send, S> IntoParallelIterator for HashMap<K, V, S> {
    type Item = (K, V);
    type Iter = ParIntoIter<K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIntoIter { inner: self.table.into_par_iter() }
    }
}

impl<'a, K: Sync, V: Sync, S> IntoParallelIterator for &'a HashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type Iter = ParIter<'a, K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIter { inner: self.table.into_par_iter() }
    }
}

impl<'a, K: Sync, V: Send, S> IntoParallelIterator for &'a mut HashMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type Iter = ParIterMut<'a, K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIterMut { inner: self.table.into_par_iter() }
    }
}


/// Collect (key, value) pairs from a parallel iterator into a
/// hashmap. If multiple pairs correspond to the same key, then the
/// ones produced earlier in the parallel iterator will be
/// overwritten, just as with a sequential iterator.
impl<K, V, S> FromParallelIterator<(K, V)> for HashMap<K, V, S>
    where K: Eq + Hash + Send,
          V: Send,
          S: BuildHasher + Default + Send
{
    fn from_par_iter<P>(par_iter: P) -> Self
        where P: IntoParallelIterator<Item = (K, V)>
    {
        let mut map = HashMap::default();
        map.par_extend(par_iter);
        map
    }
}


/// Extend a hash map with items from a parallel iterator.
impl<K, V, S> ParallelExtend<(K, V)> for HashMap<K, V, S>
    where K: Eq + Hash + Send,
          V: Send,
          S: BuildHasher + Send
{
    fn par_extend<I>(&mut self, par_iter: I)
        where I: IntoParallelIterator<Item = (K, V)>
    {
        extend(self, par_iter);
    }
}

/// Extend a hash map with copied items from a parallel iterator.
impl<'a, K, V, S> ParallelExtend<(&'a K, &'a V)> for HashMap<K, V, S>
    where K: Copy + Eq + Hash + Send + Sync,
          V: Copy + Send + Sync,
          S: BuildHasher + Send
{
    fn par_extend<I>(&mut self, par_iter: I)
        where I: IntoParallelIterator<Item = (&'a K, &'a V)>
    {
        extend(self, par_iter);
    }
}

// This is equal to the normal `HashMap` -- no custom advantage.
fn extend<K, V, S, I>(map: &mut HashMap<K, V, S>, par_iter: I)
    where K: Eq + Hash,
          S: BuildHasher,
          I: IntoParallelIterator,
          HashMap<K, V, S>: Extend<I::Item>
{
    use std::collections::LinkedList;

    let list: LinkedList<_> = par_iter.into_par_iter()
        .fold(Vec::new, |mut vec, elem| {
            vec.push(elem);
            vec
        })
        .collect();

    map.reserve(list.iter().map(Vec::len).sum());
    for vec in list {
        map.extend(vec);
    }
}


impl<K: Send, V: Send> ParallelIterator for ParIntoIter<K, V> {
    type Item = (K, V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.drive_unindexed(consumer)
    }
}


impl<'a, K: Sync, V: Sync> ParallelIterator for ParIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.drive_unindexed(consumer)
    }
}


impl<'a, K: Sync, V: Send> ParallelIterator for ParIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.drive_unindexed(consumer)
    }
}


impl<'a, K: Sync, V: Sync> ParallelIterator for ParKeys<'a, K, V> {
    type Item = &'a K;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.map(|(k, _)| k).drive_unindexed(consumer)
    }
}


impl<'a, K: Sync, V: Sync> ParallelIterator for ParValues<'a, K, V> {
    type Item = &'a V;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.map(|(_, v)| v).drive_unindexed(consumer)
    }
}


impl<'a, K: Sync, V: Send> ParallelIterator for ParValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
        where C: UnindexedConsumer<Self::Item>
    {
        self.inner.map(|(_, v)| v).drive_unindexed(consumer)
    }
}


#[cfg(test)]
mod test_par_map {
    use super::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::hash::{Hash, Hasher};
    use rayon::prelude::*;

    struct Dropable<'a> {
        k: usize,
        counter: &'a AtomicUsize,
    }

    impl<'a> Dropable<'a> {
        fn new(k: usize, counter: &AtomicUsize) -> Dropable {
            counter.fetch_add(1, Ordering::Relaxed);

            Dropable { k: k, counter: counter }
        }
    }

    impl<'a> Drop for Dropable<'a> {
        fn drop(&mut self) {
            self.counter.fetch_sub(1, Ordering::Relaxed);
        }
    }

    impl<'a> Clone for Dropable<'a> {
        fn clone(&self) -> Dropable<'a> {
            Dropable::new(self.k, self.counter)
        }
    }

    impl<'a> Hash for Dropable<'a> {
        fn hash<H>(&self, state: &mut H)
            where H: Hasher
        {
            self.k.hash(state)
        }
    }

    impl<'a> PartialEq for Dropable<'a> {
        fn eq(&self, other: &Self) -> bool {
            self.k == other.k
        }
    }

    impl<'a> Eq for Dropable<'a> {}

    #[test]
    fn test_into_iter_drops() {
        let key = AtomicUsize::new(0);
        let value = AtomicUsize::new(0);

        let hm = {
            let mut hm = HashMap::new();

            assert_eq!(key.load(Ordering::Relaxed), 0);
            assert_eq!(value.load(Ordering::Relaxed), 0);

            for i in 0..100 {
                let d1 = Dropable::new(i, &key);
                let d2 = Dropable::new(i + 100, &value);
                hm.insert(d1, d2);
            }

            assert_eq!(key.load(Ordering::Relaxed), 100);
            assert_eq!(value.load(Ordering::Relaxed), 100);

            hm
        };

        // By the way, ensure that cloning doesn't screw up the dropping.
        drop(hm.clone());

        {
            assert_eq!(key.load(Ordering::Relaxed), 100);
            assert_eq!(value.load(Ordering::Relaxed), 100);

            // retain only half
            let _v: Vec<_> = hm.into_par_iter()
                .filter(|&(ref key, _)| key.k < 50)
                .collect();

            assert_eq!(key.load(Ordering::Relaxed), 50);
            assert_eq!(value.load(Ordering::Relaxed), 50);
        };

        assert_eq!(key.load(Ordering::Relaxed), 0);
        assert_eq!(value.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_empty_iter() {
        let mut m: HashMap<isize, bool> = HashMap::new();
        //assert_eq!(m.par_drain().count(), 0);
        assert_eq!(m.par_keys().count(), 0);
        assert_eq!(m.par_values().count(), 0);
        assert_eq!(m.par_values_mut().count(), 0);
        assert_eq!(m.par_iter().count(), 0);
        assert_eq!(m.par_iter_mut().count(), 0);
        assert_eq!(m.len(), 0);
        assert!(m.is_empty());
        assert_eq!(m.into_par_iter().count(), 0);
    }

    #[test]
    fn test_iterate() {
        let mut m = HashMap::with_capacity(4);
        for i in 0..32 {
            assert!(m.insert(i, i*2).is_none());
        }
        assert_eq!(m.len(), 32);

        let observed = AtomicUsize::new(0);

        m.par_iter().for_each(|(k, v)| {
            assert_eq!(*v, *k * 2);
            observed.fetch_or(1 << *k, Ordering::Relaxed);
        });
        assert_eq!(observed.into_inner(), 0xFFFF_FFFF);
    }

    #[test]
    fn test_keys() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: HashMap<_, _> = vec.into_par_iter().collect();
        let keys: Vec<_> = map.par_keys().cloned().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }

    #[test]
    fn test_values() {
        let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
        let map: HashMap<_, _> = vec.into_par_iter().collect();
        let values: Vec<_> = map.par_values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }

    #[test]
    fn test_values_mut() {
        let vec = vec![(1, 1), (2, 2), (3, 3)];
        let mut map: HashMap<_, _> = vec.into_par_iter().collect();
        map.par_values_mut().for_each(|value| {
            *value = (*value) * 2
        });
        let values: Vec<_> = map.par_values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&2));
        assert!(values.contains(&4));
        assert!(values.contains(&6));
    }

    #[test]
    fn test_eq() {
        let mut m1 = HashMap::new();
        m1.insert(1, 2);
        m1.insert(2, 3);
        m1.insert(3, 4);

        let mut m2 = HashMap::new();
        m2.insert(1, 2);
        m2.insert(2, 3);

        assert!(!m1.par_eq(&m2));

        m2.insert(3, 4);

        assert!(m1.par_eq(&m2));
    }

    #[test]
    fn test_from_iter() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];

        let map: HashMap<_, _> = xs.par_iter().cloned().collect();

        for &(k, v) in &xs {
            assert_eq!(map.get(&k), Some(&v));
        }
    }

    #[test]
    fn test_extend_ref() {
        let mut a = HashMap::new();
        a.insert(1, "one");
        let mut b = HashMap::new();
        b.insert(2, "two");
        b.insert(3, "three");

        a.par_extend(&b);

        assert_eq!(a.len(), 3);
        assert_eq!(a[&1], "one");
        assert_eq!(a[&2], "two");
        assert_eq!(a[&3], "three");
    }
}
