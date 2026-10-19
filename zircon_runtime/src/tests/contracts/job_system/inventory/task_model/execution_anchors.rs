pub(crate) const JOB_SCHEDULER_ANCHORS: &[&str] = &[
    "pub fn schedule(&self",
    "pub fn schedule_after(",
    "schedule_after_with_submission_and_prelaunch_terminal",
    "pub fn wait_all(&self",
    "PendingScheduledJob",
    "record_started",
    "record_active_terminal",
    "JobExecutionOutcome",
    "run_detached_task",
    "complete_scheduled_task",
];

pub(crate) const JOB_SCHEDULER_TEST_ANCHORS: &[&str] =
    &["detached_spawn_counts_panicked_tasks_as_completed"];

pub(crate) const JOB_SCHEDULER_PENDING_ANCHORS: &[&str] = &[
    "pub(super) struct PendingScheduledJob",
    "pub(super) struct PendingScheduledWork",
    "PrelaunchTerminalHook",
    "TaskPoolSubmission",
    "record_dependency_wait",
    "record_enqueued",
    "record_terminal_without_launch",
    "record_cancelled",
    "record_active_cancelled",
    "catch_unwind",
    "complete_scheduled_task",
];

pub(crate) const JOB_HANDLE_ANCHORS: &[&str] = &[
    "pub fn combine(handles: &[JobHandle])",
    "pub fn completed() -> Self",
    "pub fn on_terminal",
    "pub fn is_cancelled",
    "pub fn terminal_state",
    "pub fn terminal_observer_panic_count",
    "pub fn wait(&self)",
    "assist_current_thread_once",
    "WORKER_WAIT_IDLE_PARK",
    "terminal_observers",
    "panic_message",
    "mark_panicked",
    "mark_cancelled",
    "mod task_node;",
    "pub(crate) use task_node::TaskNode;",
];

pub(crate) const TASK_NODE_ANCHORS: &[&str] = &[
    "pub(crate) struct TaskNode",
    "pub(in crate::core::runtime::tasks) struct TaskNodeState",
    "fn lock_inner(&self) -> MutexGuard<'_, TaskNodeState>",
    "fn wait_inner<'a>(",
    "fn wait_inner_timeout<'a>(",
    "dependency_continuations_published",
    "terminal_observers",
    "catch_unwind",
    "Condvar",
    "retain_public_handle",
    "release_public_handle",
];

pub(crate) const PARALLEL_FOR_ANCHORS: &[&str] =
    &["pub fn parallel_for", "pool.install", "par_chunks_mut"];

pub(crate) const ENGINE_TASK_GRAPH_ANCHORS: &[&str] = &[
    "pub struct EngineTaskGraph",
    "pub fn try_new",
    "pub fn shutdown(",
    "pub fn worker_inventory(&self)",
    "worker_pools: TaskPools",
    "EngineTaskGraphLifecycle",
];

pub(crate) const TASK_GRAPH_LEASE_ANCHORS: &[&str] = &[
    "pub(super) struct TaskGraphClientLease",
    "AtomicUsize",
    "fetch_add",
    "fetch_sub",
];

pub(crate) const TASK_GRAPH_CANCELLATION_ANCHORS: &[&str] = &[
    "pub struct TaskCancellationToken",
    "pub fn is_cancellation_requested",
    "pub fn acknowledge_cancellation",
];

pub(crate) const TASK_GRAPH_ADMISSION_ANCHORS: &[&str] =
    &["struct TaskAdmission", "TaskPoolSubmission"];

pub(crate) const TASK_GRAPH_SCOPE_ANCHORS: &[&str] = &[
    "pub struct TaskGraphScope",
    "pub fn close_admission",
    "pub fn submit_after(",
    "pub fn submit(",
    "Result<TaskHandle",
    "dependencies: &[TaskHandle]",
    "wait_until_quiescent",
    "TaskIdAlreadyActive",
];

pub(crate) const TASK_GRAPH_HANDLE_ANCHORS: &[&str] = &[
    "pub struct TaskHandle",
    "pub fn descriptor(&self)",
    "pub fn status(&self)",
    "pub fn is_complete(&self)",
    "pub fn is_cancelled(&self)",
    "pub fn wait(&self)",
    "pub fn wait_all(handles: &[Self])",
    "pub fn on_terminal",
    "pub fn request_cancellation(&self)",
    "pub(crate) fn schedule_detached",
    "pub(crate) fn completed",
    "completion: JobHandle",
];
