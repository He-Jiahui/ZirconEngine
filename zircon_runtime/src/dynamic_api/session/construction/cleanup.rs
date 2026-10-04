use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use crate::core::{CoreRuntime, TaskGraphScope, TaskGraphShutdownReport};
use crate::plugin::{CompiledProjectPluginPlan, RuntimePluginCatalogSnapshot};

use super::super::shutdown::{
    shutdown_runtime_core_until, RuntimeCoreShutdownError, DYNAMIC_SESSION_LIBRARY_UNLOAD_TIMEOUT,
};
use super::super::RuntimeDynamicSessionError;

/// Retains the same Core and plugin generation until the existing startup owner closes them.
struct ConstructionCoreOwner {
    runtime: CoreRuntime,
    scope: Option<TaskGraphScope>,
    _catalog: Arc<RuntimePluginCatalogSnapshot>,
    _plan: Arc<CompiledProjectPluginPlan>,
    project_watchers_shutdown: bool,
}

pub(in crate::dynamic_api::session) struct RuntimeConstructionFailure {
    primary: RuntimeDynamicSessionError,
    pending_core: Option<ConstructionCoreOwner>,
    shutdown_report: Option<TaskGraphShutdownReport>,
    shutdown_error: Option<RuntimeCoreShutdownError>,
    #[cfg(test)]
    pub(in crate::dynamic_api::session) observed_core: Option<crate::core::CoreHandle>,
    #[cfg(test)]
    pub(super) running_modules_before_close: usize,
}

impl fmt::Debug for RuntimeConstructionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeConstructionFailure")
            .field("primary", &self.primary)
            .field("pending_core", &self.has_pending_core())
            .field("shutdown_report", &self.shutdown_report)
            .field("shutdown_error", &self.shutdown_error)
            .finish()
    }
}

impl fmt::Display for RuntimeConstructionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.primary, formatter)
    }
}

impl std::error::Error for RuntimeConstructionFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.primary)
    }
}

impl From<RuntimeDynamicSessionError> for RuntimeConstructionFailure {
    fn from(primary: RuntimeDynamicSessionError) -> Self {
        Self {
            primary,
            pending_core: None,
            shutdown_report: None,
            shutdown_error: None,
            #[cfg(test)]
            observed_core: None,
            #[cfg(test)]
            running_modules_before_close: 0,
        }
    }
}

impl RuntimeConstructionFailure {
    pub(in crate::dynamic_api::session) fn primary(&self) -> &RuntimeDynamicSessionError {
        &self.primary
    }

    pub(in crate::dynamic_api::session) fn has_pending_core(&self) -> bool {
        self.pending_core.is_some()
    }

    pub(in crate::dynamic_api::session) fn shutdown_error(
        &self,
    ) -> Option<&RuntimeCoreShutdownError> {
        self.shutdown_error.as_ref()
    }

    #[cfg(test)]
    pub(in crate::dynamic_api::session) fn shutdown_report(
        &self,
    ) -> Option<&TaskGraphShutdownReport> {
        self.shutdown_report.as_ref()
    }

    pub(in crate::dynamic_api::session) fn shutdown_until(&mut self, deadline: Instant) -> bool {
        let Some(owner) = self.pending_core.as_mut() else {
            return true;
        };
        match shutdown_runtime_core_until(
            &owner.runtime,
            owner.scope.as_ref(),
            &mut owner.project_watchers_shutdown,
            deadline,
        ) {
            Ok(report) => {
                self.shutdown_report = Some(report);
                self.shutdown_error = None;
                self.pending_core = None;
                true
            }
            Err(error) => {
                self.shutdown_error = Some(error);
                false
            }
        }
    }
}

pub(in crate::dynamic_api::session) struct RuntimeConstructionCleanup {
    owner: Option<ConstructionCoreOwner>,
}

impl RuntimeConstructionCleanup {
    pub(in crate::dynamic_api::session) fn new(
        runtime: &CoreRuntime,
        catalog: &Arc<RuntimePluginCatalogSnapshot>,
        plan: &Arc<CompiledProjectPluginPlan>,
    ) -> Self {
        Self {
            owner: Some(ConstructionCoreOwner {
                // CoreRuntime::clone shares the original CoreHandle authority and workers.
                runtime: runtime.clone(),
                scope: None,
                _catalog: Arc::clone(catalog),
                _plan: Arc::clone(plan),
                project_watchers_shutdown: false,
            }),
        }
    }

    pub(in crate::dynamic_api::session) fn attach_scope(&mut self, scope: &TaskGraphScope) {
        self.owner.as_mut().expect("construction Core owner").scope = Some(scope.clone());
    }

    pub(in crate::dynamic_api::session) fn check<T>(
        &mut self,
        result: Result<T, RuntimeDynamicSessionError>,
    ) -> Result<T, RuntimeConstructionFailure> {
        let deadline = Instant::now()
            .checked_add(DYNAMIC_SESSION_LIBRARY_UNLOAD_TIMEOUT)
            .unwrap_or_else(Instant::now);
        self.check_until(result, deadline)
    }

    pub(in crate::dynamic_api::session) fn check_until<T>(
        &mut self,
        result: Result<T, RuntimeDynamicSessionError>,
        deadline: Instant,
    ) -> Result<T, RuntimeConstructionFailure> {
        result.map_err(|primary| {
            let owner = self.owner.take().expect("construction Core owner");
            #[cfg(test)]
            let observed_core = owner.runtime.handle();
            #[cfg(test)]
            let running_modules_before_close = observed_core
                .inner
                .modules
                .lock()
                .expect("test module registry")
                .values()
                .filter(|entry| entry.lifecycle == crate::core::LifecycleState::Running)
                .count();
            let mut failure = RuntimeConstructionFailure {
                primary,
                pending_core: Some(owner),
                shutdown_report: None,
                shutdown_error: None,
                #[cfg(test)]
                observed_core: Some(observed_core),
                #[cfg(test)]
                running_modules_before_close,
            };
            if !failure.shutdown_until(deadline) {
                eprintln!(
                    "runtime construction failed: {}; secondary shutdown: {}",
                    failure.primary(),
                    failure
                        .shutdown_error()
                        .expect("incomplete close diagnostic")
                );
            }
            failure
        })
    }
}
