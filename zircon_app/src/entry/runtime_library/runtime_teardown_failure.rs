//! RuntimeSession 销毁错误进入产品关停账本的桥梁。
//! 入口在释放会话前克隆共享账本，供正常返回的清理与产品退出读取。
//! 会话 destroy 在 Drop 中失败会记录后终止进程，不会返回退出入口。

use crate::entry::product_shutdown::{
    ProductFailureLedger, ProductFailureSeverity, ProductHostPhase,
};

use super::RuntimeLibraryError;

#[derive(Clone, Debug, Default)]
/// 由会话析构和产品退出入口共享的销毁失败记录句柄。
pub(in crate::entry) struct RuntimeSessionTeardownFailureState(ProductFailureLedger);

impl RuntimeSessionTeardownFailureState {
    pub(super) fn record(&self, failure: RuntimeLibraryError) {
        self.0.record(
            ProductHostPhase::DestroyingRuntime,
            ProductFailureSeverity::Terminal,
            "runtime_session",
            &failure,
        );
    }

    pub(in crate::entry) fn failure_ledger(&self) -> ProductFailureLedger {
        self.0.clone()
    }
}

#[cfg(test)]
#[path = "tests/runtime_teardown_failure.rs"]
mod tests;
