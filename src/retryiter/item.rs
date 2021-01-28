use std::marker::PhantomData;

use uuid::Uuid;

use crate::retryiter::tracker::{NotExist, Tracker};

#[derive(Debug)]
pub struct ItemMeta<'a, V: Clone, Err, T: Tracker<V, Err>> {
    pub item_id: Uuid,
    pub attempt: usize,
    pub tracker: T,
    _marker: PhantomData<Err>,
    _lifetime: PhantomData<&'a ()>,
    _marker2: PhantomData<V>
}

impl<'a, V: Clone, Err, T: Tracker<V, Err>> ItemMeta<'a, V, Err, T> {
    pub fn new(item_id: Uuid, attempt: usize, tracker: T) -> Self {
        ItemMeta {
            item_id,
            attempt,
            tracker,
            _marker: Default::default(),
            _lifetime: Default::default(),
            _marker2: Default::default(),
        }
    }
    pub fn succeeded(&mut self) -> Result<(), NotExist> {
        self.tracker.success(self.item_id)
    }

    pub fn failed(&mut self, err: Err) -> Result<(), NotExist> {
        self.tracker.failure(self.item_id, err)
    }
}


#[derive(Debug)]
pub struct Item<'a, V: Clone, Err, T: Tracker<V, Err>> {
    pub value: V,
    pub meta: ItemMeta<'a, V, Err, T>,
}

impl<'a, V: Clone, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    pub fn new(item_id: Uuid, value: V, attempt: usize, tracker: T) -> Self {
        Item {
            value,
            meta: ItemMeta::new(item_id, attempt, tracker),
        }
    }
}

impl<'a, V: Clone + PartialEq, Err, T: Tracker<V, Err>> PartialEq for Item<'a, V, Err, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<'a, V: Clone, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    pub fn attempt(&self) -> usize {
        self.meta.attempt
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn unpack(self) -> (V, ItemMeta<'a, V, Err, T>) {
        (self.value, self.meta)
    }

    pub fn succeeded(&mut self) -> Result<(), NotExist> {
        self.meta.tracker.success(self.meta.item_id)
    }

    pub fn failed(&mut self, err: Err) -> Result<(), NotExist> {
        self.meta.tracker.failure(self.meta.item_id, err)
    }
}
