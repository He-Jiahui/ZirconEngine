use std::fmt;
use std::time::Duration;

use super::{TaskGraphScopeCensus, TaskGraphWorkerShutdownCensus};

/// Snapshot returned while a task graph closes scopes and its shared worker set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphShutdownReport {
    pub elapsed: Duration,
    pub scopes: Vec<TaskGraphScopeCensus>,
    pub worker_shutdown: TaskGraphWorkerShutdownCensus,
}

impl TaskGraphShutdownReport {
    pub fn has_in_flight_work(&self) -> bool {
        self.scopes.iter().any(|scope| !scope.is_quiescent()) || !self.worker_shutdown.all_joined()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphShutdownError {
    pub report: TaskGraphShutdownReport,
}

impl fmt::Display for TaskGraphShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "engine task graph did not quiesce tasks and join its worker set before the shutdown deadline",
        )
    }
}

impl std::error::Error for TaskGraphShutdownError {}
