//! 将一次会话动作的存活期接入销毁屏障；这里只管理动作计数，不分派 owner 命令。

use std::sync::Arc;

use super::session_slot::SessionSlot;

/// Keeps a session action visible to the destroy quiescence barrier.
/// 使销毁方能等待已获准的动作完成，包括纳入 finalizer 的结果登记与 ABI 输出发布。
///
/// # 使用时机与生命周期
/// 由 [`SessionSlot::begin_action`] 或 [`SessionSlot::begin_release_action`] 先增加计数后创建。
/// guard 离开作用域时归还该次计数；仅有 `Arc` 引用不足以表示一个正在执行的动作。
/// committed 包装器返回后的 ABI 写入不在此 guard 内，非空输出另由 allocation census 约束销毁。
///
/// # 参考
/// [`SessionSlot::finish_action`]；`session_store.rs::with_session_activity_result_finalized`。
pub(super) struct SessionActionGuard {
    /// 保留被计数的同一个槽位，使归还动作计数时槽位仍然存在。
    slot: Arc<SessionSlot>,
}

impl SessionActionGuard {
    /// 将已登记的动作绑定到作用域结束时的归还操作。
    ///
    /// # 参数
    /// `slot` 必须是已经增加一次 `active_actions` 的槽位；本构造函数不会再次增加计数。
    ///
    /// # 返回值
    /// 唯一负责归还该次动作计数的 guard；上层动作包装器决定 action/finalize/commit 的持有范围。
    pub(super) fn new(slot: Arc<SessionSlot>) -> Self {
        Self { slot }
    }
}

/// 归还已获取的动作计数，最后一个动作结束时唤醒等待销毁的线程。
impl Drop for SessionActionGuard {
    fn drop(&mut self) {
        self.slot.finish_action();
    }
}
