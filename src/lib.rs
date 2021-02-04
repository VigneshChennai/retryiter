//#![deny(missing_docs)]

use std::cell::RefCell;
use std::mem;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub use crate::retryiter::item::{Item, ItemStatus};
use crate::retryiter::tracker::Tracker;
pub use crate::retryiter::tracker::TrackerImpl;
use crate::retryiter::RetryIter;

mod retryiter;

pub trait IntoRetryIter<V, Itr: Iterator<Item = V>> {
    /// Adds retry support to any [std::iter::Iterator].
    ///
    /// **Note:** The iterator returned doesn't support sending across
    /// threads, if you need to process iterator values in parallel, use
    /// [IntoRetryIter::par_retires].
    ///
    /// # Examples 1: Common use case.
    ///
    /// ```
    /// use retryiter::{IntoRetryIter};
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct ValueError;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// // Initializing retryiter with retry count 1.
    /// // Also defined the error that can occur in while processing the item.
    /// //
    /// // There is no constrained set on the Error type. So, if we don't want to capture
    /// // any error during the failure, we can set the type to `()`
    /// //
    /// // Example:
    /// //
    /// // let mut iter = a.into_iter().retries::<()>(1);
    /// //
    /// let mut iter = a.into_iter().retries::<ValueError>(1);
    ///
    /// iter.for_each(|mut item| {
    ///
    ///     // item variable here is a wrapper type [retryiter::Item] which wraps
    ///     // the original items in iter.
    ///     // The retryiter::Item type implements both Deref and DerefMut,
    ///     // also implements PartialEq and PartialOrd for original iterator value type.
    ///     //
    ///     // So, operations like ==, !=, >, <, >= and <= should works as long as the
    ///     // item is the LHS.
    ///     //
    ///     // Example:
    ///     //
    ///     // item == 3 // will work.
    ///     // 3 == item // won't work.
    ///
    ///     if item == 3 {
    ///         // Always failing for value 3.
    ///         item.failed(Some(ValueError));
    ///     } else if item < 3 && item.attempt() == 1 {
    ///         // Only fail on first attempt. The item with value 1 or 2 will
    ///         // succeed on second attempt.
    ///         item.failed(Some(ValueError));
    ///     } else {
    ///         // Marking success for all the other case.
    ///         item.succeeded();
    ///     }
    /// });
    ///
    /// assert_eq!(vec![(3, Some(ValueError))], iter.failed_items())
    /// ```
    ///
    /// # Examples 2: Not Done status
    ///
    /// In addition to "succeeded" and "failed", there is a third status called "not done".
    /// When an [crate::Item] is marked as "not done", it will be retried again like how it will
    /// happen when the status is set to "failed" but without
    /// increasing the attempt count.
    ///
    /// It is might not be very useful in synchronous code but when it come to asynchronous
    /// code, it will be very handy during [std::future:Future] cancellations/drop.
    ///
    fn retries<Err: Clone>(
        self,
        max_retries: usize,
    ) -> RetryIter<V, Itr, Err, Rc<RefCell<TrackerImpl<V, Err>>>>;
    fn par_retries<Err: Clone>(
        self,
        max_retries: usize,
    ) -> RetryIter<V, Itr, Err, Arc<Mutex<TrackerImpl<V, Err>>>>;
}

impl<V, Itr: Iterator<Item = V>> IntoRetryIter<V, Itr> for Itr {
    fn retries<Err: Clone>(
        self,
        max_retries: usize,
    ) -> RetryIter<V, Itr, Err, Rc<RefCell<TrackerImpl<V, Err>>>> {
        RetryIter::new(self, Rc::new(RefCell::new(TrackerImpl::new(max_retries))))
    }

    fn par_retries<Err: Clone>(
        self,
        max_retries: usize,
    ) -> RetryIter<V, Itr, Err, Arc<Mutex<TrackerImpl<V, Err>>>> {
        RetryIter::new(self, Arc::new(Mutex::new(TrackerImpl::new(max_retries))))
    }
}

impl<V, Err> Tracker<V, Err> for Arc<Mutex<TrackerImpl<V, Err>>> {
    #[inline]
    fn item_from_failed(&mut self) -> Option<(V, usize, Option<Err>)> {
        self.lock().expect("Lock poisoned").item_from_failed()
    }

    #[inline]
    fn add_item_to_failed(&mut self, item: V, attempt: usize, err: Option<Err>) {
        self.lock()
            .expect("Lock poisoned")
            .add_item_to_failed(item, attempt, err)
    }

    #[inline]
    fn add_item_to_permanent_failure(&mut self, item: V, err: Option<Err>) {
        self.lock()
            .expect("Lock poisoned")
            .add_item_to_permanent_failure(item, err)
    }

    #[inline]
    fn get_max_retries(&self) -> usize {
        self.lock().expect("Lock poisoned").get_max_retries()
    }

    fn failed_items(self) -> Vec<(V, Option<Err>)> {
        let mut guard = self.lock().expect("Lock poisoned");
        let tracker = mem::replace(guard.deref_mut(), Default::default());

        return tracker.failed_items();
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

#[cfg(test)]
mod test_async_future_cancellation {
    use std::fmt::Debug;
    use std::iter::Iterator;
    use std::time::Duration;

    use futures::stream::StreamExt;
    use tokio::runtime::Runtime;
    use tokio::time::Instant;

    use crate::{IntoRetryIter, ItemStatus};

    #[test]
    fn default_success_item_status_no_resume() {
        default_item_status(
            "success",
            vec![1, 2, 3, 4, 5, 6],
            vec![],
            vec![4, 5, 6],
            1,
            3,
            0,
        )
    }

    #[test]
    fn default_success_item_status_with_resume() {
        default_item_status("success", vec![1, 2, 3, 4, 5, 6], vec![], vec![], 2, 3, 0)
    }

    #[test]
    fn default_failed_item_status_no_resume() {
        default_item_status(
            "failed",
            vec![1, 2, 3, 4, 5, 6],
            vec![(1, None), (2, None), (3, None)],
            vec![4, 5, 6],
            1,
            3,
            0,
        )
    }

    #[test]
    fn default_failed_item_status_with_resume() {
        default_item_status(
            "failed",
            vec![1, 2, 3, 4, 5, 6],
            vec![
                (1, None),
                (2, None),
                (3, None),
                (4, None),
                (5, None),
                (6, None),
            ],
            vec![],
            4,
            3,
            1,
        )
    }

    #[test]
    fn default_not_done_item_status_no_resume() {
        default_item_status(
            "not_done",
            vec![1, 2, 3, 4, 5, 6],
            vec![],
            vec![4, 5, 6, 3, 2, 1],
            1,
            3,
            1,
        )
    }

    #[test]
    fn default_not_done_item_status_with_resume() {
        default_item_status(
            "not_done",
            vec![1, 2, 3, 4, 5, 6],
            vec![],
            vec![6, 5, 4, 3, 2, 1],
            6,
            3,
            1,
        )
    }

    #[derive(Debug, Clone, PartialEq)]
    struct ValueError;

    fn default_item_status<T: std::cmp::Eq + std::cmp::Ord + Clone + Debug>(
        default_status: &str,
        input_items: Vec<T>,
        expected_failed_items: Vec<(T, Option<ValueError>)>,
        expected_remaining_items: Vec<T>,
        iteration: usize,
        concurrency: usize,
        retries: usize,
    ) {
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async move {
            let iter = input_items.into_iter();

            // Initializing retryiter with retry count 1.
            let mut iter = iter.retries::<ValueError>(retries);
            for _ in 1..=iteration {
                let stream = futures::stream::iter(&mut iter);

                let concurrent_task = stream.for_each_concurrent(concurrency, |mut item| {
                    item.set_default(match default_status {
                        "failed" => ItemStatus::Failed(None),
                        "success" => ItemStatus::Success,
                        "not_done" => ItemStatus::NotDone,
                        _ => ItemStatus::None,
                    });
                    tokio::time::sleep(Duration::from_secs(45))
                });

                let result = tokio::time::timeout_at(
                    Instant::now() + Duration::from_millis(100),
                    concurrent_task,
                )
                .await;

                if result.is_ok() {
                    break;
                }
            }

            // Checking remaining items
            assert_eq!(
                iter.map(|v| v.value()).collect::<Vec<T>>(),
                expected_remaining_items
            );

            // Checking failed items
            let mut failed_items = iter.failed_items();
            failed_items.sort_by(|f, s| f.0.cmp(&s.0));
            assert_eq!(failed_items, expected_failed_items);
        });
    }
}
