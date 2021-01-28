use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use uuid::Uuid;

use crate::retryiter::item::ItemImpl;
use crate::retryiter::RetryIterImpl;
use crate::retryiter::tracker::{NotExist, Tracker, TrackerImpl};

#[derive(Debug)]
pub struct RcItem<'a, V: Clone, Err> {
    inner: ItemImpl<'a, V, Err, Rc<RefCell<TrackerImpl<V, Err>>>>
}

impl<'a, V: Clone, Err> RcItem<'a, V, Err> {
    pub fn new(item_id: Uuid, value: V, attempt: usize,
               tracker: Rc<RefCell<TrackerImpl<V, Err>>>) -> Self {
        RcItem { inner: ItemImpl::new(item_id, value, attempt, tracker) }
    }
}

pub struct RcRetryIter<V: Clone, Itr: Iterator<Item=V>, Err> {
    inner: RetryIterImpl<V, Itr, Err, Rc<RefCell<TrackerImpl<V, Err>>>>
}

impl<V: Clone, Itr: Iterator<Item=V>, Err> RcRetryIter<V, Itr, Err> {
    pub fn new(iter: Itr, max_retries: usize) -> RcRetryIter<V, Itr, Err> {
        RcRetryIter {
            inner: RetryIterImpl::new(
                iter,
                Rc::new(RefCell::new(TrackerImpl::new(max_retries))),
            )
        }
    }

    pub fn failed_items(self) -> Vec<(V, Err)> {
        self.inner.tracker().failed_items()
    }
}

impl<'a, V: Clone, Err> Deref for RcItem<'a, V, Err> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.inner.value
    }
}

impl<'a, V: Clone, Err> DerefMut for RcItem<'a, V, Err> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.value
    }
}

impl<'a, V: Clone, Err> RcItem<'a, V, Err> {
    pub fn succeeded(&mut self) -> Result<(), NotExist> {
        self.inner.succeeded()
    }

    pub fn failed(&mut self, err: Err) -> Result<(), NotExist> {
        self.inner.failed(err)
    }
}

impl<'a, V: Clone + PartialEq, Err> PartialEq for RcItem<'a, V, Err> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<'a, V: Clone + PartialEq, Err> PartialEq<V> for RcItem<'a, V, Err> {
    fn eq(&self, other: &V) -> bool {
        &self.inner.value == other
    }
}

impl<'a, V: Clone + PartialOrd, Err> PartialOrd for RcItem<'a, V, Err> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.value.partial_cmp(&other.inner.value)
    }
}

impl<'a, V: Clone + PartialOrd, Err> PartialOrd<V> for RcItem<'a, V, Err> {
    fn partial_cmp(&self, other: &V) -> Option<Ordering> {
        self.inner.value.partial_cmp(other)
    }
}


impl<'a, Itr: Iterator<Item=V>, V: Clone, Err> Iterator for &'a mut RcRetryIter<V, Itr, Err> {
    type Item = RcItem<'a, V, Err>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut iter = &mut self.inner;
        if let Some(v) = iter.next() {
            Some(RcItem::<'a, V, Err>::new(
                v.item_id, v.value, v.attempt, v.tracker,
            ))
        } else {
            None
        }
    }
}

impl<V: Clone, Err> Tracker<V, Err> for Rc<RefCell<TrackerImpl<V, Err>>> {
    fn item_from_pending(&mut self) -> Option<(V, usize)> {
        self.borrow_mut().item_from_pending()
    }

    fn item_from_failed(&mut self) -> Option<(V, usize, Err)> {
        self.borrow_mut().item_from_failed()
    }

    fn move_item_to_pending(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        self.borrow_mut().move_item_to_pending(item_id)
    }

    fn move_item_to_failed(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist> {
        self.borrow_mut().move_item_to_failed(item_id, err)
    }

    fn move_incomplete_to_pending(&mut self) {
        self.borrow_mut().move_incomplete_to_pending()
    }

    fn add_item_to_permanent_failure(&mut self, item: V, err: Err) {
        self.borrow_mut().add_item_to_permanent_failure(item, err)
    }

    fn add_item_to_in_progress(&mut self, item_id: Uuid, item: V, attempt: usize) {
        self.borrow_mut().add_item_to_in_progress(item_id, item, attempt)
    }

    fn delete_item_from_in_progress(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        self.borrow_mut().delete_item_from_in_progress(item_id)
    }

    fn get_max_retries(&self) -> usize {
        self.borrow().get_max_retries()
    }

    fn failed_items(self) -> Vec<(V, Err)> {
        self.replace(Default::default()).failed_items()
    }
}
