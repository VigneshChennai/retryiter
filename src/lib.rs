pub use crate::retryiter::item::{Item, ItemStatus};
pub use crate::retryiter::tracker::TrackerImpl;
pub use crate::retryiter_arc::ArcRetryIter;
pub use crate::retryiter_rc::RcRetryIter;

mod retryiter;
mod retryiter_rc;
mod retryiter_arc;


pub trait IntoRetryIter<V: Clone, Itr: Iterator<Item=V>> {
    /// Adds retry support to the Iterator
    ///
    /// # Examples:
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
    ///         item.failed(Some(ValueError));
    ///     } else if item == 2 && item.attempt() == 1 {
    ///         item.failed(Some(ValueError));
    ///     } else {
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
