//! 为待运行队列提供可替换的最小任务夹具，子回归只考察准入及公平性，避免执行器行为干扰队列状态断言。
use std::any::Any;

use crate::core::jobs::JobContext;

use super::super::pending_task::PendingTask;

pub(super) struct ReplaceablePendingTask;

impl PendingTask for ReplaceablePendingTask {
    fn run(self: Box<Self>, _context: JobContext) {}

    fn replace_with(&mut self, latest: Box<dyn PendingTask>) -> bool {
        latest.into_any().is::<Self>()
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

mod admission;
mod fairness;
