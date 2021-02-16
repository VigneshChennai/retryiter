use std::marker::PhantomData;

use item::Item;
use tracker::Tracker;

pub mod item;
pub mod tracker;

/// A generic RetryIter implementation.
pub struct RetryIter<V, Itr, Err, T>
where
    Itr: Iterator<Item = V>,
    T: Tracker<V, Err> + Clone,
{
    inner_iter: Itr,
    tracker: T,
    _marker: PhantomData<Err>,
}

impl<V, Itr, Err, T> RetryIter<V, Itr, Err, T>
where
    Itr: Iterator<Item = V>,
    T: Tracker<V, Err> + Clone,
{
    pub(crate) fn new(iter: Itr, tracker: T) -> RetryIter<V, Itr, Err, T> {
        RetryIter {
            inner_iter: iter,
            tracker,
            _marker: Default::default(),
        }
    }

    /// Returns all the failed items during processing.
    pub fn failed_items(self) -> Vec<(V, Err)> {
        self.tracker.failed_items()
    }
}

impl<'a, Itr, V, Err, T> Iterator for &'a mut RetryIter<V, Itr, Err, T>
where
    Itr: Iterator<Item = V>,
    T: Tracker<V, Err> + Clone,
{
    type Item = Item<'a, V, Err, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner_iter.next() {
            None => {
                if let Some((item, attempt, _err)) = self.tracker.item_from_failed() {
                    Some(Item::<'a>::new(item, attempt + 1, self.tracker.clone()))
                } else {
                    None
                }
            }
            Some(value) => Some(Item::<'a>::new(value, 1, self.tracker.clone())),
        }
    }
}
