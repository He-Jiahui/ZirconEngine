//! Winit 回调失败进入产品关停账本的记录和停止信号。
//! 账本存完整错误；原子标志只用于阻止后续回调继续驱动动态会话。

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::entry) struct RuntimeEntryAppFailure {
    component: &'static str,
    requested: String,
    cause: String,
    recovery: &'static str,
}

impl RuntimeEntryAppFailure {
    pub(super) fn new(
        component: &'static str,
        requested: impl Display,
        cause: impl Display,
        recovery: &'static str,
    ) -> Self {
        Self {
            component,
            requested: requested.to_string(),
            cause: cause.to_string(),
            recovery,
        }
    }
}

impl Display for RuntimeEntryAppFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime startup diagnostic: component={} requested={} cause={} recovery={}",
            self.component, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for RuntimeEntryAppFailure {}

#[derive(Clone, Debug)]
/// 入口、回调和退出报告共享的失败句柄；克隆后仍指向同一账本与停止标志。
pub(in crate::entry) struct RuntimeEntryAppFailureState {
    recorded: Arc<AtomicBool>,
    failures: ProductFailureLedger,
}

impl Default for RuntimeEntryAppFailureState {
    fn default() -> Self {
        Self::with_failure_ledger(ProductFailureLedger::default())
    }
}

impl RuntimeEntryAppFailureState {
    pub(in crate::entry) fn with_failure_ledger(failures: ProductFailureLedger) -> Self {
        Self {
            recorded: Arc::new(AtomicBool::new(false)),
            failures,
        }
    }

    /// 先登记产品失败，再发布停止标志，供其他回调按 Acquire 观察。
    pub(super) fn record(&self, failure: RuntimeEntryAppFailure) {
        self.failures.record(
            ProductHostPhase::Running,
            ProductFailureSeverity::Terminal,
            failure.component,
            failure,
        );
        self.recorded.store(true, Ordering::Release);
    }

    pub(super) fn is_recorded(&self) -> bool {
        self.recorded.load(Ordering::Acquire)
    }

    pub(in crate::entry) fn failure_ledger(&self) -> ProductFailureLedger {
        self.failures.clone()
    }
}

#[cfg(test)]
#[path = "tests/failure.rs"]
mod tests;
