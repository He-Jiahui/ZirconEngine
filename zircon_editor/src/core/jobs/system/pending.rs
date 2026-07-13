use super::super::{EditorJobSpec, JobContext, JobId};

pub(super) type PendingTask = Box<dyn FnOnce(JobContext) + Send + 'static>;
pub(super) type PendingCancelTask = Box<dyn FnOnce(JobContext) + Send + 'static>;

pub(super) struct PendingJob {
    pub(super) id: JobId,
    pub(super) spec: EditorJobSpec,
    pub(super) task: PendingTask,
    pub(super) cancel_task: PendingCancelTask,
}

impl PendingJob {
    pub(super) fn new(
        id: JobId,
        spec: EditorJobSpec,
        task: PendingTask,
        cancel_task: PendingCancelTask,
    ) -> Self {
        Self {
            id,
            spec,
            task,
            cancel_task,
        }
    }
}
