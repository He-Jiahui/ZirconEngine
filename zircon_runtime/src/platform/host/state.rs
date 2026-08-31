use std::sync::Arc;

use crate::core::framework::platform::{
    PlatformHostBackend, PlatformHostEvidence, PlatformHostGeneration, PlatformHostInstanceId,
    PlatformHostLifecycleState, PlatformHostOperationId, PlatformHostQuiesceRequest,
    PlatformHostSnapshot, PlatformHostTerminalResult,
};

use super::PlatformHostServiceError;

pub(super) struct PlatformHostServiceState {
    pub(super) next_instance: u64,
    pub(super) next_operation: u64,
    pub(super) backend: Option<Arc<dyn PlatformHostBackend>>,
    pub(super) active_quiesce: Option<PlatformHostQuiesceRequest>,
    pub(super) snapshot: PlatformHostSnapshot,
}

impl PlatformHostServiceState {
    pub(super) fn new() -> Self {
        Self {
            next_instance: 1,
            next_operation: 1,
            backend: None,
            active_quiesce: None,
            snapshot: PlatformHostSnapshot::new(
                PlatformHostLifecycleState::Uninstalled,
                None,
                None,
                None,
                None,
                None,
                PlatformHostGeneration::initial(),
            ),
        }
    }

    pub(super) fn allocate_instance(
        &mut self,
    ) -> Result<PlatformHostInstanceId, PlatformHostServiceError> {
        let raw = self.next_instance;
        let next = raw
            .checked_add(1)
            .ok_or(PlatformHostServiceError::InstanceIdExhausted)?;
        let instance = PlatformHostInstanceId::new(raw)
            .ok_or(PlatformHostServiceError::InstanceIdExhausted)?;
        self.next_instance = next;
        Ok(instance)
    }

    pub(super) fn allocate_operation(
        &mut self,
    ) -> Result<PlatformHostOperationId, PlatformHostServiceError> {
        let raw = self.next_operation;
        let next = raw
            .checked_add(1)
            .ok_or(PlatformHostServiceError::OperationIdExhausted)?;
        let operation = PlatformHostOperationId::new(raw)
            .ok_or(PlatformHostServiceError::OperationIdExhausted)?;
        self.next_operation = next;
        Ok(operation)
    }

    pub(super) fn next_generation(
        &self,
    ) -> Result<PlatformHostGeneration, PlatformHostServiceError> {
        self.snapshot
            .generation()
            .next()
            .ok_or(PlatformHostServiceError::SnapshotGenerationExhausted)
    }

    pub(super) fn publish(
        &mut self,
        lifecycle: PlatformHostLifecycleState,
        evidence: Option<PlatformHostEvidence>,
        active_operation: Option<PlatformHostOperationId>,
        terminal: Option<PlatformHostTerminalResult>,
    ) -> Result<PlatformHostSnapshot, PlatformHostServiceError> {
        let generation = self.next_generation()?;
        let snapshot = PlatformHostSnapshot::new(
            lifecycle,
            self.snapshot.instance(),
            self.snapshot.descriptor(),
            evidence,
            active_operation,
            terminal,
            generation,
        );
        self.snapshot = snapshot.clone();
        Ok(snapshot)
    }

    pub(super) fn validate_instance(
        &self,
        received: PlatformHostInstanceId,
    ) -> Result<(), PlatformHostServiceError> {
        let expected = self
            .snapshot
            .instance()
            .ok_or(PlatformHostServiceError::NoHostInstalled)?;
        if expected == received {
            Ok(())
        } else {
            Err(PlatformHostServiceError::StaleInstance { expected, received })
        }
    }
}
