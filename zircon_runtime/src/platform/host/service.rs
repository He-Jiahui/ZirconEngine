use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::core::framework::platform::{
    PlatformHostBackend, PlatformHostEvidence, PlatformHostFailureReason, PlatformHostInstanceId,
    PlatformHostLifecycleState, PlatformHostQuiesceRequest, PlatformHostSnapshot,
    PlatformHostTerminalResult,
};

use super::state::PlatformHostServiceState;
use super::PlatformHostServiceError;

/// Driver-owned control plane for one process-host platform backend.
///
/// It stores only an Arc-safe request bridge. Native event-loop and window
/// objects remain owned by the app host thread, which publishes observed
/// transitions and terminal receipts back through this service.
pub(crate) struct PlatformHostService {
    state: Mutex<PlatformHostServiceState>,
}

impl PlatformHostService {
    pub(crate) fn snapshot(&self) -> PlatformHostSnapshot {
        self.lock_state().snapshot.clone()
    }

    pub(crate) fn install(
        &self,
        backend: Arc<dyn PlatformHostBackend>,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        let mut state = self.lock_state();
        match state.snapshot.lifecycle() {
            PlatformHostLifecycleState::Uninstalled
            | PlatformHostLifecycleState::Failed
            | PlatformHostLifecycleState::Stopped => {}
            lifecycle => {
                return Err(PlatformHostServiceError::AlreadyInstalled { state: lifecycle });
            }
        }

        let instance = state.allocate_instance()?;
        let generation = state.next_generation()?;
        let descriptor = backend.descriptor();
        let snapshot = PlatformHostSnapshot::new(
            PlatformHostLifecycleState::Starting,
            Some(instance),
            Some(descriptor),
            None,
            None,
            None,
            generation,
        );
        state.backend = Some(backend);
        state.active_quiesce = None;
        state.snapshot = snapshot;
        Ok(snapshot)
    }

    pub(crate) fn publish_ready(
        &self,
        instance: PlatformHostInstanceId,
        evidence: PlatformHostEvidence,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        let mut state = self.lock_state();
        state.validate_instance(instance)?;
        match state.snapshot.lifecycle() {
            PlatformHostLifecycleState::Starting | PlatformHostLifecycleState::Degraded => state
                .publish(
                    PlatformHostLifecycleState::Ready,
                    Some(evidence),
                    None,
                    None,
                ),
            lifecycle => Err(PlatformHostServiceError::InvalidLifecycleState {
                operation: "publish readiness",
                state: lifecycle,
            }),
        }
    }

    pub(crate) fn publish_degraded(
        &self,
        instance: PlatformHostInstanceId,
        evidence: PlatformHostEvidence,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        let mut state = self.lock_state();
        state.validate_instance(instance)?;
        match state.snapshot.lifecycle() {
            PlatformHostLifecycleState::Starting | PlatformHostLifecycleState::Ready => state
                .publish(
                    PlatformHostLifecycleState::Degraded,
                    Some(evidence),
                    None,
                    None,
                ),
            lifecycle => Err(PlatformHostServiceError::InvalidLifecycleState {
                operation: "publish degradation",
                state: lifecycle,
            }),
        }
    }

    pub(crate) fn request_quiesce(
        &self,
        deadline: Instant,
    ) -> Result<PlatformHostQuiesceRequest, PlatformHostServiceError> {
        let (backend, request) = {
            let mut state = self.lock_state();
            if state.snapshot.lifecycle() == PlatformHostLifecycleState::Quiescing {
                return state.active_quiesce.ok_or(
                    PlatformHostServiceError::InvalidLifecycleState {
                        operation: "reuse an in-flight quiesce request",
                        state: PlatformHostLifecycleState::Quiescing,
                    },
                );
            }
            match state.snapshot.lifecycle() {
                PlatformHostLifecycleState::Starting
                | PlatformHostLifecycleState::Ready
                | PlatformHostLifecycleState::Degraded => {}
                lifecycle => {
                    return Err(PlatformHostServiceError::InvalidLifecycleState {
                        operation: "request quiesce",
                        state: lifecycle,
                    });
                }
            }
            let instance = state
                .snapshot
                .instance()
                .ok_or(PlatformHostServiceError::NoHostInstalled)?;
            let operation = state.allocate_operation()?;
            let request = PlatformHostQuiesceRequest::new(instance, operation, deadline);
            let backend = Arc::clone(
                state
                    .backend
                    .as_ref()
                    .ok_or(PlatformHostServiceError::BackendBridgeMissing)?,
            );
            let evidence = state.snapshot.evidence().cloned();
            state.publish(
                PlatformHostLifecycleState::Quiescing,
                evidence,
                Some(operation),
                None,
            )?;
            state.active_quiesce = Some(request);
            (backend, request)
        };

        if let Err(reason) = backend.request_quiesce(request) {
            let mut state = self.lock_state();
            if state.active_quiesce == Some(request) {
                state.publish(
                    PlatformHostLifecycleState::Failed,
                    None,
                    None,
                    Some(PlatformHostTerminalResult::Failed {
                        instance: request.instance(),
                        operation: Some(request.operation()),
                        reason: PlatformHostFailureReason::BackendRejectedRequest,
                    }),
                )?;
                state.active_quiesce = None;
                state.backend = None;
            }
            return Err(PlatformHostServiceError::BackendRejected { reason });
        }
        Ok(request)
    }

    pub(crate) fn publish_quiesced(
        &self,
        request: PlatformHostQuiesceRequest,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        let mut state = self.lock_state();
        state.validate_instance(request.instance())?;
        if state.snapshot.lifecycle() != PlatformHostLifecycleState::Quiescing {
            return Err(PlatformHostServiceError::InvalidLifecycleState {
                operation: "publish quiesced receipt",
                state: state.snapshot.lifecycle(),
            });
        }
        let active =
            state
                .active_quiesce
                .ok_or(PlatformHostServiceError::InvalidLifecycleState {
                    operation: "publish quiesced receipt without an in-flight operation",
                    state: PlatformHostLifecycleState::Quiescing,
                })?;
        if active.operation() != request.operation() {
            return Err(PlatformHostServiceError::OperationMismatch {
                expected: active.operation(),
                received: request.operation(),
            });
        }
        let snapshot = state.publish(
            PlatformHostLifecycleState::Quiesced,
            None,
            None,
            Some(PlatformHostTerminalResult::Quiesced {
                instance: request.instance(),
                operation: request.operation(),
            }),
        )?;
        state.active_quiesce = None;
        Ok(snapshot)
    }

    pub(crate) fn publish_failed(
        &self,
        instance: PlatformHostInstanceId,
        reason: PlatformHostFailureReason,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        let mut state = self.lock_state();
        state.validate_instance(instance)?;
        if state.snapshot.lifecycle() == PlatformHostLifecycleState::Failed {
            return Ok(state.snapshot.clone());
        }
        if state.snapshot.lifecycle() == PlatformHostLifecycleState::Stopped {
            return Err(PlatformHostServiceError::InvalidLifecycleState {
                operation: "publish failure",
                state: PlatformHostLifecycleState::Stopped,
            });
        }
        let operation = state.active_quiesce.map(|request| request.operation());
        let snapshot = state.publish(
            PlatformHostLifecycleState::Failed,
            None,
            None,
            Some(PlatformHostTerminalResult::Failed {
                instance,
                operation,
                reason,
            }),
        )?;
        state.active_quiesce = None;
        state.backend = None;
        Ok(snapshot)
    }

    pub(crate) fn publish_stopped(
        &self,
        instance: PlatformHostInstanceId,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        let mut state = self.lock_state();
        state.validate_instance(instance)?;
        if state.snapshot.lifecycle() != PlatformHostLifecycleState::Quiesced {
            return Err(PlatformHostServiceError::InvalidLifecycleState {
                operation: "publish stopped receipt",
                state: state.snapshot.lifecycle(),
            });
        }
        let snapshot = state.publish(
            PlatformHostLifecycleState::Stopped,
            None,
            None,
            Some(PlatformHostTerminalResult::Stopped { instance }),
        )?;
        state.backend = None;
        Ok(snapshot)
    }

    fn lock_state(&self) -> MutexGuard<'_, PlatformHostServiceState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for PlatformHostService {
    fn default() -> Self {
        Self {
            state: Mutex::new(PlatformHostServiceState::new()),
        }
    }
}
