use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;

use crate::retryiter::tracker::Tracker;

#[derive(Debug)]
pub enum ItemStatus<Err> {
    Success,
    Failed(Option<Err>),
    NotDone,
    None,
}

impl<Err: Clone> Clone for ItemStatus<Err> {
    fn clone(&self) -> Self {
        match self {
            ItemStatus::Success => ItemStatus::Success,
            ItemStatus::Failed(err) => ItemStatus::Failed(err.clone()),
            ItemStatus::NotDone => ItemStatus::NotDone,
            ItemStatus::None => ItemStatus::None
        }
    }
}

#[derive(Debug)]
pub struct Item<'a, V, Err, T: Tracker<V, Err>> {
    value: ManuallyDrop<V>,
    attempt: usize,
    status: ItemStatus<Err>,
    tracker: T,
    _marker: PhantomData<Err>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, V, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    pub fn new(value: V, attempt: usize, tracker: T) -> Self {
        Item {
            value: ManuallyDrop::new(value),
            attempt,
            status: ItemStatus::None,
            tracker,
            _marker: Default::default(),
            _lifetime: Default::default(),
        }
    }
}

impl<'a, V, Err, T: Tracker<V, Err>> Drop for Item<'a, V, Err, T> {
    fn drop(&mut self) {
        let value = unsafe {
            // This is safe as we are doing it in drop implementation and
            // not using the self.value after this statement in this function.
            ManuallyDrop::take(&mut self.value)
        };
        let status = std::mem::replace(&mut self.status, ItemStatus::None);

        match status {
            ItemStatus::Success | ItemStatus::None => { /* No operation on success */ }
            ItemStatus::Failed(err) => {
                if self.tracker.get_max_retries() + 1 > self.attempt {
                    self.tracker.failed(value, self.attempt, err)
                } else {
                    self.tracker.add_item_to_permanent_failure(value, err)
                }
            }
            ItemStatus::NotDone => self.tracker.not_done(value, self.attempt),
        };
    }
}

impl<'a, V: PartialEq, Err, T: Tracker<V, Err>> PartialEq for Item<'a, V, Err, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.deref() == other.value.deref()
    }
}

impl<'a, V: PartialEq, Err, T: Tracker<V, Err>> PartialEq<V> for Item<'a, V, Err, T> {
    fn eq(&self, other: &V) -> bool {
        self.value.deref() == other
    }
}

impl<'a, V: PartialOrd, Err, T: Tracker<V, Err>> PartialOrd for Item<'a, V, Err, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.deref().partial_cmp(other.value.deref())
    }
}

impl<'a, V: PartialOrd, Err, T: Tracker<V, Err>> PartialOrd<V> for Item<'a, V, Err, T> {
    fn partial_cmp(&self, other: &V) -> Option<Ordering> {
        self.value.deref().partial_cmp(other)
    }
}

impl<'a, V, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    pub fn attempt(&self) -> usize {
        self.attempt
    }

    pub fn succeeded(mut self) {
        self.status = ItemStatus::Success;
    }

    pub fn failed(mut self, err: Option<Err>) {
        self.status = ItemStatus::Failed(err);
    }

    pub fn set_default(&mut self, status: ItemStatus<Err>) {
        self.status = status
    }
}

impl<'a, V: Clone, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    pub fn value(&self) -> V {
        self.value.deref().clone()
    }
}
