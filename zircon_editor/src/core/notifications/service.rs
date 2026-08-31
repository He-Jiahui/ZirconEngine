use std::{
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use crate::core::jobs::{EditorJobProgressObserver, EditorJobProgressSource, JobId};

use super::progress::{AUTOMATIC_PROGRESS_SOURCE_ID, MAX_PROGRESS_NOTIFICATIONS};
use super::{
    DecisionCenterConfig, DecisionNotificationCenter, DecisionNotificationError, NotificationId,
    NotificationSource, ProgressNotification, ProgressNotificationCenter, ToastCenterConfig,
    ToastNotification, ToastNotificationCenter, ToastNotificationError, ToastNotificationSnapshot,
};

/// Context-owned notification authority. Leaf consumers resolve immutable receipts;
/// callbacks and producer-specific mutations remain outside this service.
#[derive(Default)]
pub struct EditorNotificationService {
    decisions: OnceLock<DecisionNotificationCenter>,
    progress: OnceLock<Arc<ProgressNotificationCenter>>,
    toasts: OnceLock<ToastNotificationCenter>,
    toast_epoch: OnceLock<Instant>,
}

impl EditorNotificationService {
    pub fn decisions(&self) -> Result<&DecisionNotificationCenter, DecisionNotificationError> {
        if let Some(decisions) = self.decisions.get() {
            return Ok(decisions);
        }
        let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())?;
        let _ = self.decisions.set(center);
        Ok(self
            .decisions
            .get()
            .expect("a successful notification center initialization must publish a value"))
    }

    pub fn progress(&self) -> &ProgressNotificationCenter {
        self.progress
            .get_or_init(|| Arc::new(ProgressNotificationCenter::default()))
            .as_ref()
    }

    pub(crate) fn job_progress_observer(&self) -> Arc<dyn EditorJobProgressObserver> {
        Arc::new(EditorJobProgressNotificationObserver {
            center: Arc::clone(
                self.progress
                    .get_or_init(|| Arc::new(ProgressNotificationCenter::default())),
            ),
        })
    }

    pub fn toasts(&self) -> &ToastNotificationCenter {
        self.toasts
            .get_or_init(|| ToastNotificationCenter::new(ToastCenterConfig::default()))
    }

    /// Publishes against the context-owned monotonic epoch so leaf hosts do not invent
    /// their own expiry clocks.
    pub fn publish_toast(
        &self,
        notification: ToastNotification,
    ) -> Result<(), ToastNotificationError> {
        self.toasts().publish_at(notification, self.toast_elapsed())
    }

    pub fn toast_snapshot(&self) -> Vec<ToastNotificationSnapshot> {
        let now = self.toast_elapsed();
        self.toasts().snapshot_at(now)
    }

    pub fn live_toast_snapshot(&self) -> (Duration, Vec<ToastNotificationSnapshot>) {
        let now = self.toast_elapsed();
        (now, self.toasts().snapshot_at(now))
    }

    fn toast_elapsed(&self) -> Duration {
        self.toast_epoch.get_or_init(Instant::now).elapsed()
    }
}

struct EditorJobProgressNotificationObserver {
    center: Arc<ProgressNotificationCenter>,
}

impl EditorJobProgressObserver for EditorJobProgressNotificationObserver {
    fn job_admitted(&self, job: JobId, _source: &EditorJobProgressSource) {
        if self.center.remaining_capacity() != 0 {
            self.track_job(job);
        }
    }

    fn job_finished(&self, job: JobId, source: &EditorJobProgressSource) {
        self.center.retire_job(job);
        self.refill(source);
    }

    fn jobs_resynchronized(&self, source: &EditorJobProgressSource) {
        let _ = self.center.snapshot(source);
        self.refill(source);
    }
}

impl EditorJobProgressNotificationObserver {
    fn refill(&self, source: &EditorJobProgressSource) {
        if self.center.remaining_capacity() == 0 {
            return;
        }
        for snapshot in source.snapshot_limit(MAX_PROGRESS_NOTIFICATIONS) {
            if self.center.remaining_capacity() == 0 {
                break;
            }
            self.track_job(snapshot.id());
        }
    }

    fn track_job(&self, job: JobId) {
        let Ok(id) = NotificationId::parse(format!("editor.job.progress.{}", job.value())) else {
            return;
        };
        let Ok(source) = NotificationSource::builtin(AUTOMATIC_PROGRESS_SOURCE_ID) else {
            return;
        };
        let Ok(notification) =
            ProgressNotification::new(id, source, job, "editor.notification.job_progress.title")
        else {
            return;
        };
        let _ = self.center.publish(notification);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, mpsc};
    use std::thread;
    use std::time::Duration;

    use crate::core::editor_message::SharedEditorMessageBus;
    use crate::core::jobs::{
        EditorJob, EditorJobLimits, EditorJobProgressObserver, EditorJobProgressSource,
        EditorJobSpec, EditorJobSystem, JobCategory, JobContext, JobError, JobId,
        test_job_scheduler,
    };

    use super::EditorNotificationService;

    #[test]
    fn job_progress_observer_registers_and_retires_the_authoritative_job_binding() {
        let service = EditorNotificationService::default();
        let observer = service.job_progress_observer();
        let job = JobId::new(7);
        let source = EditorJobProgressSource::default();

        observer.job_admitted(job, &source);
        assert!(!service.progress().is_empty());

        observer.job_finished(job, &source);
        assert!(service.progress().is_empty());
    }

    struct ImmediateJob;

    impl EditorJob for ImmediateJob {
        type Output = ();

        fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
            Ok(())
        }
    }

    struct GateJob {
        started: mpsc::SyncSender<()>,
        release: mpsc::Receiver<()>,
    }

    impl EditorJob for GateJob {
        type Output = ();

        fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
            self.started.send(()).unwrap();
            self.release.recv_timeout(Duration::from_secs(5)).unwrap();
            Ok(())
        }
    }

    struct DelayedFirstAdmissionObserver {
        delegate: Arc<dyn EditorJobProgressObserver>,
        delay_next_admission: AtomicBool,
        admission_entered: mpsc::SyncSender<()>,
        release_admission: Mutex<mpsc::Receiver<()>>,
    }

    impl EditorJobProgressObserver for DelayedFirstAdmissionObserver {
        fn job_admitted(&self, job: JobId, source: &EditorJobProgressSource) {
            if self.delay_next_admission.swap(false, Ordering::AcqRel) {
                self.admission_entered.send(()).unwrap();
                self.release_admission
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
            }
            self.delegate.job_admitted(job, source);
        }

        fn job_finished(&self, job: JobId, source: &EditorJobProgressSource) {
            self.delegate.job_finished(job, source);
        }

        fn jobs_resynchronized(&self, source: &EditorJobProgressSource) {
            self.delegate.jobs_resynchronized(source);
        }
    }

    struct PanickingFirstAdmissionObserver {
        delegate: Arc<dyn EditorJobProgressObserver>,
        panic_next_admission: AtomicBool,
    }

    struct PanickingAdmissionAndFirstResyncObserver {
        delegate: Arc<dyn EditorJobProgressObserver>,
        panic_next_admission: AtomicBool,
        remaining_resync_panics: AtomicUsize,
    }

    impl EditorJobProgressObserver for PanickingAdmissionAndFirstResyncObserver {
        fn job_admitted(&self, job: JobId, source: &EditorJobProgressSource) {
            if self.panic_next_admission.swap(false, Ordering::AcqRel) {
                panic!("injected progress observer admission panic");
            }
            self.delegate.job_admitted(job, source);
        }

        fn job_finished(&self, job: JobId, source: &EditorJobProgressSource) {
            self.delegate.job_finished(job, source);
        }

        fn jobs_resynchronized(&self, source: &EditorJobProgressSource) {
            if self
                .remaining_resync_panics
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |remaining| {
                    remaining.checked_sub(1)
                })
                .is_ok()
            {
                panic!("injected progress observer resynchronization panic");
            }
            self.delegate.jobs_resynchronized(source);
        }
    }

    impl EditorJobProgressObserver for PanickingFirstAdmissionObserver {
        fn job_admitted(&self, job: JobId, source: &EditorJobProgressSource) {
            if self.panic_next_admission.swap(false, Ordering::AcqRel) {
                panic!("injected progress observer admission panic");
            }
            self.delegate.job_admitted(job, source);
        }

        fn job_finished(&self, job: JobId, source: &EditorJobProgressSource) {
            self.delegate.job_finished(job, source);
        }

        fn jobs_resynchronized(&self, source: &EditorJobProgressSource) {
            self.delegate.jobs_resynchronized(source);
        }
    }

    #[test]
    fn concurrent_promotion_cannot_deliver_finish_before_admission() {
        let service = EditorNotificationService::default();
        let (admission_entered_sender, admission_entered_receiver) = mpsc::sync_channel(1);
        let (release_admission_sender, release_admission_receiver) = mpsc::sync_channel(1);
        let observer = Arc::new(DelayedFirstAdmissionObserver {
            delegate: service.job_progress_observer(),
            delay_next_admission: AtomicBool::new(true),
            admission_entered: admission_entered_sender,
            release_admission: Mutex::new(release_admission_receiver),
        });
        let jobs = Arc::new(
            EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
                test_job_scheduler(),
                SharedEditorMessageBus::default(),
                EditorJobLimits::default(),
                observer,
            ),
        );

        let first_jobs = Arc::clone(&jobs);
        let first_submit = thread::spawn(move || {
            first_jobs.submit(
                EditorJobSpec::new("delayed admission", JobCategory::Import),
                ImmediateJob,
            )
        });
        admission_entered_receiver
            .recv_timeout(Duration::from_secs(5))
            .unwrap();

        let second = jobs
            .submit(
                EditorJobSpec::new("promotion trigger", JobCategory::Import),
                ImmediateJob,
            )
            .unwrap();
        second.wait().unwrap();
        release_admission_sender.send(()).unwrap();
        first_submit.join().unwrap().unwrap().wait().unwrap();

        assert!(
            service.progress().is_empty(),
            "a late admission callback must not recreate a finished job binding"
        );
    }

    #[test]
    fn observer_panic_resynchronizes_without_unwinding_job_lifecycle() {
        let service = EditorNotificationService::default();
        let observer = Arc::new(PanickingFirstAdmissionObserver {
            delegate: service.job_progress_observer(),
            panic_next_admission: AtomicBool::new(true),
        });
        let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
            test_job_scheduler(),
            SharedEditorMessageBus::default(),
            EditorJobLimits::default(),
            observer,
        );

        jobs.submit(
            EditorJobSpec::new("observer panic recovery", JobCategory::Import),
            ImmediateJob,
        )
        .unwrap()
        .wait()
        .unwrap();

        assert!(
            jobs.shutdown(std::time::Instant::now() + Duration::from_secs(5))
                .is_empty()
        );
        assert!(service.progress().is_empty());
    }

    #[test]
    fn event_pump_retries_a_resync_stranded_by_two_observer_panics() {
        let service = EditorNotificationService::default();
        let observer = Arc::new(PanickingAdmissionAndFirstResyncObserver {
            delegate: service.job_progress_observer(),
            panic_next_admission: AtomicBool::new(true),
            remaining_resync_panics: AtomicUsize::new(1),
        });
        let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
            test_job_scheduler(),
            SharedEditorMessageBus::default(),
            EditorJobLimits::default(),
            observer,
        );
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let (release_sender, release_receiver) = mpsc::channel();
        let ticket = jobs
            .submit(
                EditorJobSpec::new("observer resync retry", JobCategory::Import),
                GateJob {
                    started: started_sender,
                    release: release_receiver,
                },
            )
            .unwrap();
        started_receiver
            .recv_timeout(Duration::from_secs(5))
            .unwrap();

        assert!(service.progress().is_empty());
        jobs.pump_events();
        assert_eq!(service.progress().snapshot(&jobs.progress()).len(), 1);

        release_sender.send(()).unwrap();
        ticket.wait().unwrap();
        assert!(service.progress().is_empty());
    }
}
