use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::retryiter::item::Item;
use crate::retryiter::RetryIterImpl;
use crate::retryiter::tracker::{Tracker, TrackerImpl};

#[derive(Debug)]
pub struct RcItem<'a, V, Err> {
    inner: Item<'a, V, Err, Rc<RefCell<TrackerImpl<V, Err>>>>
}

impl<'a, V, Err> Deref for RcItem<'a, V, Err> {
    type Target = Item<'a, V, Err, Rc<RefCell<TrackerImpl<V, Err>>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, V, Err> DerefMut for RcItem<'a, V, Err> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct RcRetryIter<V, Itr: Iterator<Item=V>, Err> {
    inner: RetryIterImpl<V, Itr, Err, Rc<RefCell<TrackerImpl<V, Err>>>>
}

impl<V, Itr: Iterator<Item=V>, Err> Deref for RcRetryIter<V, Itr, Err> {
    type Target = RetryIterImpl<V, Itr, Err, Rc<RefCell<TrackerImpl<V, Err>>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<V, Itr: Iterator<Item=V>, Err> DerefMut for RcRetryIter<V, Itr, Err> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<V, Itr: Iterator<Item=V>, Err> RcRetryIter<V, Itr, Err> {
    pub fn new(iter: Itr, max_retries: usize) -> RcRetryIter<V, Itr, Err> {
        RcRetryIter {
            inner: RetryIterImpl::new(
                iter,
                Rc::new(RefCell::new(TrackerImpl::new(max_retries))),
            )
        }
    }

    pub fn failed_items(self) -> Vec<(V, Option<Err>)> {
        self.inner.tracker().failed_items()
    }
}


impl<V, Err> Tracker<V, Err> for Rc<RefCell<TrackerImpl<V, Err>>> {
    #[inline]
    fn item_from_failed(&mut self) -> Option<(V, usize, Option<Err>)> {
        self.borrow_mut().item_from_failed()
    }

    #[inline]
    fn add_item_to_failed(&mut self, item: V, attempt: usize, err: Option<Err>) {
        self.borrow_mut().add_item_to_failed(item, attempt, err)
    }

    #[inline]
    fn add_item_to_permanent_failure(&mut self, item: V, err: Option<Err>) {
        self.borrow_mut().add_item_to_permanent_failure(item, err)
    }

    #[inline]
    fn get_max_retries(&self) -> usize {
        self.borrow().get_max_retries()
    }

    #[inline]
    fn failed_items(self) -> Vec<(V, Option<Err>)> {
        self.replace(Default::default()).failed_items()
    }
}
