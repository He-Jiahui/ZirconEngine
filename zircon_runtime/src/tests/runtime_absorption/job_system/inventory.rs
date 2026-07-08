pub(super) const JOB_SYSTEM_MODULE_MAX_LINES: usize = 500;

pub(super) const EXPECTED_JOB_SYSTEM_MODULES: &[&str] = &[
    "diagnostics",
    "job_handle",
    "job_scheduler",
    "mod",
    "parallel_for",
    "pool",
    "pools",
    "report",
    "thread_assignment",
];

pub(super) const EXPECTED_DIRECT_RAYON_PATHS: &[&str] = &[
    "src/core/runtime/tasks/parallel_for.rs",
    "src/core/runtime/tasks/pool.rs",
];

pub(super) const TASKS_MOD_DECLARATIONS: &[&str] = &[
    "mod diagnostics;",
    "mod job_handle;",
    "mod job_scheduler;",
    "mod parallel_for;",
    "mod pool;",
    "mod pools;",
    "mod report;",
    "mod thread_assignment;",
];

pub(super) const TASKS_MOD_PUBLIC_ANCHORS: &[&str] = &[
    "pub use diagnostics::{",
    "pub use job_handle::JobHandle;",
    "pub use job_scheduler::JobScheduler;",
    "pub use parallel_for::parallel_for;",
    "pub use report::{JobSchedulerReport, TaskPoolReport, TaskPoolReportEntry};",
    "TASKS_SCHEDULED_DIAGNOSTIC",
    "TASKS_COMPLETED_DIAGNOSTIC",
    "TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC",
    "TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC",
];

pub(super) const JOB_SCHEDULER_ANCHORS: &[&str] = &[
    "pub fn schedule(&self",
    "pub fn schedule_after(",
    "pub fn wait_all(&self",
    "PendingScheduledJob",
    "record_dependency_wait",
    "record_terminal_without_launch",
    "catch_unwind",
    "complete_scheduled_task",
];

pub(super) const JOB_HANDLE_ANCHORS: &[&str] = &[
    "pub fn combine(handles: &[JobHandle])",
    "pub fn completed() -> Self",
    "pub fn wait(&self)",
    "assist_current_thread_once",
    "WORKER_WAIT_IDLE_PARK",
    "panic_message",
    "mark_panicked",
    "Condvar",
];

pub(super) const PARALLEL_FOR_ANCHORS: &[&str] =
    &["pub fn parallel_for", "pool.install", "par_chunks_mut"];

pub(super) const DIAGNOSTIC_ANCHORS: &[&str] = &[
    "\"tasks.scheduled\"",
    "\"tasks.completed\"",
    "\"tasks.dependency_wait_ms\"",
    "\"tasks.main_thread_wait_ms\"",
];

pub(super) const REPORT_ANCHORS: &[&str] = &["pub struct JobSchedulerReport", "record_diagnostics"];

pub(super) const BEHAVIOR_TEST_ANCHORS: &[&str] = &[
    "job_handle_wait_blocks_until_task_completes",
    "job_handle_wait_reports_task_panic_without_leaking_completion",
    "schedule_after_runs_task_only_after_all_dependencies",
    "schedule_after_propagates_dependency_panic_without_running_dependent_task",
    "combined_handle_completes_when_all_children_complete",
    "schedule_after_does_not_consume_worker_while_waiting_on_dependencies",
    "job_diagnostics_track_schedule_complete_and_wait_times",
    "deep_dependency_chain_completes_in_order",
    "wide_fanout_combine_waits_for_all",
    "scheduler_wait_all_waits_for_all_handles_and_records_sync_time",
    "worker_thread_wait_does_not_deadlock_scheduler",
    "parallel_for_visits_every_item_exactly_once",
    "parallel_for_chunk_size_bounds_task_granularity",
];

pub(super) const SCHEDULE_EXECUTOR_ANCHORS: &[&str] = &[
    "JobHandle::completed()",
    ".schedule_after(",
    "run_parallel_tasks(",
    "scheduler.join(",
];

pub(super) const FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS: &[&str] = &[
    "use rayon",
    "rayon::",
    ".par_iter(",
    ".par_chunks",
    ".into_par_iter(",
];

pub(super) const MIRROR_DOC_ANCHORS: &[&str] = &[
    "job_system_boundary",
    "expected_module_count = 9",
    "direct_rayon_paths = 2",
    "schedule_parallel_executor_direct_rayon = []",
    "diagnostic_anchor_count = 4",
    "behavior_test_anchor_count = 13",
    "missing_behavior_test_anchors = []",
    "oversized_modules = []",
    "mirror_docs_guard_present = true",
    "risks = []",
    "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
];
