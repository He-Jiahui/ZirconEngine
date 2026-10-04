use std::error::Error;
use std::fmt;
use std::time::{Duration, Instant};

use crate::core::manager::resolve_manager_service;
use crate::core::{CoreRuntime, TaskGraphScope, TaskGraphShutdownReport};

pub(super) const DYNAMIC_SESSION_LIBRARY_UNLOAD_TIMEOUT: Duration = Duration::from_secs(5);

/// A secondary close diagnostic; the original constructor error remains primary.
#[derive(Debug)]
pub(super) struct RuntimeCoreShutdownError {
    step: &'static str,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl RuntimeCoreShutdownError {
    fn pending(step: &'static str) -> Self {
        Self { step, source: None }
    }

    fn failed(step: &'static str, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            step,
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for RuntimeCoreShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime Core shutdown incomplete at {}",
            self.step
        )?;
        if let Some(source) = &self.source {
            write!(formatter, ": {source}")?;
        }
        Ok(())
    }
}

impl Error for RuntimeCoreShutdownError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &dyn Error)
    }
}

/// Shared canonical Core close. The caller stops mirrors first and the process log last.
pub(super) fn shutdown_runtime_core_until(
    runtime: &CoreRuntime,
    task_graph_scope: Option<&TaskGraphScope>,
    project_watchers_shutdown: &mut bool,
    deadline: Instant,
) -> Result<TaskGraphShutdownReport, RuntimeCoreShutdownError> {
    if !*project_watchers_shutdown {
        // Watch callbacks may still emit diagnostics, so they stop before the final lease.
        let core = runtime.handle();
        if let Ok(handle) = crate::asset::project_asset_manager_handle(&core) {
            if let Ok(manager) = resolve_manager_service(&core, handle) {
                if !manager.shutdown_project_watchers_until(deadline) {
                    return Err(RuntimeCoreShutdownError::pending("project watchers"));
                }
            }
        }
        *project_watchers_shutdown = true;
    }
    if let Some(scope) = task_graph_scope {
        scope.close_admission();
        let remaining = deadline.saturating_duration_since(Instant::now());
        if !scope.wait_until_quiescent(remaining) {
            return Err(RuntimeCoreShutdownError::pending("dynamic session scope"));
        }
    }
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(RuntimeCoreShutdownError::pending("Core close deadline"));
    }
    runtime
        .shutdown_until(deadline)
        .map_err(|source| RuntimeCoreShutdownError::failed("Core modules and task graph", source))
}

#[cfg(test)]
#[path = "shutdown/tests/cases.rs"]
mod tests;
