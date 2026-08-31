pub const DEFAULT_TASK_GRAPH_SCOPE_TASK_CAPACITY: usize = 1_024;

/// Names the runtime, module, or subsystem that owns a set of task-graph work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphScopeDescriptor {
    pub owner: String,
    pub task_capacity: usize,
}

impl TaskGraphScopeDescriptor {
    pub fn new(owner: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            task_capacity: DEFAULT_TASK_GRAPH_SCOPE_TASK_CAPACITY,
        }
    }

    pub fn with_task_capacity(mut self, task_capacity: usize) -> Self {
        self.task_capacity = task_capacity.max(1);
        self
    }
}

/// A consistent snapshot of one scope's admission and terminal work states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphScopeCensus {
    pub owner: String,
    pub task_capacity: usize,
    pub accepting: bool,
    pub submitted: u64,
    pub queued: usize,
    pub running: usize,
    pub completed: u64,
    pub failed: u64,
    pub cancelled: u64,
}

impl TaskGraphScopeCensus {
    pub const fn is_quiescent(&self) -> bool {
        self.queued == 0 && self.running == 0
    }
}
