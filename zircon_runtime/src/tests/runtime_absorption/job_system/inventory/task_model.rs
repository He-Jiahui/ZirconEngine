pub(crate) const EXPECTED_JOB_SYSTEM_MODULES: &[&str] = &[
    "bounded_stream_io/mod.rs",
    "callback_dispatcher.rs",
    "diagnostic_observation/mod.rs",
    "diagnostics.rs",
    "task_graph/mod.rs",
    "job_handle.rs",
    "job_scheduler.rs",
    "mod.rs",
    "parallel_for.rs",
    "pool.rs",
    "pools.rs",
    "report.rs",
    "retained_byte_budget.rs",
    "task_cancellation_policy.rs",
    "task_descriptor.rs",
    "task_id.rs",
    "task_pool_descriptor.rs",
    "task_pool_kind.rs",
    "task_state.rs",
    "task_status.rs",
    "thread_assignment.rs",
    "timer.rs",
];

pub(crate) const TASKS_MOD_DECLARATIONS: &[&str] = &[
    "mod bounded_stream_io;",
    "mod callback_dispatcher;",
    "mod diagnostic_observation;",
    "mod diagnostics;",
    "mod task_graph;",
    "mod job_handle;",
    "mod job_scheduler;",
    "mod parallel_for;",
    "mod pool;",
    "mod pools;",
    "mod report;",
    "mod retained_byte_budget;",
    "mod task_cancellation_policy;",
    "mod task_descriptor;",
    "mod task_id;",
    "mod task_pool_descriptor;",
    "mod task_pool_kind;",
    "mod task_state;",
    "mod task_status;",
    "mod thread_assignment;",
    "mod timer;",
];

pub(crate) const TASKS_MOD_PUBLIC_ANCHORS: &[&str] = &[
    "pub use bounded_stream_io::{",
    "BoundedStreamIoCapture,",
    "BoundedStreamIoLane,",
    "BoundedStreamIoLimits,",
    "BoundedStreamIoRecord,",
    "pub use diagnostic_observation::{",
    "TaskDiagnosticSource,",
    "TaskDiagnosticCursor,",
    "pub use diagnostics::{",
    "pub use job_handle::JobHandle;",
    "pub use job_scheduler::JobScheduler;",
    "pub use parallel_for::parallel_for;",
    "pub use task_graph::{",
    "EngineTaskGraph,",
    "TaskGraphScope,",
    "TaskHandle,",
    "TaskGraphWorkerInventory,",
    "pub use task_cancellation_policy::TaskCancellationPolicy;",
    "pub use task_descriptor::TaskDescriptor;",
    "pub use task_id::TaskId;",
    "pub use task_pool_descriptor::TaskPoolDescriptor;",
    "pub use task_pool_kind::TaskPoolKind;",
    "pub use task_state::TaskState;",
    "pub use task_status::TaskStatus;",
    "pub use retained_byte_budget::{",
    "RetainedByteBudget,",
    "RetainedByteLease,",
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

pub(crate) const BOUNDED_STREAM_IO_LANE_ANCHORS: &[&str] = &[
    "pub struct BoundedStreamIoLane",
    "pub fn try_new(",
    "pub fn reader_capacity(&self)",
    "pub fn capture(",
    "TaskCancellationPolicy::FinishOnShutdown",
    ".min(runtime.worker_pool().parallelism())",
];

pub(crate) const BOUNDED_STREAM_IO_CAPTURE_ANCHORS: &[&str] = &[
    "pub struct BoundedStreamIoCapture",
    "pub fn request_cancellation(&self)",
    "pub fn wait_until_terminal(&self",
    "pub fn drain(&self",
];

pub(crate) const BOUNDED_STREAM_IO_STATE_ANCHORS: &[&str] = &[
    "VecDeque<BoundedStreamIoRecord>",
    "queue_entry_capacity",
    "queue_byte_capacity",
    "dropped_records",
    "oldest_age",
];

pub(crate) const BOUNDED_STREAM_IO_WORKER_ANCHORS: &[&str] = &[
    "pub(super) fn run_reader(",
    "ReaderStartGate",
    "TaskCancellationToken",
    "ReaderTerminalGuard",
];

pub(crate) const TASK_DIAGNOSTIC_OBSERVATION_ANCHORS: &[&str] = &[
    "pub use batch::TaskDiagnosticBatch;",
    "pub use cursor::TaskDiagnosticCursor;",
    "pub use identity::TaskDiagnosticIdentity;",
    "pub use source::TaskDiagnosticSource;",
    "TASK_DIAGNOSTIC_RETENTION_CAPACITY: usize = 256",
    "TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES: usize = 64",
];

pub(crate) const TASK_DIAGNOSTIC_JOURNAL_ANCHORS: &[&str] = &[
    "VecDeque<TaskDiagnosticObservation>",
    "dropped_count",
    "read_after(",
    "TASK_DIAGNOSTIC_RETENTION_CAPACITY",
];

pub(crate) const CALLBACK_DISPATCHER_ANCHORS: &[&str] = &[
    "pub(super) struct TaskCallbackDispatcher",
    "pub(super) fn process_default()",
    "pub(super) fn dispatch(",
    "MAX_CALLBACKS_PER_RUN",
    "MAX_CONCURRENT_DISPATCH_RUNNERS",
];

pub(crate) const JOB_SCHEDULER_ANCHORS: &[&str] = &[
    "pub fn schedule(&self",
    "pub fn schedule_after(",
    "schedule_after_with_submission_and_prelaunch_terminal",
    "pub fn wait_all(&self",
    "PendingScheduledJob",
    "record_started",
    "record_active_terminal",
    "record_active_cancelled",
    "JobExecutionOutcome",
    "run_detached_task",
    "catch_unwind",
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
    "complete_scheduled_task",
];

pub(crate) const TASK_POOL_SUBMISSION_ANCHORS: &[&str] = &[
    "pub(super) struct TaskPoolSubmission",
    "pub(super) fn try_acquire_submission",
    "fn try_acquire_continuation",
    "fn wait_until_submissions_quiescent",
    "active_submission_count",
    "impl Drop for TaskPoolSubmission",
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
    "catch_unwind",
    "panic_message",
    "mark_panicked",
    "mark_cancelled",
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

pub(crate) const RETAINED_BYTE_BUDGET_ANCHORS: &[&str] = &[
    "pub struct RetainedByteBudget",
    "pub struct RetainedByteLease",
    "pub fn with_max_leases",
    "pub fn try_reserve",
    "impl Drop for RetainedByteLeaseInner",
];

pub(crate) const TIMER_ANCHORS: &[&str] = &[
    "pub(crate) fn process_default()",
    "pub(crate) fn schedule_at(",
    "pub(crate) struct TaskTimerSubscription",
    "spawn_named_thread(PROCESS_TIMER_THREAD_NAME",
];

pub(crate) const ENGINE_TASK_GRAPH_ANCHORS: &[&str] = &[
    "pub struct EngineTaskGraph",
    "pub fn try_new",
    "pub fn shutdown(",
    "pub fn worker_inventory(&self)",
    "worker_pool: TaskPool",
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

pub(crate) const TASK_GRAPH_SCHEDULER_ADMISSION_ANCHORS: &[&str] =
    &["struct SchedulerTaskAdmission", "TaskPoolSubmission"];

pub(crate) const TASK_GRAPH_SCOPE_ANCHORS: &[&str] = &[
    "pub struct TaskGraphScope",
    "pub fn close_admission",
    "pub fn schedule(",
    "pub fn schedule_after(",
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
    "pub fn on_terminal",
    "pub fn request_cancellation(&self)",
    "pub(crate) fn schedule_detached",
    "pub(crate) fn completed",
    "completion: JobHandle",
];

pub(crate) const TASK_CANCELLATION_POLICY_ANCHORS: &[&str] = &[
    "pub enum TaskCancellationPolicy",
    "CancelOnDrop",
    "DetachOnDrop",
    "FinishOnShutdown",
];

pub(crate) const TASK_DESCRIPTOR_ANCHORS: &[&str] = &[
    "pub struct TaskDescriptor",
    "pub id: TaskId",
    "pub kind: TaskPoolKind",
    "pub cancellation_policy: TaskCancellationPolicy",
    "pub fn with_cancellation_policy",
];

pub(crate) const TASK_ID_ANCHORS: &[&str] = &[
    "pub struct TaskId(u64)",
    "pub const fn new",
    "pub const fn raw",
];

pub(crate) const TASK_POOL_DESCRIPTOR_ANCHORS: &[&str] = &[
    "pub struct TaskPoolDescriptor",
    "pub fn with_worker_threads",
    "pub fn with_thread_name",
];

pub(crate) const TASK_POOL_KIND_ANCHORS: &[&str] =
    &["pub enum TaskPoolKind", "pub const fn default_thread_name"];

pub(crate) const TASK_STATE_ANCHORS: &[&str] = &["pub enum TaskState", "pub const fn is_terminal"];

pub(crate) const TASK_STATUS_ANCHORS: &[&str] = &[
    "pub struct TaskStatus",
    "pub fn pending",
    "pub const fn is_terminal",
    "runtime_task_status_has_one_terminal_state_and_no_poll_clock",
];
