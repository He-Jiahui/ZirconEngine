use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use super::super::TaskHandle;
use super::model::{
    BoundedStreamIoBatch, BoundedStreamIoDiagnostics, BoundedStreamIoDrainBudget,
    BoundedStreamIoFailure,
};
use super::state::CaptureState;

pub struct BoundedStreamIoCapture {
    state: Arc<CaptureState>,
    tasks: Vec<TaskHandle>,
}

impl BoundedStreamIoCapture {
    pub(super) fn new(state: Arc<CaptureState>, tasks: Vec<TaskHandle>) -> Self {
        Self { state, tasks }
    }

    pub fn request_cancellation(&self) {
        self.state.request_cancellation();
    }

    pub fn wait_until_terminal(&self, timeout: Duration) -> bool {
        self.state.wait_until_terminal(timeout)
    }

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
