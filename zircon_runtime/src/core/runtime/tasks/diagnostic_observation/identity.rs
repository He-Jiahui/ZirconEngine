use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaskDiagnosticIdentity {
    scheduler_id: u64,
    task_sequence: u64,
}

impl TaskDiagnosticIdentity {
    pub(in crate::core::runtime::tasks) const fn new(
        scheduler_id: u64,
        task_sequence: u64,
    ) -> Self {
        Self {
            scheduler_id,
            task_sequence,
        }
    }

    pub const fn scheduler_id(self) -> u64 {
        self.scheduler_id
    }

    pub const fn task_sequence(self) -> u64 {
        self.task_sequence
    }
}

impl Display for TaskDiagnosticIdentity {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}:{}", self.scheduler_id, self.task_sequence)
    }
}
