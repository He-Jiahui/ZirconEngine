use std::time::Instant;

use super::{PlatformHostInstanceId, PlatformHostOperationId};

/// 发给宿主线程的停机请求；回执须保留实例和操作身份以拒绝旧宿主的迟到通知。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformHostQuiesceRequest {
    instance: PlatformHostInstanceId,
    operation: PlatformHostOperationId,
    deadline: Instant,
}

impl PlatformHostQuiesceRequest {
    pub(crate) const fn new(
        instance: PlatformHostInstanceId,
        operation: PlatformHostOperationId,
        deadline: Instant,
    ) -> Self {
        Self {
            instance,
            operation,
            deadline,
        }
    }

    pub const fn instance(self) -> PlatformHostInstanceId {
        self.instance
    }

    pub const fn operation(self) -> PlatformHostOperationId {
        self.operation
    }

    pub const fn deadline(self) -> Instant {
        self.deadline
    }
}
