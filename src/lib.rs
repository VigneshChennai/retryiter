#![deny(missing_docs)]

//! retryiter is a small helper crate which adds retry support to [Iterator][std::iter::Iterator].
//! It does it by implementing the crate's [IntoRetryIter][crate::IntoRetryIter]
//! for all [std::iter::Iterator] types. In addition to retries support, the main
//! feature of this crate is to preserve the iterator items during [Future][std::future::Future]
//! cancellation in asynchronous processing. It is explained with example in
//! [Crate's Main Feature](#crates-main-feature)
//!
//! The [IntoRetryIter][crate::IntoRetryIter] has two methods
//!
//! 1. [retries][crate::IntoRetryIter::retries] - To create a lite
//!    [RetryIter][crate::RetryIter] which
//!    can only be used to process Iterator's Item within a single thread.
//! 2. [par_retries][crate::IntoRetryIter::par_retries] - To create a
//!    [RetryIter][crate::RetryIter]
//!    which can be used to process Iterator's Item in parallel threads.
//!
//! #### Example
//!
//! ```
//! use retryiter::{IntoRetryIter};
//!
//! #[derive(Debug, Clone, PartialEq)]
//! struct ValueError;
//!
//! let a = vec![1, 2, 3];
//!
//! // Initializing retryiter with retry count 1.
//! // Also defined the error that can occur in while processing the item.
//! let mut iter = a.into_iter().retries::<ValueError>(1);
//!
//! for item in &mut iter {
//!     if item == 3 {
//!         // Always failing for value 3.
//!         item.failed(ValueError);
//!     } else if item < 3 && item.attempt() == 1 {
//!         // Only fail on first attempt. The item with value 1 or 2 will
//!         // succeed on second attempt.
//!         item.failed(ValueError);
//!     } else {
//!         // Marking success for all the other case.
//!         item.succeeded();
//!     }
//! }
//! assert_eq!(vec![(3, ValueError)], iter.failed_items())
//! ```
//!
//! There are few things that requires more explanation in the above example.
//!
//! **First** `let mut iter = a.into_iter().retries::<ValueError>(1)` line. We here initializing
//! the retryiter with retry count as 1 and also defining the possible error that can occur during
//! processing of iterator's items, as `ValueError`. But there is no constrained set on the
//! Error type. So, if we don't want to capture
//! any error during the failure, we can set the error type to `()`
//!
//! #### Example:
//!
//! The below one should work just fine.
//! ```
//! use retryiter::{IntoRetryIter};
//! let a = vec![1, 2, 3];
//! let mut _iter = a.into_iter().retries::<()>(1);
//! ```
//!
//! **Second** is this line `iter.for_each(|mut item| {`.
//! `item` variable here is a wrapper type [Item][crate::Item] which wraps
//! the item of type i32 (in our example) in original iter.
//! The [Item][crate::Item] implements both Deref and DerefMut,
//! also implements PartialEq, PartialOrd, Eq and Ord if the original iterator's item
//! implemented it.
//!
//! So, operations like ==, !=, >, <, >= and <= should works as long as the
//! item is the LHS.
//!
//! **Example:**
//!
//! ```compile_fail
//! item == 3 // will work.
//! 3 == item // won't work.
//! ```
//!
//! # Item
//!
//! [Item][crate::Item] is a wrapper type which allow us to mark the processing
//! status of each item in Iterator. [Item][crate::Item] has below two methods which
//! allow us to mark the status.
//!
//! 1. [succeeded][crate::Item::succeeded]
//! 2. [failed][crate::Item::failed]
//!
//!
//! ### Default Item Status
//!
//! When an [Item][crate::Item] neither marked as 'succeeded' or 'failed' during the
//! processing, then the status is considered as 'succeeded' by default.
//! To change this default, we can use the [Item][crate::Item]'s ([set_default][crate::Item])
//! method.
//!
//! #### Example:
//!
//! In the below example, the default is set to [Failed][crate::ItemStatus::Failed]]
//! ```
//! use retryiter::{IntoRetryIter, ItemStatus};
//!
//! let a = vec![1, 2, 3];
//!
//! let mut iter = a.into_iter().retries::<()>(1);
//!
//! iter.for_each(|mut item| {
//!     item.set_default(ItemStatus::Failed(()));
//!
//!     if item != 3 {
//!         // Success for all items except 3
//!         item.succeeded();
//!     }
//! });
//!
//! assert_eq!(vec![(3, ())], iter.failed_items())
//! ```
//!
//! # Crate's Main Feature
//!
//! We may wonder why we need a separate third party crate for retry when we can do the
//! same in a simplest way like below.
//!
//! ```
//! let a = vec![1, 2, 3];
//!
//! #[derive(Debug, Clone, PartialEq)]
//! struct ValueError;
//!
//! let mut failed = Vec::<(i32, ValueError)>::new();
//!
//! a.into_iter().for_each(|item| {
//!     let mut tries = 0;
//!     while tries < 3 {
//!         tries += 1;
//!
//!         // Doing our processing here.
//!         // .....
//!         // Handling error
//!         if item == 3 {
//!             failed.push((item, ValueError));
//!         }
//!     }
//! })
//! ```
//!
//! But this crate has one additional feature.
//! In addition to "succeeded" and "failed", there is a third status called "not done".
//! When an [Item][crate::Item] is marked as "not done", it will be retried again like how it will
//! happen when the status is set to "failed" but without
//! increasing the attempt count.
//!
//! It is not be very useful in synchronous code but when it come to asynchronous
//! code, it will be very handy during [std::future::Future] cancellations/drop.
//!
//! Look at the example below to under the behaviour with different default [crate::Item] status
//! during [std::future::Future] cancellation.
//!
//! ```
//! use tokio::runtime::Runtime;
//! use futures::StreamExt;
//! use retryiter::{ItemStatus, IntoRetryIter};
//! use std::time::Duration;
//! use tokio::time::Instant;
//!
//! let runtime = Runtime::new().unwrap();
//!
//! async fn abrupt_future_cancellation(input: Vec<i32>, default_status: ItemStatus<()>)
//! -> (Vec<i32>, Vec<(i32, ())>) {
//!
//!     // Initializing retryiter with retry count 0.
//!     let mut iter = input.into_iter().retries::<()>(0);
//!     let stream = futures::stream::iter(&mut iter);
//!
//!     // Doing 100 tasks in parallel.
//!     let concurrent_task = stream.for_each_concurrent(100, |mut item| {
//!         item.set_default(default_status.clone());
//!
//!         // Sleeping the task to ensure none of the task get to completion
//!         tokio::time::sleep(Duration::from_secs(50))
//!     });
//!
//!     // Cancelling the future 'concurrent_task' within 100 millisecond
//!     let result = tokio::time::timeout_at(
//!         Instant::now() + Duration::from_millis(100),
//!         concurrent_task,
//!     ).await;
//!
//!     let mut remaining_input: Vec<i32> = iter.map(|v| v.value()).collect();
//!     remaining_input.sort();
//!
//!     // Checking failed items
//!     let mut failed_items = iter.failed_items();
//!     // Sorting by values only.
//!     failed_items.sort_by(|f, s| f.0.cmp(&s.0));
//!
//!     (remaining_input, failed_items)
//! }
//!
//! // Default set to ItemStatus::Success
//! let (remaining_input, failed_items) = runtime.block_on(abrupt_future_cancellation(
//!     vec![1, 2, 3, 4, 5, 6],
//!     ItemStatus::Succeeded,
//! ));
//!
//! assert_eq!(remaining_input, vec![]); // All input lost
//! // The items which where in progress are not marked as failure as well.
//! assert_eq!(failed_items, vec![]);
//!
//! // Default set to ItemStatus::Failed(None)
//! let (remaining_input, failed_items) = runtime.block_on(abrupt_future_cancellation(
//!     vec![1, 2, 3, 4, 5, 6],
//!     ItemStatus::Failed(()),
//! ));
//!
//! assert_eq!(remaining_input, vec![]); // All input lost
//! // The items which where in progress are marked as failed items
//! assert_eq!(
//!     failed_items,
//!     vec![(1, ()), (2, ()), (3, ()), (4, ()), (5, ()), (6, ())]
//! );
//!
//! // Default set to ItemStatus::NotDone
//! let (remaining_input, failed_items) = runtime.block_on(abrupt_future_cancellation(
//!     vec![1, 2, 3, 4, 5, 6],
//!     ItemStatus::NotDone,
//! ));
//!
//! // All un done input preserved in case of future drop
//! assert_eq!(remaining_input, vec![1, 2, 3, 4, 5, 6]);
//! assert_eq!(failed_items, vec![]);
//!
//! ```
//! # Notes
//!
//! Unlike normal iterator, the [Item][crate::Item] from [RetryIter][crate::RetryIter] can't outlive
//! the Iterator.
//!
//! ```compile_fail
//! use tokio::runtime::Runtime;
//! use crate::IntoRetryIter;
//!
//!
//! let runtime = Runtime::new().unwrap();
//! runtime.block_on(async move {
//!     let value = vec![1, 2, 3];
//!     let mut retryiter = value.into_iter().retries::<()>(1);
//!     let item = (&mut retryiter).next().unwrap();
//!
//!     // Dropping `retryiter` before `item` prevents the code from compiling.
//!     // Comment out the below line to make the code work.
//!     drop(retryiter);
//!
//!     assert_eq!(item.value(), 1)
//! });
//!
//! ```
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::retryiter::tracker::TrackerData;
pub use crate::retryiter::{
    item::{Item, ItemStatus},
    RetryIter,
};

mod retryiter;

type RcTrackerData<V, Err> = Rc<RefCell<TrackerData<V, Err>>>;
type ArcTrackerData<V, Err> = Arc<Mutex<TrackerData<V, Err>>>;

/// Conversion to [RetryIter][crate::RetryIter]
pub trait IntoRetryIter<V, Itr: Iterator<Item = V>> {
    /// Instantiates a [RetryIter][crate::RetryIter] which can be used to process
    /// Iterator's Item only within a single thread.
    ///  #### Example
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
    /// let mut iter = a.into_iter().retries::<ValueError>(1);
    ///
    /// for item in &mut iter {
    ///     if item == 3 {
    ///         // Always failing for value 3.
    ///         item.failed(ValueError);
    ///     } else if item < 3 && item.attempt() == 1 {
    ///         // Only fail on first attempt. The item with value 1 or 2 will
    ///         // succeed on second attempt.
    ///         item.failed(ValueError);
    ///     } else {
    ///         // Marking success for all the other case.
    ///         item.succeeded();
    ///     }
    /// }
    /// assert_eq!(vec![(3, ValueError)], iter.failed_items())
    /// ```
    fn retries<Err>(self, max_retries: usize) -> RetryIter<V, Itr, Err, RcTrackerData<V, Err>>;

    /// Instantiates a [RetryIter][crate::RetryIter] which can be used to process
    /// Iterator's Item in parallel threads.
    ///
    /// #### Example
    ///
    /// ```
    /// use retryiter::{IntoRetryIter, Item};
    /// use std::thread;
    /// use rayon::iter::{ParallelIterator, ParallelBridge};
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct ValueError;
    ///
    /// let a = (0..100);
    ///
    /// // Initializing retryiter with retry count 1.
    /// // Also defined the error that can occur in while processing the item.
    /// let mut iter = a.into_iter().par_retries::<ValueError>(1);
    ///
    /// (&mut iter).par_bridge().for_each(|item| {
    ///     if item == 3 {
    ///         // Always failing for value 3.
    ///         item.failed(ValueError);
    ///     } else if item < 3 && item.attempt() == 1 {
    ///         // Only fail on first attempt. The item with value 1 or 2 will
    ///         // succeed on second attempt.
    ///         item.failed(ValueError);
    ///     } else {
    ///         // Marking success for all the other case.
    ///         item.succeeded();
    ///     }
    /// });
    ///
    /// assert_eq!(vec![(3, ValueError)], iter.failed_items())
    /// ```
    ///
    fn par_retries<Err>(self, max_retries: usize)
        -> RetryIter<V, Itr, Err, ArcTrackerData<V, Err>>;
}

impl<V, Itr: Iterator<Item = V>> IntoRetryIter<V, Itr> for Itr {
    fn retries<Err>(self, max_retries: usize) -> RetryIter<V, Itr, Err, RcTrackerData<V, Err>> {
        RetryIter::new(self, Rc::new(RefCell::new(TrackerData::new(max_retries))))
    }

    fn par_retries<Err>(
        self,
        max_retries: usize,
    ) -> RetryIter<V, Itr, Err, ArcTrackerData<V, Err>> {
        RetryIter::new(self, Arc::new(Mutex::new(TrackerData::new(max_retries))))
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
            vec![(1, ValueError), (2, ValueError), (3, ValueError)],
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
                (1, ValueError),
                (2, ValueError),
                (3, ValueError),
                (4, ValueError),
                (5, ValueError),
                (6, ValueError),
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
        expected_failed_items: Vec<(T, ValueError)>,
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
                        "failed" => ItemStatus::Failed(ValueError),
                        "not_done" => ItemStatus::NotDone,
                        _ => ItemStatus::Succeeded,
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
