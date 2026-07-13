use std::sync::OnceLock;

use zircon_runtime::core::runtime::tasks::JobScheduler;

use super::{EditorJobLimits, EditorJobSystem};
use crate::core::editor_message::SharedEditorMessageBus;

pub(crate) fn test_job_scheduler() -> JobScheduler {
    // Keep fixture state isolated while preventing parallel lib tests from multiplying worker pools.
    static SCHEDULER: OnceLock<JobScheduler> = OnceLock::new();
    SCHEDULER.get_or_init(JobScheduler::default).clone()
}

pub(crate) fn test_job_system() -> EditorJobSystem {
    test_job_system_with_limits(EditorJobLimits::default())
}

pub(crate) fn test_job_system_with_limits(limits: EditorJobLimits) -> EditorJobSystem {
    EditorJobSystem::with_scheduler(test_job_scheduler(), limits)
}

pub(crate) fn test_job_system_with_bus(
    bus: SharedEditorMessageBus,
    limits: EditorJobLimits,
) -> EditorJobSystem {
    EditorJobSystem::with_scheduler_and_bus(test_job_scheduler(), bus, limits)
}
