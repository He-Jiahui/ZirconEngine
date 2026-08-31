use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskGraphAdmissionError {
    RuntimeClosing,
    RuntimeStopped,
    RuntimeUnavailable,
    RuntimeScopeIdExhausted,
    ScopeClosed { owner: String },
    ScopeCapacityReached { owner: String, capacity: usize },
    TaskIdAlreadyActive { owner: String, id: u64 },
    SchedulerOwnerMismatch { owner: String },
}

impl fmt::Display for TaskGraphAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeClosing => formatter.write_str("engine task graph is closing admission"),
            Self::RuntimeStopped => formatter.write_str("engine task graph has stopped"),
            Self::RuntimeUnavailable => {
                formatter.write_str("engine task graph owner no longer exists")
            }
            Self::RuntimeScopeIdExhausted => {
                formatter.write_str("engine task graph scope identifier space is exhausted")
            }
            Self::ScopeClosed { owner } => {
                write!(formatter, "task graph scope `{owner}` is closing admission")
            }
            Self::ScopeCapacityReached { owner, capacity } => write!(
                formatter,
                "task graph scope `{owner}` reached its task capacity of {capacity}"
            ),
            Self::TaskIdAlreadyActive { owner, id } => write!(
                formatter,
                "task graph scope `{owner}` already owns task id {id}"
            ),
            Self::SchedulerOwnerMismatch { owner } => write!(
                formatter,
                "task graph scope `{owner}` received a scheduler from another worker owner"
            ),
        }
    }
}

impl std::error::Error for TaskGraphAdmissionError {}
