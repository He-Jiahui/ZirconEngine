use super::{PlatformHostFailureReason, PlatformHostInstanceId, PlatformHostOperationId};

/// 宿主停机或失败的最终事实，保留实例身份以区分重启前后的回执。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostTerminalResult {
    Quiesced {
        instance: PlatformHostInstanceId,
        operation: PlatformHostOperationId,
    },
    Failed {
        instance: PlatformHostInstanceId,
        operation: Option<PlatformHostOperationId>,
        reason: PlatformHostFailureReason,
    },
    Stopped {
        instance: PlatformHostInstanceId,
    },
}

impl PlatformHostTerminalResult {
    pub const fn is_quiesced(self) -> bool {
        matches!(self, Self::Quiesced { .. })
    }

    pub const fn is_failed(self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}
