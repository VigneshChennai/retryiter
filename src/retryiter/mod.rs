use std::marker::PhantomData;

use item::ItemImpl;
use tracker::Tracker;

pub mod tracker;
pub mod item;


pub struct RetryIterImpl<V, Itr, Err, T>
    where
        V: Clone,
        Itr: Iterator<Item=V>,
        T: Tracker<V, Err> + Clone {
    inner_iter: Itr,
    tracker: T,
    _marker: PhantomData<Err>,
}

impl<V, Itr, Err, T> RetryIterImpl<V, Itr, Err, T> where
    V: Clone,
    Itr: Iterator<Item=V>,
    T: Tracker<V, Err> + Clone {

    pub fn tracker(self) -> T {
        self.tracker
    }
}

impl<V, Itr, Err, T> RetryIterImpl<V, Itr, Err, T>
    where
        V: Clone,
        Itr: Iterator<Item=V>,
        T: Tracker<V, Err> + Clone {
    pub fn new(iter: Itr, tracker: T) -> RetryIterImpl<V, Itr, Err, T> {
        RetryIterImpl {
            inner_iter: iter,
            tracker,
            _marker: Default::default(),
        }
    }
}

impl<'a, Itr, V, Err, T> Iterator for &'a mut RetryIterImpl<V, Itr, Err, T>
    where
        V: Clone,
        Itr: Iterator<Item=V>,
        T: Tracker<V, Err> + Clone
{
    type Item = ItemImpl<'a, V, Err, T>;

    /// Advances the iterator and returns the next value.
    ///
    /// Returns [`None`] when iteration is finished. Individual iterator
    /// implementations may choose to resume iteration, and so calling `next()`
    /// again may or may not eventually start returning [`Some(Item)`] again at some
    /// point.
    ///
    /// [`Some(Item)`]: Some
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use retryiter::IntoRetryIter;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let mut iter = a.iter().retries::<()>(1);
    /// let mut iter_ref = &mut iter;
    /// // A call to next() returns the next value tuple of size 2 with random item_id as
    /// // first element and iter value as second element.
    ///
    /// assert_eq!(iter_ref.next().unwrap(), &1);
    /// assert_eq!(iter_ref.next().unwrap(), &2);
    /// assert_eq!(iter_ref.next().unwrap(), &3);
    ///
    /// // ... and then None once it's over.
    /// assert_eq!(None, iter_ref.next());
    ///
    /// // More calls may or may not return `None`. Here, they always will.
    /// assert_eq!(None, iter_ref.next());
    /// assert_eq!(None, iter_ref.next());
    ///
    /// ```
    ///
    /// Usage with retry:
    ///
    /// ```
    /// use retryiter::{IntoRetryIter};
    ///
    /// #[derive(Debug, Clone, PartialEq)]
    /// struct ValueError;
    ///
    /// let a = [1, 2, 3];
    ///
    /// let mut iter = a.iter().retries::<ValueError>(1);
    ///
    ///
    /// iter.for_each(|mut item| {
    ///
    ///     if item == &3 {
    ///         item.failed(ValueError);
    ///     }
    ///     item.succeeded();
    /// });
    ///
    ///
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        let item_id = uuid::Uuid::new_v4();

        loop {
            if let Some((item, attempt)) = self.tracker.item_from_pending() {
                self.tracker.add_item_to_in_progress(
                    item_id, item.clone(), attempt,
                );
                break Some(ItemImpl::<'a>::new(
                    item_id,
                    item,
                    attempt,
                    self.tracker.clone()));
            } else if let Some((item, attempt, err)) = self.tracker.item_from_failed() {
                if attempt < self.tracker.get_max_retries() + 1 {
                    self.tracker.add_item_to_in_progress(
                        item_id, item.clone(), attempt + 1,
                    );
                    break Some(ItemImpl::<'a>::new(
                        item_id,
                        item,
                        attempt + 1,
                        self.tracker.clone(),
                    ));
                } else {
                    self.tracker.add_item_to_permanent_failure(item, err);
                }
            } else {
                break match self.inner_iter.next() {
                    None => None,
                    Some(value) => {
                        self.tracker.add_item_to_in_progress(item_id, value.clone(), 1);
                        Some(ItemImpl::<'a>::new(
                            item_id,
                            value,
                            1,
                            self.tracker.clone()))
                    }
                };
            }
        }
    }
}