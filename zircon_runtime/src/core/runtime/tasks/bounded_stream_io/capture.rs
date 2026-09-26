use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use super::super::TaskHandle;
use super::model::{
    BoundedStreamIoBatch, BoundedStreamIoDiagnostics, BoundedStreamIoDrainBudget,
    BoundedStreamIoFailure,
};
use super::state::CaptureState;

// TODO: [CR-RUNTIME-TASKS-0002] 确认取消请求是否应让持续产出的读取器提前退出；当前工作循环仅在 EOF 或错误时检查取消标志，成功读取后只检查消费端是否关闭。
/// 一组已原子接纳的阻塞流读取任务及其有界输出队列，由 BoundedStreamIoLane::capture 创建。
/// 丢弃仅关闭消费端并发出协作取消；若底层 Read 仍阻塞，任务仍由 Runtime 计入停机等待。
pub struct BoundedStreamIoCapture {
    state: Arc<CaptureState>,
    tasks: Vec<TaskHandle>,
}

impl BoundedStreamIoCapture {
    pub(super) fn new(state: Arc<CaptureState>, tasks: Vec<TaskHandle>) -> Self {
        Self { state, tasks }
    }

    /// 请求读取任务在终结时报告取消；此信号本身不关闭或中断底层阻塞的 Read。
    pub fn request_cancellation(&self) {
        self.state.request_cancellation();
    }

    pub fn wait_until_terminal(&self, timeout: Duration) -> bool {
        self.state.wait_until_terminal(timeout)
    }

    /// 按消费预算取出输出记录；积压超限时生产端会丢弃整条记录而非阻塞读线程。
    pub fn drain(&self, budget: BoundedStreamIoDrainBudget) -> BoundedStreamIoBatch {
        self.state.drain(budget)
    }

    pub fn diagnostics(&self) -> BoundedStreamIoDiagnostics {
        self.state.diagnostics()
    }

    pub fn failures(&self) -> Vec<BoundedStreamIoFailure> {
        self.state.failures()
    }
}

impl fmt::Debug for BoundedStreamIoCapture {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundedStreamIoCapture")
            .field("task_count", &self.tasks.len())
            .field("diagnostics", &self.diagnostics())
            .finish()
    }
}

impl Drop for BoundedStreamIoCapture {
    fn drop(&mut self) {
        self.state.close_consumer();
        self.request_cancellation();
    }
}
