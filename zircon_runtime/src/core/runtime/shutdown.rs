use std::error::Error;
use std::fmt;
use std::time::Instant;

use super::{
    error::CoreError, CoreRuntime, ModuleShutdownReport, TaskGraphShutdownError,
    TaskGraphShutdownReport,
};

/// The complete module and owned graph results of one incomplete Core shutdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreShutdownError {
    modules: ModuleShutdownReport,
    module_error: Option<CoreError>,
    graph: Result<TaskGraphShutdownReport, TaskGraphShutdownError>,
}

impl CoreShutdownError {
    pub fn module_report(&self) -> &ModuleShutdownReport {
        &self.modules
    }

    /// Returns the first module failure, including an exhausted cleanup deadline.
    pub fn module_error(&self) -> Option<&CoreError> {
        self.module_error.as_ref()
    }

    pub fn task_graph_error(&self) -> Option<&TaskGraphShutdownError> {
        self.graph.as_ref().err()
    }

    /// Returns the actual graph receipt, including graph success after module failure.
    pub fn graph_report(&self) -> Option<&TaskGraphShutdownReport> {
        Some(match &self.graph {
            Ok(report) => report,
            Err(error) => &error.report,
        })
    }
}

impl fmt::Display for CoreShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.module_error, &self.graph) {
            (Some(module), Err(graph)) => write!(
                formatter,
                "registered module shutdown failed: {module}; owned task graph shutdown failed: {graph}"
            ),
            (Some(error), Ok(_)) => {
                write!(formatter, "registered module shutdown failed: {error}")
            }
            (None, Err(error)) => {
                write!(formatter, "owned task graph shutdown failed: {error}")
            }
            (None, Ok(_)) => formatter.write_str("Core shutdown incomplete"),
        }
    }
}

impl Error for CoreShutdownError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.module_error()
            .map(|error| error as &(dyn Error + 'static))
            .or_else(|| {
                self.task_graph_error()
                    .map(|error| error as &(dyn Error + 'static))
            })
    }
}

impl CoreRuntime {
    /// Cleans active modules in reverse order, then joins this Core's owned graph.
    ///
    /// Every stage receives the same cooperative deadline, including graph shutdown after a
    /// module failure or timeout. Non-cooperative callbacks cannot be interrupted.
    /// A module error leaves its original lifecycle available for retry;
    /// completed module cleanup is removed from the active ledger and is not
    /// replayed by a later retry.
    /// The receipt excludes external handles, process-global timers and dedicated workers.
    pub fn shutdown_until(
        &self,
        deadline: Instant,
    ) -> Result<TaskGraphShutdownReport, CoreShutdownError> {
        let modules = self.shutdown_registered_modules_until(deadline);
        let module_error = modules.clone().into_result().err();
        let graph = self.shutdown_task_graph_until(deadline);
        match (module_error, graph) {
            (None, Ok(report)) => Ok(report),
            (module_error, graph) => Err(CoreShutdownError {
                modules,
                module_error,
                graph,
            }),
        }
    }
}
