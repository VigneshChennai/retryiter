use std::cell::RefCell;
use std::mem;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub trait Tracker<I, Err> {
    fn item_from_failed(&mut self) -> Option<(I, usize, Option<Err>)>;

    fn add_item_to_failed(&mut self, item: I, attempt: usize, err: Option<Err>);
    fn add_item_to_permanent_failure(&mut self, item: I, err: Err);

    fn get_max_retries(&self) -> usize;

    fn failed(&mut self, item: I, attempt: usize, err: Option<Err>) {
        self.add_item_to_failed(item, attempt, err)
    }

    fn not_done(&mut self, item: I, attempt: usize) {
        self.add_item_to_failed(item, attempt - 1, None)
    }

    fn failed_items(self) -> Vec<(I, Err)>;
}

#[derive(Debug)]
pub struct TrackerData<I, Err> {
    failed: Vec<(I, usize, Option<Err>)>,
    max_retries: usize,
    permanent_failure: Vec<(I, Err)>,
}

impl<I, Err> TrackerData<I, Err> {
    pub(crate) fn new(max_retries: usize) -> TrackerData<I, Err> {
        TrackerData {
            failed: vec![],
            max_retries,
            permanent_failure: vec![],
        }
    }
}

impl<V, Err> Tracker<V, Err> for Arc<Mutex<TrackerData<V, Err>>> {
    #[inline]
    fn item_from_failed(&mut self) -> Option<(V, usize, Option<Err>)> {
        self.lock().expect("Lock poisoned").failed.pop()
    }

    #[inline]
    fn add_item_to_failed(&mut self, item: V, attempt: usize, err: Option<Err>) {
        self.lock()
            .expect("Lock poisoned")
            .failed
            .push((item, attempt, err))
    }

    #[inline]
    fn add_item_to_permanent_failure(&mut self, item: V, err: Err) {
        self.lock()
            .expect("Lock poisoned")
            .permanent_failure
            .push((item, err));
    }

    #[inline]
    fn get_max_retries(&self) -> usize {
        self.lock().expect("Lock poisoned").max_retries
    }

    fn failed_items(self) -> Vec<(V, Err)> {
        let mut guard = self.lock().expect("Lock poisoned");
        let tracker = mem::replace(guard.deref_mut(), TrackerData::new(0));

        tracker.permanent_failure
    }
}

impl<V, Err> Tracker<V, Err> for Rc<RefCell<TrackerData<V, Err>>> {
    #[inline]
    fn item_from_failed(&mut self) -> Option<(V, usize, Option<Err>)> {
        self.borrow_mut().failed.pop()
    }

    #[inline]
    fn add_item_to_failed(&mut self, item: V, attempt: usize, err: Option<Err>) {
        self.borrow_mut().failed.push((item, attempt, err))
    }

    #[inline]
    fn add_item_to_permanent_failure(&mut self, item: V, err: Err) {
        self.borrow_mut().permanent_failure.push((item, err));
    }

    #[inline]
    fn get_max_retries(&self) -> usize {
        self.borrow().max_retries
    }

    #[inline]
    fn failed_items(self) -> Vec<(V, Err)> {
        self.replace(TrackerData::new(0)).permanent_failure
    }
}
