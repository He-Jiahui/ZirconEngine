//! 合并短时间内重复的窗口焦点刷新，避免多个工作者等待同一个运行时锁。
//! 许可跨线程持有，释放或展开退栈时重新开放下一次刷新。

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// 命令状态共享的非重入准入门；已有刷新在执行时，后续焦点事件被合并。
#[derive(Clone, Default)]
pub(super) struct FocusRefreshGate {
    pending: Arc<AtomicBool>,
}

impl FocusRefreshGate {
    /// 在启动刷新工作者前获取许可；调用者须把许可保留到整个刷新结束。
    pub(super) fn try_enter(&self) -> Option<FocusRefreshPermit> {
        // 热点是已有工作者时的重复事件；只读拒绝避免每次焦点事件都写共享原子。
        if self.pending.load(Ordering::Acquire) {
            return None;
        }
        self.pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()?;
        Some(FocusRefreshPermit {
            pending: Arc::clone(&self.pending),
        })
    }
}

/// 拥有一次刷新占用期；移入工作者可使正常退出和异常展开统一释放准入。
pub(super) struct FocusRefreshPermit {
    pending: Arc<AtomicBool>,
}

impl Drop for FocusRefreshPermit {
    fn drop(&mut self) {
        self.pending.store(false, Ordering::Release);
    }
}

#[cfg(test)]
#[path = "tests/focus_refresh_gate.rs"]
mod tests;
