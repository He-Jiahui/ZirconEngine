//! Bounded admission and retention limits for asset worker completions.

use std::time::Duration;

pub(super) const DEFAULT_ASSET_WORKER_QUEUE_DEPTH: usize = 2;
const DEFAULT_WAITER_CAPACITY: usize = 1_024;
const DEFAULT_COMPLETION_ENTRY_CAPACITY: usize = 64;
const DEFAULT_COMPLETION_BYTE_CAPACITY: usize = 64 * 1024 * 1024;
const DEFAULT_REQUEST_MAX_AGE: Duration = Duration::from_secs(30);
const DEFAULT_COMPLETION_MAX_AGE: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetWorkerPoolOptions {
    /// `None` selects the same bounded admission limit as the default options.
    pub queue_depth: Option<usize>,
    pub waiter_capacity: usize,
    pub completion_entry_capacity: usize,
    pub completion_byte_capacity: usize,
    pub request_max_age: Duration,
    pub completion_max_age: Duration,
}

impl Default for AssetWorkerPoolOptions {
    fn default() -> Self {
        Self {
            queue_depth: Some(DEFAULT_ASSET_WORKER_QUEUE_DEPTH),
            waiter_capacity: DEFAULT_WAITER_CAPACITY,
            completion_entry_capacity: DEFAULT_COMPLETION_ENTRY_CAPACITY,
            completion_byte_capacity: DEFAULT_COMPLETION_BYTE_CAPACITY,
            request_max_age: DEFAULT_REQUEST_MAX_AGE,
            completion_max_age: DEFAULT_COMPLETION_MAX_AGE,
        }
    }
}

impl AssetWorkerPoolOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_queue_depth(mut self, queue_depth: usize) -> Self {
        self.queue_depth = Some(queue_depth);
        self
    }

    pub fn with_completion_entry_capacity(mut self, capacity: usize) -> Self {
        self.completion_entry_capacity = capacity;
        self
    }

    pub fn with_waiter_capacity(mut self, capacity: usize) -> Self {
        self.waiter_capacity = capacity;
        self
    }

    pub fn with_completion_byte_capacity(mut self, capacity: usize) -> Self {
        self.completion_byte_capacity = capacity;
        self
    }

    pub fn with_request_max_age(mut self, age: Duration) -> Self {
        self.request_max_age = age;
        self
    }

    pub fn with_completion_max_age(mut self, age: Duration) -> Self {
        self.completion_max_age = age;
        self
    }
}
