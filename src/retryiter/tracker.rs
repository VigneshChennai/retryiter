pub trait Tracker<I, Err> {
    fn item_from_failed(&mut self) -> Option<(I, usize, Option<Err>)>;

    fn add_item_to_failed(&mut self, item: I, attempt: usize, err: Option<Err>);
    fn add_item_to_permanent_failure(&mut self, item: I, err: Option<Err>);

    fn get_max_retries(&self) -> usize;

    fn failed(&mut self, item: I, attempt: usize, err: Option<Err>) {
        self.add_item_to_failed(item, attempt, err)
    }

    fn not_done(&mut self, item: I, attempt: usize) {
        self.add_item_to_failed(item, attempt - 1, None)
    }

    fn failed_items(self) -> Vec<(I, Option<Err>)>;
}

#[derive(Debug)]
pub struct TrackerImpl<I, Err> {
    failed: Vec<(I, usize, Option<Err>)>,
    max_retries: usize,
    permanent_failure: Vec<(I, Option<Err>)>,
}

impl<I, Err> TrackerImpl<I, Err> {
    pub fn new(max_retries: usize) -> TrackerImpl<I, Err> {
        TrackerImpl {
            failed: vec![],
            max_retries,
            permanent_failure: vec![],
        }
    }
}

impl<I, Err> Default for TrackerImpl<I, Err> {
    fn default() -> Self {
        TrackerImpl::new(0)
    }
}

impl<I, Err> Tracker<I, Err> for TrackerImpl<I, Err> {
    fn item_from_failed(&mut self) -> Option<(I, usize, Option<Err>)> {
        self.failed.pop()
    }

    fn add_item_to_failed(&mut self, item: I, attempt: usize, err: Option<Err>) {
        self.failed.push((item, attempt, err))
    }

    fn add_item_to_permanent_failure(&mut self, item: I, err: Option<Err>) {
        self.permanent_failure.push((item, err));
    }

    fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    fn failed(&mut self, item: I, attempt: usize, err: Option<Err>) {
        self.failed.push((item, attempt, err));
    }

    fn failed_items(self) -> Vec<(I, Option<Err>)> {
        self.permanent_failure
    }
}