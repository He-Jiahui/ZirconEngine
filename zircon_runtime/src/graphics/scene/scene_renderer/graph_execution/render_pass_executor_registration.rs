use super::{RenderPassExecutorFn, RenderPassExecutorId};

#[derive(Clone, Debug)]
pub struct RenderPassExecutorRegistration {
    pub executor_id: RenderPassExecutorId,
    pub executor: RenderPassExecutorFn,
}

impl RenderPassExecutorRegistration {
    pub fn new(
        executor_id: impl Into<RenderPassExecutorId>,
        executor: RenderPassExecutorFn,
    ) -> Self {
        Self {
            executor_id: executor_id.into(),
            executor,
        }
    }

    pub fn executor_id(&self) -> &RenderPassExecutorId {
        &self.executor_id
    }
}
