use std::collections::HashMap;
use thiserror::Error;

use uuid::Uuid;

#[derive(Debug, Error)]
#[error("Item doesn't exists")]
pub struct NotExist;

pub trait Tracker<V: Clone, Err> {
    fn item_from_pending(&mut self) -> Option<(V, usize)>;
    fn item_from_failed(&mut self) -> Option<(V, usize, Err)>;

    fn move_item_to_pending(&mut self, item_id: Uuid) -> Result<(), NotExist>;
    fn move_item_to_failed(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist>;
    fn move_incomplete_to_pending(&mut self);

    fn add_item_to_permanent_failure(&mut self, item: V, err: Err);
    fn add_item_to_in_progress(&mut self, item_id: Uuid, item: V, attempt: usize);
    fn delete_item_from_in_progress(&mut self, item_id: Uuid) -> Result<(), NotExist>;

    fn get_max_retries(&self) -> usize;

    fn success(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        self.delete_item_from_in_progress(item_id)
    }

    fn failure(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist> {
        self.move_item_to_failed(item_id, err)
    }

    fn failed_items(self) -> Vec<(V, Err)>;
}

#[derive(Debug)]
pub struct TrackerImpl<V: Clone, Err> {
    pending: Vec<(V, usize)>,
    in_progress: HashMap<Uuid, (V, usize)>,
    failed: Vec<(V, usize, Err)>,
    max_retries: usize,
    permanent_failure: Vec<(V, Err)>,
}

impl<V: Clone, Err> TrackerImpl<V, Err> {
    pub fn new(max_retries: usize) -> TrackerImpl<V, Err> {
        TrackerImpl {
            pending: vec![],
            in_progress: Default::default(),
            failed: vec![],
            max_retries,
            permanent_failure: vec![],
        }
    }
}

impl<V: Clone, Err> Default for TrackerImpl<V, Err> {
    fn default() -> Self {
        TrackerImpl {
            pending: vec![],
            in_progress: Default::default(),
            failed: vec![],
            max_retries: 0,
            permanent_failure: vec![],
        }
    }
}

impl<V: Clone, Err> Tracker<V, Err> for TrackerImpl<V, Err> {
    fn item_from_pending(&mut self) -> Option<(V, usize)> {
        self.pending.pop()
    }

    fn item_from_failed(&mut self) -> Option<(V, usize, Err)> {
        self.failed.pop()
    }

    fn move_item_to_pending(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        if let Some((v, attempt)) = self.in_progress.get(&item_id) {
            self.pending.push((v.clone(), attempt.clone()));
            Ok(())
        } else {
            Err(NotExist)
        }
    }

    fn move_item_to_failed(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist> {
        if let Some((v, attempt)) = self.in_progress.get(&item_id) {
            self.failed.push((v.clone(), attempt.clone(), err));
            Ok(())
        } else {
            Err(NotExist)
        }
    }

    fn move_incomplete_to_pending(&mut self) {
        for (_, value) in self.in_progress.drain() {
            self.pending.push(value)
        }
    }

    fn add_item_to_permanent_failure(&mut self, item: V, err: Err) {
        self.permanent_failure.push((item, err));
    }

    fn add_item_to_in_progress(&mut self, item_id: Uuid, item: V, attempt: usize) {
        self.in_progress.insert(item_id, (item, attempt));
    }

    fn delete_item_from_in_progress(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        if self.in_progress.remove(&item_id).is_none() {
            Err(NotExist)
        } else {
            Ok(())
        }
    }

    fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    fn success(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        if self.in_progress.remove(&item_id).is_none() {
            Err(NotExist)
        } else {
            Ok(())
        }
    }

    fn failure(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist> {
        let item = self.in_progress.remove(&item_id);

        if let Some((value, retries)) = item {
            self.failed.push((value, retries, err));
            Ok(())
        } else {
            Err(NotExist)
        }
    }

    fn failed_items(self) -> Vec<(V, Err)> {
        self.permanent_failure
    }
}