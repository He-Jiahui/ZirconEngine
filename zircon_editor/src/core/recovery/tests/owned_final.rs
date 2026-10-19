use super::*;
use crate::core::jobs::{
    test_job_scheduler, test_job_system, EditorJobLimits, EditorJobProgressObserver,
    EditorJobProgressSource, EditorJobSystem, JobId, MutexGroup,
};
use crate::core::recovery::{
    AutosaveExtension, AutosaveJobAdapter, AutosaveJobPolicy, AutosavePolicy,
    AutosaveScheduler, AutosaveSnapshot, AutosaveSnapshotProvenance, AutosaveSnapshotSource,
    AutosaveSourceDigest, AutosaveSourcePath, AutosaveStore,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Weak};
use std::time::Duration;

struct OwnedFinalAutosaveFixture {
    root: PathBuf,
    parent: PathBuf,
    root_created: std::time::SystemTime,
    parent_created: std::time::SystemTime,
}

impl OwnedFinalAutosaveFixture {
    fn checked_directory(path: &Path) -> std::fs::Metadata {
        let metadata =
            std::fs::symlink_metadata(path).expect("owned fixture directory metadata");
        assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            assert_eq!(
                metadata.file_attributes() & 0x400,
                0,
                "fixture must not be a reparse point"
            );
        }
        assert_eq!(
            path.canonicalize().expect("physical fixture directory"),
            path
        );
        metadata
    }

    fn new(label: &str) -> Self {
        let root = final_autosave_test_root(label);
        let parent = root.parent().expect("fixture output parent").to_path_buf();
        std::fs::create_dir_all(&parent).expect("create fixture output parent");
        let parent_metadata = Self::checked_directory(&parent);
        std::fs::create_dir(&root).expect("exclusively create owned fixture directory");
        let root_metadata = Self::checked_directory(&root);
        assert_eq!(root.parent(), Some(parent.as_path()));
        Self {
            root,
            parent,
            root_created: root_metadata.created().expect("fixture creation identity"),
            parent_created: parent_metadata
                .created()
                .expect("fixture parent creation identity"),
        }
    }
}

impl Drop for OwnedFinalAutosaveFixture {
    fn drop(&mut self) {
        let parent_metadata = Self::checked_directory(&self.parent);
        let root_metadata = Self::checked_directory(&self.root);
        assert_eq!(parent_metadata.created().unwrap(), self.parent_created);
        assert_eq!(root_metadata.created().unwrap(), self.root_created);
        assert_eq!(self.root.parent(), Some(self.parent.as_path()));
        assert_ne!(self.root, self.parent);
        assert!(self.root.starts_with(&self.parent));
        std::fs::remove_dir_all(&self.root).expect("remove only the owned fixture directory");
    }
}

fn wait_until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "owned final autosave did not become terminal"
        );
        std::thread::yield_now();
    }
}

fn retired(jobs: &EditorJobSystem, root: &Path, names: &[&str]) -> RetiredAutosaveProject {
    RetiredAutosaveProject {
        project_root: root.to_path_buf(),
        project_started_at: Instant::now(),
        adapter: AutosaveJobAdapter::new(
            jobs.clone(),
            AutosaveStore::new(root),
            AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
        ),
        fallback_diagnostics: names
            .iter()
            .map(|name| {
                AutosaveDocumentOutcome::shutdown_unavailable(
                    AutosaveDocumentId::parse(*name).unwrap(),
                    AutosaveSourcePath::parse("scenes/main.zscene").unwrap(),
                )
            })
            .collect(),
    }
}

struct GatedDiagnosticStore {
    items: Vec<FinalDiagnosticItem>,
    started: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
    fail: bool,
}

impl EditorJob for GatedDiagnosticStore {
    type Output = Vec<FinalDiagnosticResult>;

    fn run(self, _: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        self.release.recv_timeout(Duration::from_secs(5)).unwrap();
        if self.fail {
            return Err(JobError::Cancelled);
        }
        // A memory-store fixture acknowledges only the submitted records.
        Ok(self
            .items
            .into_iter()
            .map(|item| {
                let mut outcome = item.outcome;
                outcome.mark_diagnostic_persisted();
                FinalDiagnosticResult {
                    report_index: item.report_index,
                    outcome,
                    issues: Vec::new(),
                }
            })
            .collect())
    }
}

#[test]
fn diagnostic_real_ticket_acknowledges_only_its_submitted_retired_prefix() {
    for fail_first in [false, true] {
        let fixture = OwnedFinalAutosaveFixture::new(if fail_first {
            "prefix-failure"
        } else {
            "prefix"
        });
        let root = fixture.root.clone();
        let jobs = test_job_system();
        let service = EditorAutosaveService::new(
            jobs.clone(),
            AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
        );
        let mut drain =
            service.begin_final_autosave(None, None, Instant::now() + Duration::from_secs(5));
        let mut first = retired(&jobs, &root, &["prefix_alpha", "prefix_bravo"]);
        let items = {
            let mut state = drain.lock();
            let mut budget = DEFAULT_AUTOSAVE_COMPLETION_BUDGET;
            state.collect_retired_fallback(&mut first, &mut budget);
            state.diagnostics.iter().cloned().collect::<Vec<_>>()
        };
        let submitted_count = items.len();
        assert_eq!(submitted_count, 2);
        assert!(submitted_count < DEFAULT_AUTOSAVE_COMPLETION_BUDGET);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let ticket = jobs
            .submit(
                EditorJobSpec::new("gated memory diagnostic store", JobCategory::Misc),
                GatedDiagnosticStore {
                    items,
                    started: started_tx,
                    release: release_rx,
                    fail: fail_first,
                },
            )
            .unwrap();
        drain.lock().diagnostic_job = Some((ticket, submitted_count));
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();

        // Use the production retired-owner ingestion while the first real
        // worker remains blocked. These records were not submitted to it.
        service.lock_state().retired.push_back(retired(
            &jobs,
            &root,
            &["late_alpha", "late_bravo", "late_charlie"],
        ));
        service.poll_final_autosave(&mut drain);
        assert_eq!(drain.lock().diagnostics.len(), 5);
        assert!(!drain.succeeded());
        release_tx.send(()).unwrap();
        wait_until(|| {
            drain
                .lock()
                .diagnostic_job
                .as_ref()
                .is_some_and(|(ticket, _)| ticket.is_terminal())
        });
        service.poll_final_autosave(&mut drain);

        if fail_first {
            assert_eq!(drain.lock().diagnostics.len(), 5);
            assert!(drain.lock().diagnostic_job.is_none());
            assert!(matches!(
                drain.failure(),
                Some(FinalAutosaveFailure::DiagnosticJobFailed { .. })
            ));
            assert!(!drain.succeeded());
            service.poll_final_autosave(&mut drain);
            assert_eq!(drain.lock().diagnostics.len(), 5);
        } else {
            {
                let state = drain.lock();
                assert_eq!(state.diagnostics.len(), 3);
                assert!(state.report.outcomes[..2]
                    .iter()
                    .all(AutosaveDocumentOutcome::diagnostic_persisted));
                assert!(state.report.outcomes[2..]
                    .iter()
                    .all(|outcome| !outcome.diagnostic_persisted()));
                assert_eq!(
                    state.diagnostic_job.as_ref().map(|(_, count)| *count),
                    Some(3)
                );
            }
            assert!(!drain.succeeded());
            wait_until(|| {
                service.poll_final_autosave(&mut drain);
                drain.is_complete()
            });
            assert!(drain.succeeded());
            assert!(drain.lock().diagnostics.is_empty());
            assert!(drain
                .report()
                .outcomes()
                .iter()
                .all(AutosaveDocumentOutcome::diagnostic_persisted));
        }
    }
}

struct ReenteringObserver {
    service: Mutex<Weak<EditorAutosaveService>>,
    admissions: AtomicUsize,
    unlocked: AtomicBool,
}

impl EditorJobProgressObserver for ReenteringObserver {
    fn job_admitted(&self, _: JobId, _: &EditorJobProgressSource) {
        let Some(service) = self.service.lock().unwrap().upgrade() else {
            return;
        };
        let owned = match service.state.try_lock() {
            Ok(state) => state.final_shutdown.clone(),
            Err(_) => {
                self.unlocked.store(false, Ordering::Release);
                return;
            }
        };
        let Some(mut drain) = owned else {
            return;
        };
        if drain.inner.try_lock().is_err() {
            self.unlocked.store(false, Ordering::Release);
            return;
        }
        self.admissions.fetch_add(1, Ordering::AcqRel);
        service.poll_final_autosave(&mut drain);
        let same = service.begin_final_autosave(None, None, Instant::now());
        if same.is_complete() || service.release_drained_shutdown_adapters() {
            self.unlocked.store(false, Ordering::Release);
        }
    }
    fn job_finished(&self, _: JobId, _: &EditorJobProgressSource) {}
    fn jobs_resynchronized(&self, _: &EditorJobProgressSource) {}
}

struct SnapshotSource;

impl AutosaveSnapshotSource for SnapshotSource {
    fn source_path(&self) -> AutosaveSourcePath {
        AutosaveSourcePath::parse("scenes/main.zscene").unwrap()
    }
    fn capture(&self, _: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError> {
        Ok(AutosaveSnapshot::new(
            1,
            AutosaveExtension::parse("zscene").unwrap(),
            self.source_path(),
            AutosaveSnapshotProvenance::capture(0, AutosaveSourceDigest::missing()),
            b"observer reentry fixture".to_vec(),
        ))
    }
}

#[test]
fn final_submission_observer_can_reenter_without_locks_or_early_completion() {
    let fixture = OwnedFinalAutosaveFixture::new("observer");
    let root = fixture.root.clone();
    let observer = Arc::new(ReenteringObserver {
        service: Mutex::new(Weak::new()),
        admissions: AtomicUsize::new(0),
        unlocked: AtomicBool::new(true),
    });
    let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
        test_job_scheduler(),
        Default::default(),
        EditorJobLimits::default(),
        observer.clone(),
    );
    let service = Arc::new(EditorAutosaveService::new(
        jobs,
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    ));
    *observer.service.lock().unwrap() = Arc::downgrade(&service);
    service.poll_project(Some(&root));
    let request = AutosaveDocumentRequest::new(
        AutosaveDocumentId::parse("observer_final").unwrap(),
        AutosaveJobPolicy::for_save_mutex(MutexGroup::parse("save_observer_final").unwrap()),
        Arc::new(SnapshotSource),
    );
    let mut drain = service.begin_final_autosave(
        Some(&root),
        Some(vec![request]),
        Instant::now() + Duration::from_secs(5),
    );
    service.poll_final_autosave(&mut drain);
    assert!(observer.admissions.load(Ordering::Acquire) >= 1);
    assert!(observer.unlocked.load(Ordering::Acquire));
    assert!(!drain.is_complete());
    wait_until(|| {
        service.poll_final_autosave(&mut drain);
        drain.is_complete()
    });
    assert!(drain.succeeded());
    assert!(observer.unlocked.load(Ordering::Acquire));
}
