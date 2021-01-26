use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

use thiserror::Error;
use uuid::Uuid;

pub fn retry_iter_sync<V: Clone, T: Iterator<Item = V>, Err: Clone>(
    iter: T,
    max_reties: usize,
) -> ArcRetryIter<V, T, Err> {
    ArcRetryIter {
        inner: Arc::new(Mutex::new(RetryIterImpl::new(iter, max_reties))),
    }
}

pub fn retry_iter<V: Clone, T: Iterator<Item = V>, Err: Clone>(
    iter: T,
    max_reties: usize,
) -> RcRetryIter<V, T, Err> {
    RcRetryIter {
        inner: Rc::new(RefCell::new(RetryIterImpl::new(iter, max_reties))),
    }
}

#[derive(Debug, Error)]
#[error("Item doesn't exists")]
pub struct NotExist;

pub trait RetryIter<V: Clone, T: Iterator<Item = V>, Err: Clone>: Iterator {
    fn succeeded(&mut self, item_id: Uuid) -> Result<(), NotExist>;

    fn failed(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist>;

    fn failed_items(&self) -> Vec<(V, Err)>;

    fn requeue_in_progress_items(&mut self);
}

struct Tracker<V: Clone, Err: Clone> {
    pending: Vec<(V, usize)>,
    in_progress: HashMap<Uuid, (V, usize)>,
    failed: Vec<(V, usize, Err)>,
    max_retries: usize,
    permanent_failure: Vec<(V, Err)>,
}

struct RetryIterImpl<V: Clone, T: Iterator<Item = V>, Err: Clone> {
    inner_iter: T,
    status_tracker: Tracker<V, Err>,
}

impl<T, V, Err> Iterator for RetryIterImpl<V, T, Err>
where
    V: Clone,
    Err: Clone,
    T: Iterator<Item = V>,
{
    type Item = (Uuid, V);

    fn next(&mut self) -> Option<Self::Item> {
        let item_id = uuid::Uuid::new_v4();

        loop {
            if let Some((item, retries)) = self.status_tracker.pending.pop() {
                self.status_tracker
                    .in_progress
                    .insert(item_id, (item.clone(), retries));
                break Some((item_id, item));
            } else if let Some((item, retries, err)) = self.status_tracker.failed.pop() {
                if retries < self.status_tracker.max_retries + 1 {
                    self.status_tracker
                        .in_progress
                        .insert(item_id, (item.clone(), retries + 1));
                    break Some((item_id, item));
                } else {
                    self.status_tracker.permanent_failure.push((item, err));
                }
            } else {
                break match self.inner_iter.next() {
                    None => None,
                    Some(v) => {
                        self.status_tracker
                            .in_progress
                            .insert(item_id, (v.clone(), 1));
                        Some((item_id, v))
                    }
                };
            }
        }
    }
}

impl<V: Clone, T: Iterator<Item = V>, Err: Clone> RetryIter<V, T, Err>
    for RetryIterImpl<V, T, Err>
{
    fn succeeded(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        if self.status_tracker.in_progress.remove(&item_id).is_none() {
            Err(NotExist)
        } else {
            Ok(())
        }
    }

    fn failed(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist> {
        let item = self.status_tracker.in_progress.remove(&item_id);

        if let Some((value, retries)) = item {
            self.status_tracker.failed.push((value, retries, err));
            Ok(())
        } else {
            Err(NotExist)
        }
    }

    fn failed_items(&self) -> Vec<(V, Err)> {
        self.status_tracker.permanent_failure.clone()
    }

    fn requeue_in_progress_items(&mut self) {
        for (_, value) in self.status_tracker.in_progress.drain() {
            self.status_tracker.pending.push(value)
        }
    }
}

impl<T, V, Err> RetryIterImpl<V, T, Err>
where
    V: Clone,
    Err: Clone,
    T: Iterator<Item = V>,
{
    fn new(iter: T, max_reties: usize) -> Self {
        RetryIterImpl {
            inner_iter: iter,
            status_tracker: Tracker {
                in_progress: HashMap::new(),
                pending: vec![],
                failed: vec![],
                max_retries: max_reties,
                permanent_failure: vec![],
            },
        }
    }
}

pub struct RcRetryIter<V: Clone, T: Iterator<Item = V>, Err: Clone> {
    inner: Rc<RefCell<RetryIterImpl<V, T, Err>>>,
}

impl<V: Clone, T: Iterator<Item = V>, Err: Clone> Clone for RcRetryIter<V, T, Err> {
    fn clone(&self) -> Self {
        RcRetryIter {
            inner: self.inner.clone(),
        }
    }
}

impl<T, V, Err> Iterator for RcRetryIter<V, T, Err>
where
    V: Clone,
    Err: Clone,
    T: Iterator<Item = V>,
{
    type Item = (Uuid, V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut retry_iter = self.inner.as_ref().borrow_mut();
        retry_iter.next()
    }
}

impl<V: Clone, T: Iterator<Item = V>, Err: Clone> RetryIter<V, T, Err> for RcRetryIter<V, T, Err>
where
    V: Clone,
    Err: Clone,
    T: Iterator<Item = V>,
{
    fn succeeded(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        self.inner.as_ref().borrow_mut().succeeded(item_id)
    }

    fn failed(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist> {
        self.inner.as_ref().borrow_mut().failed(item_id, err)
    }

    fn failed_items(&self) -> Vec<(V, Err)> {
        self.inner.as_ref().borrow().failed_items()
    }

    fn requeue_in_progress_items(&mut self) {
        self.inner
            .as_ref()
            .borrow_mut()
            .requeue_in_progress_items();
    }
}

impl<V: Clone, T: Iterator<Item = V>, Err: Clone> RcRetryIter<V, T, Err> {
    pub fn new(iter: T, max_retries: usize) -> Self {
        retry_iter(iter, max_retries)
    }
}

pub struct ArcRetryIter<V: Clone, T: Iterator<Item = V>, Err: Clone> {
    inner: Arc<Mutex<RetryIterImpl<V, T, Err>>>,
}

impl<V: Clone, T: Iterator<Item = V>, Err: Clone> Clone for ArcRetryIter<V, T, Err> {
    fn clone(&self) -> Self {
        ArcRetryIter {
            inner: self.inner.clone(),
        }
    }
}

impl<T, V, Err> Iterator for ArcRetryIter<V, T, Err>
where
    V: Clone,
    Err: Clone,
    T: Iterator<Item = V>,
{
    type Item = (Uuid, V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut retry_iter_guard = self.inner.lock().expect("Lock poisoned!");
        retry_iter_guard.next()
    }
}

impl<V: Clone, T: Iterator<Item = V>, Err: Clone> RetryIter<V, T, Err> for ArcRetryIter<V, T, Err> {
    fn succeeded(&mut self, item_id: Uuid) -> Result<(), NotExist> {
        self.get_inner_guard().succeeded(item_id)
    }

    fn failed(&mut self, item_id: Uuid, err: Err) -> Result<(), NotExist> {
        self.get_inner_guard().failed(item_id, err)
    }

    fn failed_items(&self) -> Vec<(V, Err)> {
        self.get_inner_guard().failed_items()
    }

    fn requeue_in_progress_items(&mut self) {
        self.get_inner_guard().requeue_in_progress_items();
    }
}

impl<V: Clone, T: Iterator<Item = V>, Err: Clone> ArcRetryIter<V, T, Err> {
    pub fn new(iter: T, max_retries: usize) -> Self {
        retry_iter_sync(iter, max_retries)
    }

    fn get_inner_guard(&self) -> MutexGuard<RetryIterImpl<V, T, Err>> {
        self.inner.lock().expect("Mutex poisoned in ArcRetryIter!")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
