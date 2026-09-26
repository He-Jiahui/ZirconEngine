use super::{ApplicationLifecycleOperationId, ApplicationLifecycleState};

/// 一次恢复或挂起事务的回执令牌。调用端须将原令牌带回终态发布入口。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationLifecycleOperation {
    id: ApplicationLifecycleOperationId,
    target: ApplicationLifecycleState,
}

impl ApplicationLifecycleOperation {
    pub(crate) const fn new(
        id: ApplicationLifecycleOperationId,
        target: ApplicationLifecycleState,
    ) -> Self {
        Self { id, target }
    }

    pub const fn id(self) -> ApplicationLifecycleOperationId {
        self.id
    }

    pub const fn target(self) -> ApplicationLifecycleState {
        self.target
    }
}
