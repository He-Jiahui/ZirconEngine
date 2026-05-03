use std::fmt;
use std::sync::Arc;

use super::{RenderPassExecutionContext, RenderPassExecutorFn, RenderPassExecutorId};

pub trait RenderPassExecutor: Send + Sync {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String>;
}

struct FunctionRenderPassExecutor {
    executor: RenderPassExecutorFn,
}

impl RenderPassExecutor for FunctionRenderPassExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        (self.executor)(context)
    }
}

pub(super) fn render_pass_executor_from_fn(
    executor: RenderPassExecutorFn,
) -> Arc<dyn RenderPassExecutor> {
    Arc::new(FunctionRenderPassExecutor { executor })
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
}

impl fmt::Debug for RenderPassExecutorRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RenderPassExecutorRegistration")
            .field("executor_id", &self.executor_id)
            .finish_non_exhaustive()
    }
}
