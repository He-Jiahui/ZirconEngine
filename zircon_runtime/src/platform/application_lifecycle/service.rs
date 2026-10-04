use std::sync::{Mutex, MutexGuard};

use crate::core::framework::platform::{
    ApplicationActivationState, ApplicationLifecycleOperation, ApplicationLifecycleSnapshot,
    ApplicationLifecycleState, ApplicationLifecycleTerminalResult, ApplicationSurfaceAvailability,
};

use super::state::ApplicationLifecycleServiceState;
use super::ApplicationLifecycleServiceError;

/// Driver-owned state machine for application lifetime facts. Window focus and
/// per-window visibility remain outside this machine by design.
pub(crate) struct ApplicationLifecycleService {
    state: Mutex<ApplicationLifecycleServiceState>,
}

impl ApplicationLifecycleService {
    pub(crate) fn snapshot(&self) -> ApplicationLifecycleSnapshot {
        self.lock_state().snapshot
    }

    pub(crate) fn publish_activation(
        &self,
        activation: ApplicationActivationState,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let mut state = self.lock_state();
        let previous = state.snapshot;
        if previous.state() == ApplicationLifecycleState::Exiting {
            return Err(ApplicationLifecycleServiceError::InvalidState {
                operation: "publish application activation",
                state: ApplicationLifecycleState::Exiting,
            });
        }
        state.publish(
            previous.state(),
            activation,
            previous.surface_availability(),
            previous.active_operation(),
            previous.terminal(),
        )
    }

    pub(crate) fn publish_surface_availability(
        &self,
        surface_availability: ApplicationSurfaceAvailability,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let mut state = self.lock_state();
        let previous = state.snapshot;
        // Suspension revokes native surfaces from WillSuspend through its
        // terminal receipt; a delayed host callback cannot reopen a lease.
        if previous.state() == ApplicationLifecycleState::Exiting
            || (matches!(
                previous.state(),
                ApplicationLifecycleState::WillSuspend | ApplicationLifecycleState::Suspended
            ) && surface_availability == ApplicationSurfaceAvailability::Available)
        {
            return Err(ApplicationLifecycleServiceError::InvalidState {
                operation: "publish surface availability",
                state: previous.state(),
            });
        }
        state.publish(
            previous.state(),
            previous.activation(),
            surface_availability,
            previous.active_operation(),
            previous.terminal(),
        )
    }

    // 恢复先分配带目标态的操作票；终端回调必须带回同一 ID，迟到票据不能推进新一轮状态。
    pub(crate) fn request_resume(
        &self,
    ) -> Result<ApplicationLifecycleOperation, ApplicationLifecycleServiceError> {
        let mut state = self.lock_state();
        let previous = state.snapshot;
        if previous.state() == ApplicationLifecycleState::WillResume {
            return previous.active_operation().ok_or(
                ApplicationLifecycleServiceError::InvalidState {
                    operation: "reuse an in-flight resume operation",
                    state: ApplicationLifecycleState::WillResume,
                },
            );
        }
        match previous.state() {
            ApplicationLifecycleState::Cold | ApplicationLifecycleState::Suspended => {}
            current => {
                return Err(ApplicationLifecycleServiceError::InvalidState {
                    operation: "request resume",
                    state: current,
                });
            }
        }
        let operation = state.allocate_operation(ApplicationLifecycleState::Running)?;
        state.publish(
            ApplicationLifecycleState::WillResume,
            previous.activation(),
            previous.surface_availability(),
            Some(operation),
            None,
        )?;
        Ok(operation)
    }

    pub(crate) fn publish_running(
        &self,
        operation: ApplicationLifecycleOperation,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        self.complete(
            operation,
            ApplicationLifecycleState::WillResume,
            ApplicationLifecycleState::Running,
        )
    }

    pub(crate) fn request_suspend(
        &self,
    ) -> Result<ApplicationLifecycleOperation, ApplicationLifecycleServiceError> {
        let mut state = self.lock_state();
        let previous = state.snapshot;
        if previous.state() == ApplicationLifecycleState::WillSuspend {
            return previous.active_operation().ok_or(
                ApplicationLifecycleServiceError::InvalidState {
                    operation: "reuse an in-flight suspend operation",
                    state: ApplicationLifecycleState::WillSuspend,
                },
            );
        }
        if previous.state() != ApplicationLifecycleState::Running {
            return Err(ApplicationLifecycleServiceError::InvalidState {
                operation: "request suspend",
                state: previous.state(),
            });
        }
        let operation = state.allocate_operation(ApplicationLifecycleState::Suspended)?;
        state.publish(
            ApplicationLifecycleState::WillSuspend,
            previous.activation(),
            previous.surface_availability(),
            Some(operation),
            None,
        )?;
        Ok(operation)
    }

    /// Call only after submit work stopped, surface leases were retired, and
    /// the platform host has confirmed its suspend-side quiescence.
    pub(crate) fn publish_suspended(
        &self,
        operation: ApplicationLifecycleOperation,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        self.complete(
            operation,
            ApplicationLifecycleState::WillSuspend,
            ApplicationLifecycleState::Suspended,
        )
    }

    pub(crate) fn begin_exit(
        &self,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let mut state = self.lock_state();
        let previous = state.snapshot;
        if previous.state() == ApplicationLifecycleState::Exiting {
            return Ok(previous);
        }
        state.publish(
            ApplicationLifecycleState::Exiting,
            previous.activation(),
            ApplicationSurfaceAvailability::Unavailable,
            None,
            previous.terminal(),
        )
    }

    // 完成路径先校验期望状态和活动操作 ID，再发布终态快照并清除活动操作，保证回调顺序可验证。
    fn complete(
        &self,
        operation: ApplicationLifecycleOperation,
        expected_state: ApplicationLifecycleState,
        terminal_state: ApplicationLifecycleState,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let mut state = self.lock_state();
        let previous = state.snapshot;
        if previous.state() != expected_state {
            return Err(ApplicationLifecycleServiceError::InvalidState {
                operation: "publish lifecycle terminal receipt",
                state: previous.state(),
            });
        }
        let active =
            previous
                .active_operation()
                .ok_or(ApplicationLifecycleServiceError::InvalidState {
                    operation: "publish lifecycle terminal receipt without an operation",
                    state: expected_state,
                })?;
        if active.id() != operation.id() {
            return Err(ApplicationLifecycleServiceError::OperationMismatch {
                expected: active.id(),
                received: operation.id(),
            });
        }
        let surface_availability = if terminal_state == ApplicationLifecycleState::Suspended {
            ApplicationSurfaceAvailability::Unavailable
        } else {
            previous.surface_availability()
        };
        state.publish(
            terminal_state,
            previous.activation(),
            surface_availability,
            None,
            Some(ApplicationLifecycleTerminalResult::new(
                operation.id(),
                terminal_state,
            )),
        )
    }

    fn lock_state(&self) -> MutexGuard<'_, ApplicationLifecycleServiceState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for ApplicationLifecycleService {
    fn default() -> Self {
        Self {
            state: Mutex::new(ApplicationLifecycleServiceState::new()),
        }
    }
}

#[cfg(test)]
#[path = "tests/service.rs"]
mod tests;
