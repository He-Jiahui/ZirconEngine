from __future__ import annotations


JOB_SYSTEM_REQUIRED_DECLARATIONS = (
    "mod diagnostics;",
    "mod job_handle;",
    "mod job_scheduler;",
    "mod parallel_for;",
    "mod pool;",
    "mod pools;",
    "mod report;",
    "mod thread_assignment;",
)
JOB_SYSTEM_REQUIRED_PUBLIC_SURFACE = {
    "JobHandle": "pub use job_handle::JobHandle;",
    "JobScheduler": "pub use job_scheduler::JobScheduler;",
    "parallel_for": "pub use parallel_for::parallel_for;",
    "JobSchedulerReport": "JobSchedulerReport",
    "TASKS_SCHEDULED_DIAGNOSTIC": "TASKS_SCHEDULED_DIAGNOSTIC",
    "TASKS_COMPLETED_DIAGNOSTIC": "TASKS_COMPLETED_DIAGNOSTIC",
    "TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC": "TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC",
    "TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC": "TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC",
}
JOB_SYSTEM_API_SNIPPETS = {
    "job_scheduler.rs": (
        "pub fn schedule(&self",
        "pub fn schedule_after(",
        "pub fn wait_all(&self",
        "PendingScheduledJob",
        "record_dependency_wait",
        "record_terminal_without_launch",
        "catch_unwind",
        "complete_scheduled_task",
    ),
    "job_handle.rs": (
        "pub fn combine(handles: &[JobHandle])",
        "pub fn wait(&self)",
        "assist_current_thread_once",
        "WORKER_WAIT_IDLE_PARK",
        "panic_message",
        "mark_panicked",
        "Condvar",
    ),
    "parallel_for.rs": (
        "pub fn parallel_for",
        "pool.install",
        "par_chunks_mut",
    ),
    "diagnostics.rs": (
        '"tasks.scheduled"',
        '"tasks.completed"',
        '"tasks.dependency_wait_ms"',
        '"tasks.main_thread_wait_ms"',
    ),
    "report.rs": (
        "pub struct JobSchedulerReport",
        "record_diagnostics",
    ),
}
SCHEDULE_EXECUTOR_REQUIRED_SNIPPETS = (
    "JobHandle::completed()",
    ".schedule_after(",
    "run_parallel_tasks(",
    "scheduler.join(",
)
JOB_SYSTEM_BEHAVIOR_TEST_ANCHORS = (
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
)
MIRROR_DOCS_GUARD = "runtime_11_job_system_mirror_docs_match_structure_audit_counts"
