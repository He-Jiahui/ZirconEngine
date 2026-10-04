use super::{
    PlatformHostDescriptor, PlatformHostEvidence, PlatformHostGeneration, PlatformHostHealth,
    PlatformHostInstanceId, PlatformHostLifecycleState, PlatformHostOperationId,
    PlatformHostTerminalResult,
};

/// Immutable control-plane fact published by the platform driver.
/// 平台驱动发布的不可变控制面事实；管理端以实例、版本和观测证据判定能力。
/// 描述符只声明宿主类型，Ready 加上所需观测证据才可用于运行时准入。
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

    /// 诊断用健康投影；Quiesced 可为 Healthy，但此时不再提供运行时能力。
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
