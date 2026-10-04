use super::super::TaskPoolKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphWorkerInventory {
    pub domains: Vec<TaskGraphWorkerDomainInventory>,
}

impl TaskGraphWorkerInventory {
    pub fn worker_set_count(&self) -> usize {
        self.domains.len()
    }

    pub fn worker_count(&self) -> usize {
        self.domains.iter().map(|domain| domain.worker_count).sum()
    }

    pub fn domain(&self, kind: TaskPoolKind) -> Option<&TaskGraphWorkerDomainInventory> {
        self.domains.iter().find(|domain| domain.kind == kind)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphWorkerDomainInventory {
    pub kind: TaskPoolKind,
    pub worker_count: usize,
    pub thread_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphWorkerShutdownCensus {
    pub kind: TaskPoolKind,
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
