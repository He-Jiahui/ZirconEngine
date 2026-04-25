#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraphExecutionRecord {
    executed_passes: Vec<String>,
    executed_executor_ids: Vec<String>,
}

impl RenderGraphExecutionRecord {
    pub fn push_executed_pass(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
    ) {
        self.executed_passes.push(pass_name.into());
        self.executed_executor_ids.push(executor_id.into());
    }

    pub fn executed_passes(&self) -> &[String] {
        &self.executed_passes
    }

    pub fn executed_executor_ids(&self) -> &[String] {
        &self.executed_executor_ids
    }
}
