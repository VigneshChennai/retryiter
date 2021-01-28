pub use retryiter_arc::ArcRetryIter;
pub use retryiter_rc::RcRetryIter;

mod retryiter;
mod retryiter_rc;
mod retryiter_arc;


fn retry_iter<V: Clone, Itr: Iterator<Item=V>, Err>(
    iter: Itr,
    max_retries: usize,
) -> RcRetryIter<V, Itr, Err> {
    RcRetryIter::new(iter, max_retries)
}

fn retry_iter_sync<V: Clone, Itr: Iterator<Item=V>, Err>(
    iter: Itr,
    max_retries: usize,
) -> ArcRetryIter<V, Itr, Err> {
    ArcRetryIter::new(iter, max_retries)
}

pub trait IntoRetryIter<V: Clone, Itr: Iterator<Item=V>> {
    /// Adds retry support to the Iterator
    ///
    /// # Examples
    ///
    /// ```
    /// use retryiter::{IntoRetryIter};
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct ValueError;
    ///
    /// let a = [1, 2, 3];
    ///
    /// // Initializing retryiter with retry count 1.
    /// let mut iter = a.into_iter().retries::<ValueError>(1);
    ///
    /// iter.for_each(|mut item| {
    ///
    ///     if item == &3 {
    ///         item.failed(ValueError);
    ///     } else if item == &2 && item.attempt() == 1 {
    ///         item.failed(ValueError);
    ///     }
    ///     item.succeeded();
    /// });
    ///
    /// assert_eq!(vec![(&3, ValueError)], iter.failed_items())
    /// ```
    fn retries<Err: Clone>(self, max_retries: usize) -> RcRetryIter<V, Itr, Err>;
    fn retries_sync<Err: Clone>(self, max_retries: usize) -> ArcRetryIter<V, Itr, Err>;
}

impl<V: Clone, Itr: Iterator<Item=V>> IntoRetryIter<V, Itr> for Itr {
    fn retries<Err: Clone>(self, max_retries: usize) -> RcRetryIter<V, Itr, Err> {
        retry_iter(self, max_retries)
    }

    fn retries_sync<Err: Clone>(self, max_retries: usize) -> ArcRetryIter<V, Itr, Err> {
        retry_iter_sync(self, max_retries)
    }
}