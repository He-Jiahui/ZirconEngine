use super::{ApplicationLifecycleOperationId, ApplicationLifecycleState};

/// 最近一次生命周期事务的终态回执，用操作 ID 将完成事件关联到请求。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationLifecycleTerminalResult {
    operation: ApplicationLifecycleOperationId,
    state: ApplicationLifecycleState,
}

impl ApplicationLifecycleTerminalResult {
    pub(crate) const fn new(
        operation: ApplicationLifecycleOperationId,
        state: ApplicationLifecycleState,
    ) -> Self {
        Self { operation, state }
    }

    pub const fn operation(self) -> ApplicationLifecycleOperationId {
        self.operation
    }

    pub const fn state(self) -> ApplicationLifecycleState {
        self.state
    }
}
