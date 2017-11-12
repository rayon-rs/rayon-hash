/// Rayon extensions to `RawTable`

use std::marker;
use std::ptr;

use rayon::prelude::*;
use rayon::iter::plumbing::*;

use std_hash::table::{RawTable, RawBucket, EMPTY_BUCKET};


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

    fn split(&self) -> (Self, Option<Self>) {
        let mut left = SplitBuckets { ..*self };
        let len = left.end - left.bucket.idx;
        if len > 1 {
            let mut right = SplitBuckets { ..left };
            right.bucket.idx += len / 2;
            left.end = right.bucket.idx;
            (left, Some(right))
        } else {
            (left, None)
        }
    }
}

impl<'a, K, V> Iterator for SplitBuckets<'a, K, V> {
    type Item = RawBucket<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket.idx < self.end {
            let item = self.bucket;
            self.bucket.idx += 1;

            unsafe {
                if *item.hash() != EMPTY_BUCKET {
                    return Some(item);
                }
            }
        }
        None
    }
}


/// Parallel iterator over shared references to entries in a table.
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
        where C: UnindexedConsumer<Self::Item>
    {
        let producer = ParIterProducer { iter: SplitBuckets::new(self.table) };
        bridge_unindexed(producer, consumer)
    }
}

struct ParIterProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
}

unsafe impl<'a, K: Sync, V: Sync> Send for ParIterProducer<'a, K, V> {}

impl<'a, K: Sync, V: Sync> UnindexedProducer for ParIterProducer<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn split(mut self) -> (Self, Option<Self>) {
        let (left, right) = self.iter.split();
        self.iter = left;
        let right = right.map(|iter| ParIterProducer { iter: iter });
        (self, right)
    }

    fn fold_with<F>(self, folder: F) -> F
        where F: Folder<Self::Item>
    {
        let iter = self.iter
            .map(|bucket| unsafe {
                     let pair_ptr = bucket.pair();
                     (&(*pair_ptr).0, &(*pair_ptr).1)
                 });
        folder.consume_iter(iter)
    }
}


/// Parallel iterator over mutable references to entries in a table.
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
        where C: UnindexedConsumer<Self::Item>
    {
        let producer = ParIterMutProducer {
            iter: SplitBuckets::new(self.table),
            marker: marker::PhantomData,
        };
        bridge_unindexed(producer, consumer)
    }
}

struct ParIterMutProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
    // To ensure invariance with respect to V
    marker: marker::PhantomData<&'a mut V>,
}

unsafe impl<'a, K: Sync, V: Send> Send for ParIterMutProducer<'a, K, V> {}

impl<'a, K: Sync, V: Send> UnindexedProducer for ParIterMutProducer<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn split(mut self) -> (Self, Option<Self>) {
        let (left, right) = self.iter.split();
        self.iter = left;
        let right = right.map(|iter| ParIterMutProducer { iter: iter, ..self });
        (self, right)
    }

    fn fold_with<F>(self, folder: F) -> F
        where F: Folder<Self::Item>
    {
        let iter = self.iter
            .map(|bucket| unsafe {
                     let pair_ptr = bucket.pair();
                     (&(*pair_ptr).0, &mut (*pair_ptr).1)
                 });
        folder.consume_iter(iter)
    }
}

/// Parallel iterator over the entries in a table, consuming it.
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
        where C: UnindexedConsumer<Self::Item>
    {
        // Pre-set the map size to zero, indicating all items drained.
        let mut table = self.table;
        table.size = 0;

        let producer = ParIntoIterProducer {
            iter: SplitBuckets::new(&table),
        };
        bridge_unindexed(producer, consumer)
    }
}

struct ParIntoIterProducer<'a, K: 'a, V: 'a> {
    iter: SplitBuckets<'a, K, V>,
}

unsafe impl<'a, K: Send, V: Send> Send for ParIntoIterProducer<'a, K, V> {}

impl<'a, K: Send, V: Send> UnindexedProducer for ParIntoIterProducer<'a, K, V> {
    type Item = (K, V);

    fn split(mut self) -> (Self, Option<Self>) {
        let (left, right) = self.iter.split();
        self.iter = left;
        let right = right.map(|iter| ParIntoIterProducer { iter: iter });
        (self, right)
    }

    fn fold_with<F>(mut self, folder: F) -> F
        where F: Folder<Self::Item>
    {
        let iter = self.iter
            .by_ref()
            .map(|bucket| unsafe {
                     *bucket.hash() = EMPTY_BUCKET;
                     ptr::read(bucket.pair())
                 });
        folder.consume_iter(iter)
    }
}

impl<'a, K: 'a, V: 'a> Drop for ParIntoIterProducer<'a, K, V> {
    fn drop(&mut self) {
        for bucket in self.iter.by_ref() {
            unsafe {
                *bucket.hash() = EMPTY_BUCKET;
                ptr::drop_in_place(bucket.pair());
            }
        }
    }
}
