pub(crate) const BOUNDED_STREAM_IO_LANE_ANCHORS: &[&str] = &[
    "pub struct BoundedStreamIoLane",
    "pub fn try_new(",
    "pub fn reader_capacity(&self)",
    "pub fn capture(",
    "TaskCancellationPolicy::FinishOnShutdown",
    ".min(runtime.task_pool(TaskPoolKind::Io).parallelism())",
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

pub(crate) const TASK_POOL_SUBMISSION_ANCHORS: &[&str] = &[
    "pub(in crate::core::runtime) struct TaskPoolSubmission",
    "pub(super) fn try_acquire_submission",
    "fn try_acquire_continuation",
    "fn wait_until_submissions_quiescent",
    "active_submission_count",
    "impl Drop for TaskPoolSubmission",
];

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
