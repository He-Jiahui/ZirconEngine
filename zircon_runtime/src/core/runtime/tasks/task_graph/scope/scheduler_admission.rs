use std::sync::Arc;

use super::super::task_handle::TaskRecord;
use crate::core::runtime::tasks::TaskPoolSubmission;

pub(super) struct SchedulerTaskAdmission {
    pub(super) record: Arc<TaskRecord>,
    pub(super) submission: TaskPoolSubmission,
}
