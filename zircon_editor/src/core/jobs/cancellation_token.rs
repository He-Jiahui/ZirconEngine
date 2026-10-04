//! 给票据持有者和工作上下文共享单向取消意图；工作应在可中止边界主动观察，令牌不负责中断线程或撤销已经持久化的副作用。
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 只传播取消意图；副作用何时仍可安全中止由工作端决定。
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}
