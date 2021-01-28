pub use retryiter_arc::ArcRetryIter;
pub use retryiter_rc::RcRetryIter;

mod retryiter;
mod retryiter_rc;
mod retryiter_arc;

pub fn retry_iter<V: Clone, Itr: Iterator<Item=V>, Err>(
    iter: Itr,
    max_retries: usize,
) -> RcRetryIter<V, Itr, Err> {
    RcRetryIter::new(iter, max_retries)
}

pub fn retry_iter_sync<V: Clone, Itr: Iterator<Item=V>, Err>(
    iter: Itr,
    max_retries: usize,
) -> ArcRetryIter<V, Itr, Err> {
    ArcRetryIter::new(iter, max_retries)
}

pub trait IntoRetryIter<V: Clone, Itr: Iterator<Item=V>> {
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