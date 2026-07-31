#[path = "inventory/audit.rs"]
mod audit;
#[path = "inventory/behavior.rs"]
mod behavior;
#[path = "inventory/task_model.rs"]
mod task_model;

pub(super) const JOB_SYSTEM_MODULE_MAX_LINES: usize = 500;

pub(super) use audit::{
    EXPECTED_DIRECT_RAYON_PATHS, FORBIDDEN_LEGACY_DIAGNOSTIC_ANCHORS,
    FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS, MIRROR_DOC_ANCHORS, SCHEDULE_EXECUTOR_ANCHORS,
};
pub(super) use behavior::BEHAVIOR_TEST_ANCHORS;
pub(super) use task_model::{
    DIAGNOSTIC_ANCHORS, EXPECTED_JOB_SYSTEM_MODULES, JOB_HANDLE_ANCHORS, JOB_SCHEDULER_ANCHORS,
    PARALLEL_FOR_ANCHORS, REPORT_ANCHORS, TASKS_MOD_DECLARATIONS, TASKS_MOD_PUBLIC_ANCHORS,
    TIMER_ANCHORS,
};
