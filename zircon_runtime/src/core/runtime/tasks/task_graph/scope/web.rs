use super::*;
use std::future::Future;
impl TaskGraphScope {
    /// Browser work keeps both scope admission and the pool's bounded submission lease
    /// until the future finishes, including every asynchronous Fetch wait.
    pub fn submit_local_future<F: Future<Output = ()> + 'static>(
        &self,
        descriptor: TaskDescriptor,
        task: impl FnOnce(TaskCancellationToken) -> F + 'static,
    ) -> Result<TaskHandle, TaskGraphAdmissionError> {
        let graph = self
            .graph
            .upgrade()
            .ok_or(TaskGraphAdmissionError::RuntimeUnavailable)?;
        let scheduler = graph.scheduler_for(descriptor.kind);
        let admission = self.admit(&graph, &scheduler, descriptor, 0)?;
        let completion = admission.completion;
        let handle = TaskHandle::new(completion.clone(), Some(Arc::clone(&self.inner)));
        let scope = Arc::clone(&self.inner);
        let task_id = completion.descriptor().id;
        wasm_bindgen_futures::spawn_local(async move {
            let _submission = admission.submission;
            completion.mark_running();
            let outcome = if let Some(token) = scope.begin(task_id) {
                task(token).await;
                scope.finish(task_id, None)
            } else {
                JobExecutionOutcome::Cancelled
            };
            match &outcome {
                JobExecutionOutcome::Completed => completion.mark_complete(),
                JobExecutionOutcome::Cancelled => completion.mark_cancelled(),
                JobExecutionOutcome::Panicked(message) => {
                    completion.mark_panicked(Arc::clone(message))
                }
            }
            scope.retire(task_id, &outcome);
        });
        Ok(handle)
    }
}
