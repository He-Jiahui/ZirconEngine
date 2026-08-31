#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphWorkerInventory {
    pub worker_set_count: usize,
    pub worker_count: usize,
    pub thread_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphWorkerShutdownCensus {
    pub active_submission_count: usize,
    pub expected_worker_count: usize,
    pub exited_worker_count: usize,
    pub joined_worker_count: usize,
    pub termination_signalled: bool,
}

impl TaskGraphWorkerShutdownCensus {
    pub const fn all_joined(&self) -> bool {
        self.active_submission_count == 0
            && self.termination_signalled
            && self.exited_worker_count == self.expected_worker_count
            && self.joined_worker_count == self.expected_worker_count
    }
}
