use super::RenderPassExecutorId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPassExecutionContext {
    pub pass_name: String,
    pub executor_id: RenderPassExecutorId,
}

impl RenderPassExecutionContext {
    pub fn new(pass_name: impl Into<String>, executor_id: RenderPassExecutorId) -> Self {
        Self {
            pass_name: pass_name.into(),
            executor_id,
        }
    }
}
