pub(crate) const EXPECTED_JOB_SYSTEM_MODULES: &[&str] = &[
    "diagnostics",
    "job_handle",
    "job_scheduler",
    "mod",
    "parallel_for",
    "pool",
    "pools",
    "report",
    "thread_assignment",
    "timer",
];

pub(crate) const TASKS_MOD_DECLARATIONS: &[&str] = &[
    "mod diagnostics;",
    "mod job_handle;",
    "mod job_scheduler;",
    "mod parallel_for;",
    "mod pool;",
    "mod pools;",
    "mod report;",
    "mod thread_assignment;",
    "mod timer;",
];

pub(crate) const TASKS_MOD_PUBLIC_ANCHORS: &[&str] = &[
    "pub use diagnostics::{",
    "pub use job_handle::JobHandle;",
    "pub use job_scheduler::JobScheduler;",
    "pub use parallel_for::parallel_for;",
    "pub use report::{JobSchedulerReport, TaskPoolReport, TaskPoolReportEntry};",
    "pub(crate) use timer::{TaskTimer, TaskTimerSubscription};",
    "TASKS_SCHEDULED_DIAGNOSTIC",
    "TASKS_COMPLETED_DIAGNOSTIC",
    "TASKS_DEPENDENCY_WAITING_DIAGNOSTIC",
    "TASKS_QUEUED_DIAGNOSTIC",
    "TASKS_ACTIVE_DIAGNOSTIC",
    "TASKS_QUEUE_WAIT_SAMPLES_DIAGNOSTIC",
    "TASKS_QUEUE_WAIT_MS_DIAGNOSTIC",
    "TASKS_PANICKED_DIAGNOSTIC",
    "TASKS_CANCELLED_DIAGNOSTIC",
    "TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC",
    "TASKS_EXPLICIT_WAIT_MS_DIAGNOSTIC",
];

pub(crate) const JOB_SCHEDULER_ANCHORS: &[&str] = &[
    "pub fn schedule(&self",
    "pub fn schedule_after(",
    "pub fn wait_all(&self",
    "PendingScheduledJob",
    "record_dependency_wait",
    "record_terminal_without_launch",
    "record_enqueued",
    "record_started",
    "record_active_terminal",
    "record_cancelled",
    "run_detached_task",
    "detached_spawn_counts_panicked_tasks_as_completed",
    "catch_unwind",
    "complete_scheduled_task",
];

pub(crate) const JOB_HANDLE_ANCHORS: &[&str] = &[
    "pub fn combine(handles: &[JobHandle])",
    "pub fn completed() -> Self",
    "pub fn on_terminal",
    "pub fn terminal_observer_panic_count",
    "pub fn wait(&self)",
    "assist_current_thread_once",
    "WORKER_WAIT_IDLE_PARK",
    "terminal_observers",
    "catch_unwind",
    "panic_message",
    "mark_panicked",
    "Condvar",
];

pub(crate) const PARALLEL_FOR_ANCHORS: &[&str] =
    &["pub fn parallel_for", "pool.install", "par_chunks_mut"];

pub(crate) const DIAGNOSTIC_ANCHORS: &[&str] = &[
    "\"tasks.scheduled\"",
    "\"tasks.completed\"",
    "\"tasks.dependency_waiting\"",
    "\"tasks.queued\"",
    "\"tasks.active\"",
    "\"tasks.queue_wait_samples\"",
    "\"tasks.queue_wait_ms\"",
    "\"tasks.panicked\"",
    "\"tasks.cancelled\"",
    "\"tasks.dependency_wait_ms\"",
    "\"tasks.explicit_wait_ms\"",
];

pub(crate) const REPORT_ANCHORS: &[&str] = &["pub struct JobSchedulerReport", "record_diagnostics"];

pub(crate) const TIMER_ANCHORS: &[&str] = &[
    "pub(crate) fn process_default()",
    "pub(crate) fn schedule_at(",
    "pub(crate) struct TaskTimerSubscription",
    "spawn_named_thread(PROCESS_TIMER_THREAD_NAME",
];
