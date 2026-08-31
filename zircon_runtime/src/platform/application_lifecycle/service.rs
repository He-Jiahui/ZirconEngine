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
mod tests {
    use super::*;

    #[test]
    fn suspended_application_rejects_late_surface_availability() {
        let service = ApplicationLifecycleService::default();
        let resume = service
            .request_resume()
            .expect("a cold application must admit its first resume operation");
        service
            .publish_surface_availability(ApplicationSurfaceAvailability::Available)
            .expect("the resume path may publish an available surface");
        service
            .publish_running(resume)
            .expect("the matching resume operation must enter running");

        let suspend = service
            .request_suspend()
            .expect("a running application must admit suspension");
        let suspended = service
            .publish_suspended(suspend)
            .expect("the matching suspend operation must enter suspended");

        assert_eq!(
            service.publish_surface_availability(ApplicationSurfaceAvailability::Available),
            Err(ApplicationLifecycleServiceError::InvalidState {
                operation: "publish surface availability",
                state: ApplicationLifecycleState::Suspended,
            })
        );
        assert_eq!(service.snapshot(), suspended);
    }

    #[test]
    fn suspending_application_rejects_late_surface_availability_before_terminal_receipt() {
        let service = ApplicationLifecycleService::default();
        let resume = service
            .request_resume()
            .expect("a cold application must admit its first resume operation");
        service
            .publish_running(resume)
            .expect("the matching resume operation must enter running");
        service
            .request_suspend()
            .expect("a running application must admit suspension");
        let suspending = service.snapshot();

        assert_eq!(
            service.publish_surface_availability(ApplicationSurfaceAvailability::Available),
            Err(ApplicationLifecycleServiceError::InvalidState {
                operation: "publish surface availability",
                state: ApplicationLifecycleState::WillSuspend,
            })
        );
        assert_eq!(service.snapshot(), suspending);
    }

    #[test]
    fn suspended_application_retains_unavailable_surface_fact() {
        let service = ApplicationLifecycleService::default();
        let resume = service
            .request_resume()
            .expect("a cold application must admit its first resume operation");
        service
            .publish_running(resume)
            .expect("the matching resume operation must enter running");
        let suspend = service
            .request_suspend()
            .expect("a running application must admit suspension");
        service
            .publish_suspended(suspend)
            .expect("the matching suspend operation must enter suspended");

        let snapshot = service
            .publish_surface_availability(ApplicationSurfaceAvailability::Unavailable)
            .expect("unavailable surface publication remains idempotent while suspended");
        assert_eq!(snapshot.state(), ApplicationLifecycleState::Suspended);
        assert_eq!(
            snapshot.surface_availability(),
            ApplicationSurfaceAvailability::Unavailable
        );
    }
}
