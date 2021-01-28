use std::marker::PhantomData;

use uuid::Uuid;

use crate::retryiter::tracker::{NotExist, Tracker};

#[derive(Debug)]
pub struct ItemImpl<'a, V: Clone, Err, T: Tracker<V, Err>> {
    pub item_id: Uuid,
    pub value: V,
    pub attempt: usize,
    pub tracker: T,
    _marker: PhantomData<Err>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, V: Clone, Err, T: Tracker<V, Err>> ItemImpl<'a, V, Err, T> {
    pub fn new(item_id: Uuid, value: V, attempt: usize, tracker: T) -> Self {
        ItemImpl {
            item_id,
            value,
            attempt,
            tracker,
            _marker: Default::default(),
            _lifetime: Default::default(),
        }
    }
}

impl<'a, V: Clone + PartialEq, Err, T: Tracker<V, Err>> PartialEq for ItemImpl<'a, V, Err, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<'a, V: Clone, Err, T: Tracker<V, Err>> ItemImpl<'a, V, Err, T> {
    pub fn succeeded(&mut self) -> Result<(), NotExist> {
        self.tracker.success(self.item_id)
    }

    pub fn failed(&mut self, err: Err) -> Result<(), NotExist> {
        self.tracker.failure(self.item_id, err)
    }
}
