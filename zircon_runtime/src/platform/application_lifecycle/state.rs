use crate::core::framework::platform::{
    ApplicationActivationState, ApplicationLifecycleGeneration, ApplicationLifecycleOperation,
    ApplicationLifecycleOperationId, ApplicationLifecycleSnapshot, ApplicationLifecycleState,
    ApplicationLifecycleTerminalResult, ApplicationSurfaceAvailability,
};

use super::ApplicationLifecycleServiceError;

pub(super) struct ApplicationLifecycleServiceState {
    pub(super) next_operation: u64,
    pub(super) snapshot: ApplicationLifecycleSnapshot,
}

impl ApplicationLifecycleServiceState {
    pub(super) const fn new() -> Self {
        Self {
            next_operation: 1,
            snapshot: ApplicationLifecycleSnapshot::new(
                ApplicationLifecycleState::Cold,
                ApplicationActivationState::Unknown,
                ApplicationSurfaceAvailability::Unavailable,
                None,
                None,
                ApplicationLifecycleGeneration::initial(),
            ),
        }
    }

    pub(super) fn allocate_operation(
        &mut self,
        target: ApplicationLifecycleState,
    ) -> Result<ApplicationLifecycleOperation, ApplicationLifecycleServiceError> {
        let raw = self.next_operation;
        let next = raw
            .checked_add(1)
            .ok_or(ApplicationLifecycleServiceError::OperationIdExhausted)?;
        let id = ApplicationLifecycleOperationId::new(raw)
            .ok_or(ApplicationLifecycleServiceError::OperationIdExhausted)?;
        self.next_operation = next;
        Ok(ApplicationLifecycleOperation::new(id, target))
    }

    pub(super) fn publish(
        &mut self,
        state: ApplicationLifecycleState,
        activation: ApplicationActivationState,
        surface_availability: ApplicationSurfaceAvailability,
        active_operation: Option<ApplicationLifecycleOperation>,
        terminal: Option<ApplicationLifecycleTerminalResult>,
    ) -> Result<ApplicationLifecycleSnapshot, ApplicationLifecycleServiceError> {
        let generation = self
            .snapshot
            .generation()
            .next()
            .ok_or(ApplicationLifecycleServiceError::GenerationExhausted)?;
        let snapshot = ApplicationLifecycleSnapshot::new(
            state,
            activation,
            surface_availability,
            active_operation,
            terminal,
            generation,
        );
        self.snapshot = snapshot;
        Ok(snapshot)
    }
}
