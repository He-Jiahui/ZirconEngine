use std::sync::Arc;

use crate::core::jobs::{EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority};
use crate::core::recovery::{RestoreExecutionReport, RestoreExecutor};

use super::model::RecoveryRestoreWork;

const RECOVERY_RESTORE_JOB_LABEL: &str = "restore_autosave_documents";
const RECOVERY_RESTORE_PROGRESS_START: &str = "Restoring autosave copies";
const RECOVERY_RESTORE_PROGRESS_COMPLETE: &str = "Autosave recovery complete";

/// Background execution boundary for an already validated recovery plan.
pub(super) struct RecoveryRestoreJob {
    work: Arc<RecoveryRestoreWork>,
}

impl RecoveryRestoreJob {
    pub(super) fn new(work: Arc<RecoveryRestoreWork>) -> Self {
        Self { work }
    }

    pub(super) fn spec(work: &RecoveryRestoreWork) -> EditorJobSpec {
        EditorJobSpec::new(RECOVERY_RESTORE_JOB_LABEL, JobCategory::Misc)
            .with_priority(JobPriority::Background)
            .with_estimated_bytes(recovery_restore_estimated_bytes(work))
    }
}

impl EditorJob for RecoveryRestoreJob {
    type Output = RestoreExecutionReport;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let total = u32::try_from(self.work.plan().resolutions().len()).unwrap_or(u32::MAX);
        context.report_progress(0, total, RECOVERY_RESTORE_PROGRESS_START);
        let report = RestoreExecutor::new(self.work.project_root())
            .execute(self.work.startup(), self.work.plan())
            .map_err(JobError::failed)?;
        context.report_progress(total, total, RECOVERY_RESTORE_PROGRESS_COMPLETE);
        Ok(report)
    }
}

fn recovery_restore_estimated_bytes(work: &RecoveryRestoreWork) -> usize {
    std::mem::size_of::<RecoveryRestoreJob>().saturating_add(
        work.plan()
            .resolutions()
            .len()
            .saturating_mul(std::mem::size_of::<crate::core::recovery::RestoreResolution>()),
    )
}
