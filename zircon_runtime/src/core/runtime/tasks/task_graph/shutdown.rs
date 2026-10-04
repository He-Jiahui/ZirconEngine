use std::fmt;
use std::time::Duration;

use super::{TaskGraphScopeCensus, TaskGraphWorkerShutdownCensus};

/// Snapshot returned while a task graph closes scopes and its worker domains.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphShutdownReport {
    pub elapsed: Duration,
    pub scopes: Vec<TaskGraphScopeCensus>,
    /// Whether this graph ever started its lazy lifecycle timer.
    pub timer_started: bool,
    /// Whether the owned timer control thread has been joined. True when no
    /// timer was started; pool-domain joins are reported separately.
    pub timer_joined: bool,
    pub worker_shutdowns: Vec<TaskGraphWorkerShutdownCensus>,
}

impl TaskGraphShutdownReport {
    pub fn has_in_flight_work(&self) -> bool {
        !self.timer_joined
            || self.scopes.iter().any(|scope| !scope.is_quiescent())
            || self
                .worker_shutdowns
                .iter()
                .any(|workers| !workers.all_joined())
    }

    pub fn worker_shutdown(
        &self,
        kind: super::super::TaskPoolKind,
    ) -> Option<&TaskGraphWorkerShutdownCensus> {
        self.worker_shutdowns
            .iter()
            .find(|workers| workers.kind == kind)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskGraphShutdownError {
    pub report: TaskGraphShutdownReport,
}

impl fmt::Display for TaskGraphShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "engine task graph did not quiesce tasks and join its timer and worker domains before the shutdown deadline",
        )
    }
}

impl std::error::Error for TaskGraphShutdownError {}
