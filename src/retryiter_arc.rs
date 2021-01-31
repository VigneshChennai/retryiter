use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use crate::retryiter::item::Item;
use crate::retryiter::RetryIterImpl;
use crate::retryiter::tracker::{Tracker, TrackerImpl};

#[derive(Debug)]
pub struct ArcItem<'a, V, Err> {
    inner: Item<'a, V, Err, Arc<Mutex<TrackerImpl<V, Err>>>>
}

impl<'a, V, Err> Deref for ArcItem<'a, V, Err> {
    type Target = Item<'a, V, Err, Arc<Mutex<TrackerImpl<V, Err>>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, V, Err> DerefMut for ArcItem<'a, V, Err> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct ArcRetryIter<V, Itr: Iterator<Item=V>, Err> {
    inner: RetryIterImpl<V, Itr, Err, Arc<Mutex<TrackerImpl<V, Err>>>>
}

impl<V, Itr: Iterator<Item=V>, Err> Deref for ArcRetryIter<V, Itr, Err> {
    type Target = RetryIterImpl<V, Itr, Err, Arc<Mutex<TrackerImpl<V, Err>>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V, Itr: Iterator<Item=V>, Err> DerefMut for ArcRetryIter<V, Itr, Err> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}


impl<V, Itr: Iterator<Item=V>, Err> ArcRetryIter<V, Itr, Err> {
    pub fn new(iter: Itr, max_retries: usize) -> ArcRetryIter<V, Itr, Err> {
        ArcRetryIter {
            inner: RetryIterImpl::new(
                iter,
                Arc::new(Mutex::new(TrackerImpl::new(max_retries))),
            )
        }
    }

    pub fn failed_items(self) -> Vec<(V, Option<Err>)> {
        self.inner.tracker().failed_items()
    }
}

impl<V, Err> Tracker<V, Err> for Arc<Mutex<TrackerImpl<V, Err>>> {
    #[inline]
    fn item_from_failed(&mut self) -> Option<(V, usize, Option<Err>)> {
        self.lock().expect("Lock poisoned").item_from_failed()
    }

    #[inline]
    fn add_item_to_failed(&mut self, item: V, attempt: usize, err: Option<Err>) {
        self.lock().expect("Lock poisoned").add_item_to_failed(item, attempt, err)
    }

    #[inline]
    fn add_item_to_permanent_failure(&mut self, item: V, err: Option<Err>) {
        self.lock().expect("Lock poisoned").add_item_to_permanent_failure(item, err)
    }

    #[inline]
    fn get_max_retries(&self) -> usize {
        self.lock().expect("Lock poisoned").get_max_retries()
    }

    fn failed_items(self) -> Vec<(V, Option<Err>)> {
        let mut guard = self.lock().expect("Lock poisoned");
        let tracker = mem::replace(
            guard.deref_mut(), Default::default());

        return tracker.failed_items();
    }
}
