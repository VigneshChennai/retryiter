use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;

use crate::retryiter::tracker::Tracker;

#[derive(Debug, Clone)]
/// Status flag indicating the processed status of an [Item][crate::Item].
/// The only place we might be using it is while setting the default
/// status of an Item.
///
/// ## Example
///
/// ```
/// use retryiter::{IntoRetryIter, ItemStatus};
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
/// iter.for_each(|mut item| {
///     item.set_default(ItemStatus::Failed(ValueError));
///
///     if item < 3 {
///         item.succeeded();
///     }
///     // We don't now have to mark `failed` for all the other cases as we
///     // have set the default
/// });
/// ```
pub enum ItemStatus<Err> {
    /// Processing is successful
    Succeeded,
    /// Processing is failed
    Failed(Err),
    /// Processing is incomplete. Needs to redo.
    NotDone,
}

/// # Item
///
/// [Item][crate::Item] is a wrapper type which allow us to mark the processing
/// status of each item in Iterator.
///
#[derive(Debug)]
pub struct Item<'a, V, Err, T: Tracker<V, Err>> {
    value: ManuallyDrop<V>,
    attempt: usize,
    status: Option<ItemStatus<Err>>,
    tracker: T,
    _marker: PhantomData<Err>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, V, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    /// Initialized the [Item][crate::Item]
    pub(crate) fn new(value: V, attempt: usize, tracker: T) -> Self {
        Item {
            value: ManuallyDrop::new(value),
            attempt,
            status: None,
            tracker,
            _marker: Default::default(),
            _lifetime: Default::default(),
        }
    }
}

impl<'a, V, Err, T: Tracker<V, Err>> Drop for Item<'a, V, Err, T> {
    fn drop(&mut self) {
        let value = unsafe {
            // This is safe as we are doing it in drop implementation and
            // not using the self.value after this statement in this function.
            ManuallyDrop::take(&mut self.value)
        };
        let status = std::mem::replace(&mut self.status, None);

        match status {
            Some(ItemStatus::Succeeded) | None => { /* No operation on success */ }
            Some(ItemStatus::Failed(err)) => {
                if self.tracker.get_max_retries() + 1 > self.attempt {
                    self.tracker.failed(value, self.attempt, Some(err))
                } else {
                    self.tracker.add_item_to_permanent_failure(value, err)
                }
            }
            Some(ItemStatus::NotDone) => self.tracker.not_done(value, self.attempt),
        };
    }
}

impl<'a, V: PartialEq, Err, T: Tracker<V, Err>> PartialEq<Self> for Item<'a, V, Err, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.deref() == other.value.deref()
    }
}

impl<'a, V: PartialEq, Err, T: Tracker<V, Err>> PartialEq<V> for Item<'a, V, Err, T> {
    fn eq(&self, other: &V) -> bool {
        self.value.deref() == other
    }
}

impl<'a, V: PartialOrd, Err, T: Tracker<V, Err>> PartialOrd<Self> for Item<'a, V, Err, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.deref().partial_cmp(other.value.deref())
    }
}

impl<'a, V: PartialOrd, Err, T: Tracker<V, Err>> PartialOrd<V> for Item<'a, V, Err, T> {
    fn partial_cmp(&self, other: &V) -> Option<Ordering> {
        self.value.deref().partial_cmp(other)
    }
}

impl<'a, V: Eq, Err, T: Tracker<V, Err>> Eq for Item<'a, V, Err, T> {}

impl<'a, V: Ord, Err, T: Tracker<V, Err>> Ord for Item<'a, V, Err, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.deref().cmp(other.value.deref())
    }
}

impl<'a, V, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    /// Returns the current attempt count of the [item][crate::Item]
    pub fn attempt(&self) -> usize {
        self.attempt
    }

    /// Marks the processing of [item][crate::Item] as successful.
    pub fn succeeded(mut self) {
        self.status = Some(ItemStatus::Succeeded);
    }

    /// Marks the processing of [item][crate::Item] as failed.
    pub fn failed(mut self, err: Err) {
        self.status = Some(ItemStatus::Failed(err));
    }

    /// Modifying the default Item status of [item][crate::Item].
    pub fn set_default(&mut self, status: ItemStatus<Err>) {
        self.status = Some(status)
    }
}

impl<'a, V: Clone, Err, T: Tracker<V, Err>> Item<'a, V, Err, T> {
    /// Clones the inner value wrapped by the [Item][crate::Item]
    /// and returns it.
    pub fn value(&self) -> V {
        self.value.deref().clone()
    }
}
