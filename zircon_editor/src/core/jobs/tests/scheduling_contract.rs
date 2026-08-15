use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use super::super::{
    CancellationToken, EditorJob, EditorJobLimits, EditorJobSpec, JobCategory, JobContext,
    JobError, JobPriority, JobSubmitError, JobTicket, MutexGroup, test_job_system,
    test_job_system_with_limits,
};
use super::RecordingJob;

#[test]
fn typed_ticket_returns_job_output_and_pre_cancelled_job_never_runs() {
    let jobs = test_job_system();
    let ticket = jobs
        .submit(EditorJobSpec::new("value", JobCategory::Misc), ValueJob(41))
        .unwrap();
    assert_eq!(ticket.wait().unwrap(), 42);

    let cancel = CancellationToken::default();
    cancel.cancel();
    let ran = Arc::new(AtomicUsize::new(0));
    let ticket = jobs
        .submit(
            EditorJobSpec::new("cancelled", JobCategory::Misc).with_cancel(cancel),
            CountingJob::new(Arc::clone(&ran)),
        )
        .unwrap();
    assert_eq!(ticket.wait(), Err(JobError::Cancelled));
    assert_eq!(ran.load(Ordering::SeqCst), 0);
}

#[test]
fn explicit_after_dependency_preserves_completion_order() {
    let jobs = test_job_system();
    let order = Arc::new(Mutex::new(Vec::new()));
    let first = jobs
        .submit(
            EditorJobSpec::new("first", JobCategory::Compile),
            DelayedRecordingJob::new("first", Arc::clone(&order)),
        )
        .unwrap();
    let second = jobs
        .submit(
            EditorJobSpec::new("second", JobCategory::Import).after(first.id()),
            RecordingJob::new("second", Arc::clone(&order)),
        )
        .unwrap();

    assert_eq!(second.wait().unwrap(), "second");
    assert_eq!(*order.lock().unwrap(), vec!["first", "second"]);
}

#[test]
fn failed_dependency_preserves_order_without_stranding_the_dependent_ticket() {
    let jobs = test_job_system();
    let failed = jobs
        .submit(
            EditorJobSpec::new("failed", JobCategory::Compile),
            FailedJob,
        )
        .unwrap();
    let dependent = jobs
        .submit(
            EditorJobSpec::new("dependent", JobCategory::Import).after(failed.id()),
            ValueJob(1),
        )
        .unwrap();

    let error = failed
        .wait()
        .expect_err("failed job should preserve its error");
    assert!(error.to_string().contains("planned failure"));
    assert_eq!(dependent.wait(), Ok(2));
}

#[test]
fn terminal_dependency_remains_available_for_late_submission() {
    let jobs = test_job_system();
    let completed = jobs
        .submit(
            EditorJobSpec::new("completed", JobCategory::Compile),
            ValueJob(1),
        )
        .unwrap();
    let completed_id = completed.id();
    assert_eq!(completed.wait(), Ok(2));
    wait_until(Duration::from_secs(5), || {
        jobs.is_terminal_record(completed_id)
    });

    let dependent = jobs
        .submit(
            EditorJobSpec::new("late dependent", JobCategory::Import).after(completed_id),
            ValueJob(2),
        )
        .unwrap();

    assert_eq!(dependent.wait(), Ok(3));
}

#[test]
fn terminal_dependency_history_is_bounded_and_reports_expired_ids() {
    let jobs = test_job_system();
    let retention_limit = jobs.terminal_record_retention_limit();
    let mut oldest_id = None;
    let mut newest_id = None;

    for index in 0..=retention_limit {
        let ticket = jobs
            .submit(
                EditorJobSpec::new(format!("terminal-{index}"), JobCategory::Misc),
                ValueJob(index as u32),
            )
            .unwrap();
        let id = ticket.id();
        oldest_id.get_or_insert(id);
        newest_id = Some(id);
        ticket.wait().unwrap();
        wait_until(Duration::from_secs(5), || jobs.is_terminal_record(id));
    }

    assert!(jobs.retained_record_count() <= retention_limit);
    let oldest_id = oldest_id.unwrap();
    let expired = jobs
        .submit(
            EditorJobSpec::new("expired", JobCategory::Misc).after(oldest_id),
            ValueJob(0),
        )
        .unwrap_err();
    assert_eq!(
        expired,
        JobSubmitError::ExpiredDependency {
            dependency: oldest_id
        }
    );

    let future_id = super::super::JobId::new(newest_id.unwrap().value() + 1);
    let unknown = jobs
        .submit(
            EditorJobSpec::new("future", JobCategory::Misc).after(future_id),
            ValueJob(0),
        )
        .unwrap_err();
    assert_eq!(
        unknown,
        JobSubmitError::UnknownDependency {
            dependency: future_id
        }
    );
}

#[test]
fn category_limit_bounds_concurrency_without_blocking_submitter() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_limit(JobCategory::Thumbnail, 2),
    );
    let active = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let tickets = (0..8)
        .map(|_| {
            jobs.submit(
                EditorJobSpec::new("thumbnail", JobCategory::Thumbnail),
                ConcurrencyProbeJob::new(Arc::clone(&active), Arc::clone(&maximum)),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    for ticket in tickets {
        ticket.wait().unwrap();
    }
    assert!(maximum.load(Ordering::SeqCst) <= 2);
}

#[test]
fn mutex_group_serializes_jobs_across_categories() {
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Import, 4)
            .with_limit(JobCategory::Compile, 4),
    );
    let group = MutexGroup::parse("script_artifacts").unwrap();
    let active = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let tickets = (0..8)
        .map(|index| {
            let category = if index % 2 == 0 {
                JobCategory::Import
            } else {
                JobCategory::Compile
            };
            jobs.submit(
                EditorJobSpec::new(format!("group-{index}"), category)
                    .with_mutex_group(group.clone()),
                ConcurrencyProbeJob::new(Arc::clone(&active), Arc::clone(&maximum)),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    for ticket in tickets {
        ticket.wait().unwrap();
    }
    assert_eq!(maximum.load(Ordering::SeqCst), 1);
    wait_until(Duration::from_secs(5), || {
        jobs.mutex_group_tail_count() == 0
    });
}

#[test]
fn fast_jobs_release_scheduled_handles_and_category_admission() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Misc, 1));
    let tickets = (0..64)
        .map(|index| {
            jobs.submit(
                EditorJobSpec::new(format!("fast-{index}"), JobCategory::Misc),
                ValueJob(index),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    for ticket in tickets {
        ticket.wait().unwrap();
    }
    wait_until(Duration::from_secs(5), || {
        jobs.scheduled_record_count() == 0 && jobs.running_job_count() == 0
    });

    let admitted_after_drain = jobs
        .submit(
            EditorJobSpec::new("admitted-after-drain", JobCategory::Misc),
            ValueJob(64),
        )
        .unwrap();
    assert_eq!(admitted_after_drain.wait(), Ok(65));
    wait_until(Duration::from_secs(5), || {
        jobs.scheduled_record_count() == 0 && jobs.running_job_count() == 0
    });
}

#[test]
fn unknown_after_dependency_is_a_typed_submit_error() {
    let jobs = test_job_system();
    let error = jobs
        .submit(
            EditorJobSpec::new("orphan", JobCategory::Misc).after(super::super::JobId::new(999)),
            ValueJob(0),
        )
        .unwrap_err();
    assert!(matches!(error, JobSubmitError::UnknownDependency { .. }));
}

#[test]
fn queued_jobs_are_admitted_by_priority_then_submission_order() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let order = Arc::new(Mutex::new(Vec::new()));
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let background = jobs
        .submit(
            EditorJobSpec::new("background", JobCategory::Export)
                .with_priority(JobPriority::Background),
            RecordingJob::new("background", Arc::clone(&order)),
        )
        .unwrap();
    let normal = jobs
        .submit(
            EditorJobSpec::new("normal", JobCategory::Export),
            RecordingJob::new("normal", Arc::clone(&order)),
        )
        .unwrap();
    let interactive = jobs
        .submit(
            EditorJobSpec::new("interactive", JobCategory::Export)
                .with_priority(JobPriority::Interactive),
            RecordingJob::new("interactive", Arc::clone(&order)),
        )
        .unwrap();

    release_sender.send(()).unwrap();
    blocker.wait().unwrap();
    interactive.wait().unwrap();
    normal.wait().unwrap();
    background.wait().unwrap();
    assert_eq!(
        *order.lock().unwrap(),
        vec!["interactive", "normal", "background"]
    );
}

#[test]
fn system_root_is_a_structural_leaf_module_entry() {
    let source = include_str!("../system/mod.rs");

    for module in [
        "construction",
        "submission",
        "lifecycle",
        "scheduling",
        "progress_observer",
    ] {
        assert!(
            source.contains(&format!("mod {module};")),
            "system root must declare the {module} owner"
        );
    }
    assert!(source.contains("pub use admission_reservation::EditorJobBatchAdmissionReservation;"));
    assert!(!source.contains("impl EditorJobSystem"));
    assert!(!source.contains("struct EditorJobSystemInner"));
}

#[test]
fn pending_behavior_contracts_stay_in_folder_backed_test_owners() {
    let pending = include_str!("../system/pending.rs");
    let tests = include_str!("../system/pending/tests/mod.rs");
    let fairness = include_str!("../system/pending/tests/fairness.rs");

    assert!(pending.contains("mod tests;"));
    assert!(!pending.contains("mod tests {"));
    assert!(tests.contains("mod admission;"));
    assert!(tests.contains("mod fairness;"));
    assert!(
        fairness.contains("ready_background_job_is_selected_within_one_weighted_fairness_round")
    );
}

#[test]
fn queued_cancel_completes_without_waiting_for_category_capacity() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let ran = Arc::new(AtomicUsize::new(0));
    let queued = jobs
        .submit(
            EditorJobSpec::new("queued", JobCategory::Export),
            CountingJob::new(Arc::clone(&ran)),
        )
        .unwrap();
    let dependent = jobs
        .submit(
            EditorJobSpec::new("dependent", JobCategory::Misc).after(queued.id()),
            ValueJob(1),
        )
        .unwrap();

    assert!(jobs.cancel(queued.id()));
    assert_eq!(
        take_before_deadline(&queued, Duration::from_secs(5)),
        Err(JobError::Cancelled)
    );
    assert_eq!(ran.load(Ordering::SeqCst), 0);
    assert_eq!(
        take_before_deadline(&dependent, Duration::from_secs(5)),
        Ok(2)
    );

    release_sender.send(()).unwrap();
    blocker.wait().unwrap();
}

#[test]
fn job_panic_is_returned_as_a_typed_error() {
    let result = test_job_system()
        .submit(EditorJobSpec::new("panic", JobCategory::Misc), PanicJob)
        .unwrap()
        .wait();

    assert_eq!(result, Err(JobError::Panicked("planned panic".to_string())));
}

#[test]
fn mutex_group_deserialization_enforces_the_same_validation_as_parse() {
    assert!(serde_json::from_str::<MutexGroup>(r#""script_artifacts""#).is_ok());
    assert!(serde_json::from_str::<MutexGroup>(r#""Script-Artifacts""#).is_err());
}

#[test]
fn pull_ticket_yields_a_result_only_once() {
    let ticket = test_job_system()
        .submit(EditorJobSpec::new("once", JobCategory::Misc), ValueJob(1))
        .unwrap();
    let result = loop {
        if let Some(result) = ticket.try_take() {
            break result;
        }
        thread::yield_now();
    };
    assert_eq!(result, Ok(2));
    assert_eq!(ticket.try_take(), None);
}

#[test]
fn join_runs_borrowing_tasks_through_the_runtime_scheduler() {
    let jobs = test_job_system();
    let values = [2_u32, 3, 5, 7];

    let (left, right) = jobs.join(
        || values[..2].iter().sum::<u32>(),
        || values[2..].iter().product::<u32>(),
    );

    assert_eq!((left, right), (5, 35));
}

#[test]
fn shutdown_after_all_jobs_finish_is_empty_idempotent_and_shared_by_clones() {
    let jobs = test_job_system();
    let clone = jobs.clone();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("finished-before-shutdown", JobCategory::Misc),
            ValueJob(41),
        )
        .unwrap();
    assert_eq!(ticket.wait(), Ok(42));

    let unfinished = jobs.shutdown(Instant::now() + Duration::from_secs(5));
    assert!(unfinished.is_empty());
    assert!(
        clone
            .shutdown(Instant::now() + Duration::from_secs(5))
            .is_empty()
    );
    assert_eq!(
        clone
            .submit(
                EditorJobSpec::new("rejected-after-shutdown", JobCategory::Misc),
                ValueJob(0),
            )
            .unwrap_err(),
        JobSubmitError::ShuttingDown
    );
}

#[test]
fn shutdown_broadcasts_cancellation_and_waits_for_cooperative_jobs() {
    let jobs = test_job_system();
    let (started_sender, started_receiver) = mpsc::channel();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("cooperative-shutdown", JobCategory::Compile),
            CooperativeCancellationJob {
                started: started_sender,
            },
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let unfinished = jobs.shutdown(Instant::now() + Duration::from_secs(5));

    assert!(unfinished.is_empty());
    assert_eq!(ticket.wait(), Err(JobError::Cancelled));
}

#[test]
fn shutdown_cancels_pending_immediately_and_reports_non_cooperative_timeout() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let running = jobs
        .submit(
            EditorJobSpec::new("running-at-deadline", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let pending = jobs
        .submit(
            EditorJobSpec::new("pending-at-shutdown", JobCategory::Export),
            ValueJob(1),
        )
        .unwrap();

    let unfinished = jobs.shutdown(Instant::now() + Duration::from_millis(20));

    assert_eq!(pending.try_take(), Some(Err(JobError::Cancelled)));
    assert_eq!(unfinished.len(), 1);
    assert_eq!(unfinished[0].id(), running.id());
    assert_eq!(unfinished[0].label(), "running-at-deadline");
    assert_eq!(unfinished[0].category(), JobCategory::Export);

    release_sender.send(()).unwrap();
    assert_eq!(running.wait(), Ok(()));
    assert!(
        jobs.shutdown(Instant::now() + Duration::from_secs(5))
            .is_empty()
    );
}

#[test]
fn simultaneous_submit_and_shutdown_linearize_as_accepted_or_shutting_down() {
    let jobs = test_job_system();
    let submit_jobs = jobs.clone();
    let shutdown_jobs = jobs.clone();
    let start = Arc::new(Barrier::new(3));
    let submit_start = Arc::clone(&start);
    let shutdown_start = Arc::clone(&start);

    let submitter = thread::spawn(move || {
        submit_start.wait();
        submit_jobs.submit(
            EditorJobSpec::new("submit-shutdown-race", JobCategory::Misc),
            ValueJob(1),
        )
    });
    let shutdown = thread::spawn(move || {
        shutdown_start.wait();
        shutdown_jobs.shutdown(Instant::now() + Duration::from_secs(5))
    });

    start.wait();
    let submission = submitter.join().unwrap();
    let unfinished = shutdown.join().unwrap();
    assert!(unfinished.is_empty());

    match submission {
        Ok(ticket) => {
            let id = ticket.id();
            assert!(matches!(
                ticket.try_take(),
                Some(Ok(2) | Err(JobError::Cancelled))
            ));
            assert!(jobs.is_terminal_record(id));
        }
        Err(error) => assert_eq!(error, JobSubmitError::ShuttingDown),
    }
}

#[test]
fn concurrent_shutdown_from_clones_converges_on_an_empty_unfinished_list() {
    let jobs = test_job_system();
    let (started_sender, started_receiver) = mpsc::channel();
    let ticket = jobs
        .submit(
            EditorJobSpec::new("concurrent-shutdown", JobCategory::Compile),
            CooperativeCancellationJob {
                started: started_sender,
            },
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let first_jobs = jobs.clone();
    let second_jobs = jobs.clone();
    let start = Arc::new(Barrier::new(3));
    let first_start = Arc::clone(&start);
    let second_start = Arc::clone(&start);
    let first = thread::spawn(move || {
        first_start.wait();
        first_jobs.shutdown(Instant::now() + Duration::from_secs(5))
    });
    let second = thread::spawn(move || {
        second_start.wait();
        second_jobs.shutdown(Instant::now() + Duration::from_secs(5))
    });

    start.wait();
    assert!(first.join().unwrap().is_empty());
    assert!(second.join().unwrap().is_empty());
    assert_eq!(ticket.try_take(), Some(Err(JobError::Cancelled)));
    assert_eq!(jobs.pending_job_count(), 0);
    assert_eq!(jobs.running_job_count(), 0);
    assert_eq!(jobs.scheduled_record_count(), 0);
    assert_eq!(jobs.mutex_group_tail_count(), 0);
}

#[test]
fn pending_cancel_shutdown_race_completes_ticket_once_and_releases_all_admission_state() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let group = MutexGroup::parse("shutdown_race").unwrap();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let running = jobs
        .submit(
            EditorJobSpec::new("shutdown-race-blocker", JobCategory::Export)
                .with_mutex_group(group.clone()),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let pending = jobs
        .submit(
            EditorJobSpec::new("shutdown-race-pending", JobCategory::Export)
                .with_mutex_group(group),
            ValueJob(1),
        )
        .unwrap();

    let cancel_jobs = jobs.clone();
    let shutdown_jobs = jobs.clone();
    let pending_id = pending.id();
    let start = Arc::new(Barrier::new(3));
    let cancel_start = Arc::clone(&start);
    let shutdown_start = Arc::clone(&start);
    let (cancel_done_sender, cancel_done_receiver) = mpsc::channel();
    let cancel = thread::spawn(move || {
        cancel_start.wait();
        let cancelled = cancel_jobs.cancel(pending_id);
        cancel_done_sender.send(cancelled).unwrap();
    });
    let shutdown = thread::spawn(move || {
        shutdown_start.wait();
        shutdown_jobs.shutdown(Instant::now() + Duration::from_secs(5))
    });

    start.wait();
    cancel_done_receiver.recv().unwrap();
    release_sender.send(()).unwrap();
    cancel.join().unwrap();
    assert!(shutdown.join().unwrap().is_empty());

    assert_eq!(pending.try_take(), Some(Err(JobError::Cancelled)));
    assert_eq!(pending.try_take(), None);
    assert!(matches!(
        running.try_take(),
        Some(Ok(()) | Err(JobError::Cancelled))
    ));
    assert_eq!(jobs.pending_job_count(), 0);
    assert_eq!(jobs.running_job_count(), 0);
    assert_eq!(jobs.scheduled_record_count(), 0);
    assert_eq!(jobs.mutex_group_tail_count(), 0);
}

struct ValueJob(u32);

fn take_before_deadline<T>(ticket: &JobTicket<T>, timeout: Duration) -> Result<T, JobError> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(result) = ticket.try_take() {
            return result;
        }
        assert!(Instant::now() < deadline, "job ticket missed its deadline");
        thread::yield_now();
    }
}

fn wait_until(timeout: Duration, condition: impl Fn() -> bool) {
    let deadline = Instant::now() + timeout;
    while !condition() {
        assert!(Instant::now() < deadline, "condition missed its deadline");
        thread::yield_now();
    }
}

impl EditorJob for ValueJob {
    type Output = u32;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(self.0 + 1)
    }
}

struct GateJob {
    started: Sender<()>,
    release: Receiver<()>,
}

impl GateJob {
    fn new(started: Sender<()>, release: Receiver<()>) -> Self {
        Self { started, release }
    }
}

impl EditorJob for GateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        let _ = self.started.send(());
        self.release.recv().map_err(JobError::failed)
    }
}

struct CooperativeCancellationJob {
    started: Sender<()>,
}

impl EditorJob for CooperativeCancellationJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        let _ = self.started.send(());
        loop {
            context.check_cancelled()?;
            thread::yield_now();
        }
    }
}

struct PanicJob;

impl EditorJob for PanicJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        panic!("planned panic");
    }
}

struct FailedJob;

impl EditorJob for FailedJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Err(JobError::failed(std::io::Error::other("planned failure")))
    }
}

struct CountingJob {
    ran: Arc<AtomicUsize>,
}

impl CountingJob {
    fn new(ran: Arc<AtomicUsize>) -> Self {
        Self { ran }
    }
}

impl EditorJob for CountingJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.ran.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct DelayedRecordingJob {
    label: &'static str,
    order: Arc<Mutex<Vec<&'static str>>>,
}

impl DelayedRecordingJob {
    fn new(label: &'static str, order: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self { label, order }
    }
}

impl EditorJob for DelayedRecordingJob {
    type Output = &'static str;

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        thread::sleep(Duration::from_millis(20));
        self.order.lock().unwrap().push(self.label);
        Ok(self.label)
    }
}

struct ConcurrencyProbeJob {
    active: Arc<AtomicUsize>,
    maximum: Arc<AtomicUsize>,
}

impl ConcurrencyProbeJob {
    fn new(active: Arc<AtomicUsize>, maximum: Arc<AtomicUsize>) -> Self {
        Self { active, maximum }
    }
}

impl EditorJob for ConcurrencyProbeJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum.fetch_max(active, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(20));
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}
