pub const DEFAULT_MAX_SINGLE_QUEUE_PER_RESOURCE: usize = 64;
pub const DEFAULT_MAX_SET_QUEUE: usize = 64;

/// Independent entry ceilings for single-resource and atomic-set wait queues.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolQueueLimits {
    max_single_queue_per_resource: usize,
    max_set_queue: usize,
}

impl ToolQueueLimits {
    pub const fn new(max_single_queue_per_resource: usize, max_set_queue: usize) -> Self {
        Self {
            max_single_queue_per_resource,
            max_set_queue,
        }
    }

    pub const fn max_single_queue_per_resource(self) -> usize {
        self.max_single_queue_per_resource
    }

    pub const fn max_set_queue(self) -> usize {
        self.max_set_queue
    }
}

impl Default for ToolQueueLimits {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_SINGLE_QUEUE_PER_RESOURCE, DEFAULT_MAX_SET_QUEUE)
    }
}
