use super::{
    PlatformHostDescriptor, PlatformHostEvidence, PlatformHostGeneration, PlatformHostHealth,
    PlatformHostInstanceId, PlatformHostLifecycleState, PlatformHostOperationId,
    PlatformHostTerminalResult,
};

/// Immutable control-plane fact published by the platform driver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformHostSnapshot {
    lifecycle: PlatformHostLifecycleState,
    instance: Option<PlatformHostInstanceId>,
    descriptor: Option<PlatformHostDescriptor>,
    evidence: Option<PlatformHostEvidence>,
    active_operation: Option<PlatformHostOperationId>,
    terminal: Option<PlatformHostTerminalResult>,
    generation: PlatformHostGeneration,
}

impl PlatformHostSnapshot {
    pub(crate) const fn new(
        lifecycle: PlatformHostLifecycleState,
        instance: Option<PlatformHostInstanceId>,
        descriptor: Option<PlatformHostDescriptor>,
        evidence: Option<PlatformHostEvidence>,
        active_operation: Option<PlatformHostOperationId>,
        terminal: Option<PlatformHostTerminalResult>,
        generation: PlatformHostGeneration,
    ) -> Self {
        Self {
            lifecycle,
            instance,
            descriptor,
            evidence,
            active_operation,
            terminal,
            generation,
        }
    }

    pub const fn lifecycle(&self) -> PlatformHostLifecycleState {
        self.lifecycle
    }

    pub const fn instance(&self) -> Option<PlatformHostInstanceId> {
        self.instance
    }

    pub const fn descriptor(&self) -> Option<PlatformHostDescriptor> {
        self.descriptor
    }

    pub const fn evidence(&self) -> Option<&PlatformHostEvidence> {
        self.evidence.as_ref()
    }

    pub const fn active_operation(&self) -> Option<PlatformHostOperationId> {
        self.active_operation
    }

    pub const fn terminal(&self) -> Option<PlatformHostTerminalResult> {
        self.terminal
    }

    pub const fn generation(&self) -> PlatformHostGeneration {
        self.generation
    }

    pub const fn is_ready(&self) -> bool {
        matches!(self.lifecycle, PlatformHostLifecycleState::Ready)
    }

    pub const fn health(&self) -> PlatformHostHealth {
        match self.lifecycle {
            PlatformHostLifecycleState::Ready | PlatformHostLifecycleState::Quiesced => {
                PlatformHostHealth::Healthy
            }
            PlatformHostLifecycleState::Degraded => PlatformHostHealth::Degraded,
            PlatformHostLifecycleState::Failed => PlatformHostHealth::Failed,
            PlatformHostLifecycleState::Uninstalled
            | PlatformHostLifecycleState::Starting
            | PlatformHostLifecycleState::Quiescing
            | PlatformHostLifecycleState::Stopped => PlatformHostHealth::Unknown,
        }
    }
}
