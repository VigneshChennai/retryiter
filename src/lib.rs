#![deny(missing_docs)]

pub use crate::retryiter::item::{Item, ItemStatus};
pub use crate::retryiter::tracker::TrackerImpl;
pub use crate::retryiter_arc::ArcRetryIter;
pub use crate::retryiter_rc::RcRetryIter;

mod retryiter;
mod retryiter_rc;
mod retryiter_arc;


pub trait IntoRetryIter<V: Clone, Itr: Iterator<Item=V>> {
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
    /// ```
    /// use retryiter::{IntoRetryIter};
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct ValueError;
    ///
    /// let a = vec![1, 2, 3];
    ///
    /// // Initializing retryiter with retry count 1.
    /// let mut iter = a.into_iter().retries::<ValueError>(1);
    ///
    /// iter.for_each(|mut item| {
    ///
    ///     if item == 3 {
    ///         // Always failing for value 3.
    ///         item.failed(Some(ValueError));
    ///     } else if item == 2 && item.attempt() == 1 {
    ///         // Only failure on first attempt. The item with value 2 will
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
    fn retries<Err: Clone>(self, max_retries: usize) -> RcRetryIter<V, Itr, Err>;
    fn par_retries<Err: Clone>(self, max_retries: usize) -> ArcRetryIter<V, Itr, Err>;
}

impl<V: Clone, Itr: Iterator<Item=V>> IntoRetryIter<V, Itr> for Itr {
    fn retries<Err: Clone>(self, max_retries: usize) -> RcRetryIter<V, Itr, Err> {
        retry_iter(self, max_retries)
    }

    fn par_retries<Err: Clone>(self, max_retries: usize) -> ArcRetryIter<V, Itr, Err> {
        retry_iter_par(self, max_retries)
    }
}

fn retry_iter<V: Clone, Itr: Iterator<Item=V>, Err>(
    iter: Itr,
    max_retries: usize,
) -> RcRetryIter<V, Itr, Err> {
    RcRetryIter::new(iter, max_retries)
}

fn retry_iter_par<V: Clone, Itr: Iterator<Item=V>, Err>(
    iter: Itr,
    max_retries: usize,
) -> ArcRetryIter<V, Itr, Err> {
    ArcRetryIter::new(iter, max_retries)
}
