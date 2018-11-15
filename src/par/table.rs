/// Rayon extensions to `RawTable`
use std::marker;
use std::ptr;

use rayon::iter::plumbing::*;
use rayon::prelude::*;

use std_hash::table::{RawBucket, RawTable};

struct SplitBuckets<'a, K, V> {
    bucket: RawBucket<K, V>,
    end: usize,
    marker: marker::PhantomData<&'a ()>,
}

impl<'a, K, V> SplitBuckets<'a, K, V> {
    fn new(table: &'a RawTable<K, V>) -> Self {
        SplitBuckets {
            bucket: table.raw_bucket_at(0),
            end: table.capacity(),
            marker: marker::PhantomData,
        }
    }

    fn split<P: From<Self>>(&self) -> (P, Option<P>) {
        let mut left = SplitBuckets { ..*self };
        let len = left.end - left.bucket.index();
        if len > 1 {
            let mut right = SplitBuckets { ..left };
            right.bucket.index_add(len / 2);
            left.end = right.bucket.index();
            (P::from(left), Some(P::from(right)))
        } else {
            (P::from(left), None)
        }
    }
}

impl<'a, K, V> Iterator for SplitBuckets<'a, K, V> {
    type Item = RawBucket<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket.index() < self.end {
            let item = self.bucket;
            self.bucket.index_add(1);

            unsafe {
                if !item.is_empty() {
                    return Some(item);
                }
            }
        }
        None
    }
}

/// Parallel iterator over shared references to entries in a map.
pub struct ParIter<'a, K: 'a, V: 'a> {
    table: &'a RawTable<K, V>,
}

impl<'a, K: Sync, V: Sync> IntoParallelIterator for &'a RawTable<K, V> {
    type Item = (&'a K, &'a V);
    type Iter = ParIter<'a, K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIter { table: self }
    }
}

impl<'a, K: Sync, V: Sync> ParallelIterator for ParIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let buckets = SplitBuckets::new(self.table);
        let producer = ParIterProducer::from(buckets);
        bridge_unindexed(producer, consumer)
    }
}

struct ParIterProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
}

impl<'a, K, V> From<SplitBuckets<'a, K, V>> for ParIterProducer<'a, K, V> {
    fn from(iter: SplitBuckets<'a, K, V>) -> Self {
        Self { iter }
    }
}

unsafe impl<'a, K: Sync, V: Sync> Send for ParIterProducer<'a, K, V> {}

impl<'a, K: Sync, V: Sync> UnindexedProducer for ParIterProducer<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn split(self) -> (Self, Option<Self>) {
        self.iter.split()
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        let iter = self.iter.map(|bucket| unsafe {
            let pair_ptr = bucket.pair();
            (&(*pair_ptr).0, &(*pair_ptr).1)
        });
        folder.consume_iter(iter)
    }
}

/// Parallel iterator over shared references to keys in a map.
pub struct ParKeys<'a, K: 'a, V: 'a> {
    table: &'a RawTable<K, V>,
}

unsafe impl<'a, K: Sync, V> Send for ParKeys<'a, K, V> {}

impl<K: Sync, V> RawTable<K, V> {
    pub fn par_keys(&self) -> ParKeys<K, V> {
        ParKeys { table: self }
    }
}

impl<'a, K: Sync, V> ParallelIterator for ParKeys<'a, K, V> {
    type Item = &'a K;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let buckets = SplitBuckets::new(self.table);
        let producer = ParKeysProducer::from(buckets);
        bridge_unindexed(producer, consumer)
    }
}

struct ParKeysProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
}

impl<'a, K, V> From<SplitBuckets<'a, K, V>> for ParKeysProducer<'a, K, V> {
    fn from(iter: SplitBuckets<'a, K, V>) -> Self {
        Self { iter }
    }
}

unsafe impl<'a, K: Sync, V> Send for ParKeysProducer<'a, K, V> {}

impl<'a, K: Sync, V> UnindexedProducer for ParKeysProducer<'a, K, V> {
    type Item = &'a K;

    fn split(self) -> (Self, Option<Self>) {
        self.iter.split()
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        let iter = self.iter.map(|bucket| unsafe {
            let pair_ptr = bucket.pair();
            &(*pair_ptr).0
        });
        folder.consume_iter(iter)
    }
}

/// Parallel iterator over shared references to values in a map.
pub struct ParValues<'a, K: 'a, V: 'a> {
    table: &'a RawTable<K, V>,
}

unsafe impl<'a, K, V: Sync> Send for ParValues<'a, K, V> {}

impl<K, V: Sync> RawTable<K, V> {
    pub fn par_values(&self) -> ParValues<K, V> {
        ParValues { table: self }
    }
}

impl<'a, K, V: Sync> ParallelIterator for ParValues<'a, K, V> {
    type Item = &'a V;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let buckets = SplitBuckets::new(self.table);
        let producer = ParValuesProducer::from(buckets);
        bridge_unindexed(producer, consumer)
    }
}

struct ParValuesProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
}

impl<'a, K, V> From<SplitBuckets<'a, K, V>> for ParValuesProducer<'a, K, V> {
    fn from(iter: SplitBuckets<'a, K, V>) -> Self {
        Self { iter }
    }
}

unsafe impl<'a, K, V: Sync> Send for ParValuesProducer<'a, K, V> {}

impl<'a, K, V: Sync> UnindexedProducer for ParValuesProducer<'a, K, V> {
    type Item = &'a V;

    fn split(self) -> (Self, Option<Self>) {
        self.iter.split()
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        let iter = self.iter.map(|bucket| unsafe {
            let pair_ptr = bucket.pair();
            &(*pair_ptr).1
        });
        folder.consume_iter(iter)
    }
}

/// Parallel iterator over mutable references to entries in a map.
pub struct ParIterMut<'a, K: 'a, V: 'a> {
    table: &'a mut RawTable<K, V>,
}

unsafe impl<'a, K: Sync, V: Send> Send for ParIterMut<'a, K, V> {}

impl<'a, K: Sync, V: Send> IntoParallelIterator for &'a mut RawTable<K, V> {
    type Item = (&'a K, &'a mut V);
    type Iter = ParIterMut<'a, K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIterMut { table: self }
    }
}

impl<'a, K: Sync, V: Send> ParallelIterator for ParIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let buckets = SplitBuckets::new(self.table);
        let producer = ParIterMutProducer::from(buckets);
        bridge_unindexed(producer, consumer)
    }
}

struct ParIterMutProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
    // To ensure invariance with respect to V
    marker: marker::PhantomData<&'a mut V>,
}

impl<'a, K, V> From<SplitBuckets<'a, K, V>> for ParIterMutProducer<'a, K, V> {
    fn from(iter: SplitBuckets<'a, K, V>) -> Self {
        Self {
            iter,
            marker: marker::PhantomData,
        }
    }
}

unsafe impl<'a, K: Sync, V: Send> Send for ParIterMutProducer<'a, K, V> {}

impl<'a, K: Sync, V: Send> UnindexedProducer for ParIterMutProducer<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn split(self) -> (Self, Option<Self>) {
        self.iter.split()
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        let iter = self.iter.map(|bucket| unsafe {
            let pair_ptr = bucket.pair();
            (&(*pair_ptr).0, &mut (*pair_ptr).1)
        });
        folder.consume_iter(iter)
    }
}

/// Parallel iterator over mutable references to values in a map.
pub struct ParValuesMut<'a, K: 'a, V: 'a> {
    table: &'a mut RawTable<K, V>,
}

unsafe impl<'a, K, V: Send> Send for ParValuesMut<'a, K, V> {}

impl<K, V: Send> RawTable<K, V> {
    pub fn par_values_mut(&mut self) -> ParValuesMut<K, V> {
        ParValuesMut { table: self }
    }
}

impl<'a, K, V: Send> ParallelIterator for ParValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let buckets = SplitBuckets::new(self.table);
        let producer = ParValuesMutProducer::from(buckets);
        bridge_unindexed(producer, consumer)
    }
}

struct ParValuesMutProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
    // To ensure invariance with respect to V
    marker: marker::PhantomData<&'a mut V>,
}

impl<'a, K, V> From<SplitBuckets<'a, K, V>> for ParValuesMutProducer<'a, K, V> {
    fn from(iter: SplitBuckets<'a, K, V>) -> Self {
        Self {
            iter,
            marker: marker::PhantomData,
        }
    }
}

unsafe impl<'a, K, V: Send> Send for ParValuesMutProducer<'a, K, V> {}

impl<'a, K, V: Send> UnindexedProducer for ParValuesMutProducer<'a, K, V> {
    type Item = &'a mut V;

    fn split(self) -> (Self, Option<Self>) {
        self.iter.split()
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        let iter = self.iter.map(|bucket| unsafe {
            let pair_ptr = bucket.pair();
            &mut (*pair_ptr).1
        });
        folder.consume_iter(iter)
    }
}

/// Parallel iterator over the entries in a map, consuming it.
pub struct ParIntoIter<K, V> {
    table: RawTable<K, V>,
}

impl<K: Send, V: Send> IntoParallelIterator for RawTable<K, V> {
    type Item = (K, V);
    type Iter = ParIntoIter<K, V>;

    fn into_par_iter(self) -> Self::Iter {
        ParIntoIter { table: self }
    }
}

impl<K: Send, V: Send> ParallelIterator for ParIntoIter<K, V> {
    type Item = (K, V);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        // Pre-set the map size to zero, indicating all items drained.
        let mut table = self.table;
        unsafe {
            table.set_size(0);
        }

        let buckets = SplitBuckets::new(&table);
        let producer = ParIntoIterProducer::from(buckets);
        bridge_unindexed(producer, consumer)
    }
}

struct ParIntoIterProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
}

impl<'a, K, V> From<SplitBuckets<'a, K, V>> for ParIntoIterProducer<'a, K, V> {
    fn from(iter: SplitBuckets<'a, K, V>) -> Self {
        Self { iter }
    }
}

unsafe impl<'a, K: Send, V: Send> Send for ParIntoIterProducer<'a, K, V> {}

impl<'a, K: Send, V: Send> UnindexedProducer for ParIntoIterProducer<'a, K, V> {
    type Item = (K, V);

    fn split(mut self) -> (Self, Option<Self>) {
        // We must not drop self yet!
        let (left, right) = self.iter.split();
        self.iter = left;
        (self, right.map(Self::from))
    }

    fn fold_with<F>(mut self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        let iter = self.iter.by_ref().map(|bucket| unsafe {
            bucket.set_empty();
            ptr::read(bucket.pair())
        });
        folder.consume_iter(iter)
    }
}

impl<'a, K: 'a, V: 'a> Drop for ParIntoIterProducer<'a, K, V> {
    fn drop(&mut self) {
        while let Some(bucket) = self.iter.next() {
            unsafe {
                bucket.set_empty();
                ptr::drop_in_place(bucket.pair());
            }
        }
    }
}
