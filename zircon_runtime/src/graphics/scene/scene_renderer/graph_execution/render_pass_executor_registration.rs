use std::fmt;
use std::sync::Arc;

use super::{RenderPassExecutionContext, RenderPassExecutorFn, RenderPassExecutorId};

/// Declares whether an executor may record commands on a worker-owned encoder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderPassRecordingPolicy {
    /// The executor remains on the frame's serial encoder.
    #[default]
    Serial,
    /// Recording uses immutable prepared inputs and has no shared upload or cache mutation.
    ParallelSafe,
}

pub trait RenderPassExecutor: Send + Sync {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String>;

    /// Defaults to serial so existing and third-party executors require an explicit audit.
    fn recording_policy(&self) -> RenderPassRecordingPolicy {
        RenderPassRecordingPolicy::Serial
    }
}

struct FunctionRenderPassExecutor {
    executor: RenderPassExecutorFn,
    recording_policy: RenderPassRecordingPolicy,
}

impl RenderPassExecutor for FunctionRenderPassExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        (self.executor)(context)
    }

    fn recording_policy(&self) -> RenderPassRecordingPolicy {
        self.recording_policy
    }
}

pub(super) fn render_pass_executor_from_fn(
    executor: RenderPassExecutorFn,
) -> Arc<dyn RenderPassExecutor> {
    render_pass_executor_from_fn_with_policy(executor, RenderPassRecordingPolicy::Serial)
}

pub(super) fn render_pass_executor_from_parallel_safe_fn(
    executor: RenderPassExecutorFn,
) -> Arc<dyn RenderPassExecutor> {
    render_pass_executor_from_fn_with_policy(executor, RenderPassRecordingPolicy::ParallelSafe)
}

fn render_pass_executor_from_fn_with_policy(
    executor: RenderPassExecutorFn,
    recording_policy: RenderPassRecordingPolicy,
) -> Arc<dyn RenderPassExecutor> {
    Arc::new(FunctionRenderPassExecutor {
        executor,
        recording_policy,
    })
}

#[derive(Clone)]
pub struct RenderPassExecutorRegistration {
    pub executor_id: RenderPassExecutorId,
    pub executor: Arc<dyn RenderPassExecutor>,
}

impl RenderPassExecutorRegistration {
    pub fn new(
        executor_id: impl Into<RenderPassExecutorId>,
        executor: RenderPassExecutorFn,
    ) -> Self {
        Self::new_executor(executor_id, render_pass_executor_from_fn(executor))
    }

    pub fn new_executor(
        executor_id: impl Into<RenderPassExecutorId>,
        executor: Arc<dyn RenderPassExecutor>,
    ) -> Self {
        Self {
            executor_id: executor_id.into(),
            executor,
        }
    }

    pub fn executor_id(&self) -> &RenderPassExecutorId {
        &self.executor_id
    }

    pub fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        self.executor.execute(context)
    }

    pub fn recording_policy(&self) -> RenderPassRecordingPolicy {
        self.executor.recording_policy()
    }
}

impl fmt::Debug for RenderPassExecutorRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderPassExecutorRegistration")
            .field("executor_id", &self.executor_id)
            .finish_non_exhaustive()
    }
}
