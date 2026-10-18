use super::*;

#[test]
fn final_autosave_bypasses_the_periodic_deadline_and_fences_regular_admission() {
    let root = temporary_root("adapter-final-autosave");
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let source = Arc::new(CountingSnapshotSource::success());
    let mut adapter = AutosaveJobAdapter::new(
        test_job_system_with_limits(EditorJobLimits::resolved(1, [])),
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );

    assert!(adapter
        .schedule_final(
            Duration::ZERO,
            &dirty,
            |_| 1,
            |requested| {
                assert_eq!(requested, &document);
                Some(AutosaveDocumentRequest::new(
                    requested.clone(),
                    AutosaveJobPolicy::for_save_mutex(
                        MutexGroup::parse("save_final_autosave").unwrap(),
                    ),
                    source.clone(),
                ))
            }
        )
        .unwrap());
    assert!(!adapter.is_accepting());
    assert!(matches!(
        adapter.schedule(Duration::from_secs(10), &dirty, |_| 1, |_| None),
        Err(AutosaveAdmissionError::ShuttingDown)
    ));

    let completion = wait_for_autosave_completion(&mut adapter, Duration::ZERO);
    assert_eq!(completion.succeeded(), 1);
    assert_eq!(completion.failed(), 0);
    assert_eq!(source.capture_count(), 1);
    remove_temporary_root(&root);
}

#[test]
fn service_drains_a_final_snapshot_before_shutting_down_the_shared_job_system() {
    let root = temporary_root("service-final-autosave");
    let document = document_id("scene_main");
    let source = Arc::new(CountingSnapshotSource::success());
    let service = EditorAutosaveService::new(
        test_job_system_with_limits(EditorJobLimits::resolved(1, [])),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );
    service.poll_project(Some(&root));

    let shutdown = service.shutdown_with_final_autosave(
        vec![AutosaveDocumentRequest::new(
            document.clone(),
            AutosaveJobPolicy::for_save_mutex(
                MutexGroup::parse("save_service_final_autosave").unwrap(),
            ),
            source.clone(),
        )],
        Instant::now() + Duration::from_secs(5),
    );

    assert!(shutdown.unfinished_jobs().is_empty());
    assert_eq!(shutdown.outcomes().len(), 1);
    assert_eq!(shutdown.outcomes()[0].document(), &document);
    assert!(matches!(
        shutdown.outcomes()[0].kind(),
        AutosaveDocumentOutcomeKind::Saved { .. }
    ));
    assert!(shutdown.diagnostic_persistence_issues().is_empty());
    assert_eq!(source.capture_count(), 1);
    assert!(root.join(".zircon/autosave/scene_main/1.zscene").is_file());
    remove_temporary_root(&root);
}

#[test]
fn service_reports_unbound_final_requests_without_releasing_them_silently() {
    let document = document_id("scene_main");
    let source = Arc::new(CountingSnapshotSource::success());
    let service = EditorAutosaveService::new(
        test_job_system_with_limits(EditorJobLimits::resolved(1, [])),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );

    let shutdown = service.shutdown_with_final_autosave(
        vec![AutosaveDocumentRequest::new(
            document.clone(),
            AutosaveJobPolicy::for_save_mutex(
                MutexGroup::parse("save_unbound_final_autosave").unwrap(),
            ),
            source.clone(),
        )],
        Instant::now() + Duration::from_secs(5),
    );

    assert!(shutdown.unfinished_jobs().is_empty());
    assert_eq!(shutdown.outcomes().len(), 1);
    assert_eq!(shutdown.outcomes()[0].document(), &document);
    assert_eq!(
        shutdown.outcomes()[0].failure_stage(),
        Some(AutosaveFailureStage::JobLifecycle)
    );
    assert_eq!(source.capture_count(), 0);
}

#[test]
fn service_drains_all_final_windows_when_admission_allows_one_document() {
    let root = temporary_root("service-final-autosave-windows");
    let first_document = document_id("scene_alpha");
    let second_document = document_id("scene_bravo");
    let source = Arc::new(CountingSnapshotSource::success());
    let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
        EditorJobAdmissionLimits::new(1, 1024, Duration::from_secs(5)),
    ));
    let service =
        EditorAutosaveService::new(jobs, AutosavePolicy::new(Duration::from_secs(10)).unwrap());
    service.poll_project(Some(&root));

    let shutdown = service.shutdown_with_final_autosave(
        vec![
            AutosaveDocumentRequest::new(
                first_document.clone(),
                AutosaveJobPolicy::for_save_mutex(
                    MutexGroup::parse("save_service_final_alpha").unwrap(),
                ),
                source.clone(),
            ),
            AutosaveDocumentRequest::new(
                second_document.clone(),
                AutosaveJobPolicy::for_save_mutex(
                    MutexGroup::parse("save_service_final_bravo").unwrap(),
                ),
                source.clone(),
            ),
        ],
        Instant::now() + Duration::from_secs(5),
    );

    assert!(shutdown.unfinished_jobs().is_empty());
    assert!(shutdown.diagnostic_persistence_issues().is_empty());
    assert_eq!(shutdown.outcomes().len(), 2);
    assert!(shutdown
        .outcomes()
        .iter()
        .all(|outcome| matches!(outcome.kind(), AutosaveDocumentOutcomeKind::Saved { .. })));
    assert_eq!(source.capture_count(), 2);
    assert!(root.join(".zircon/autosave/scene_alpha/1.zscene").is_file());
    assert!(root.join(".zircon/autosave/scene_bravo/1.zscene").is_file());
    remove_temporary_root(&root);
}

struct Editor09GateSnapshot {
    captures: AtomicUsize,
    started: mpsc::Sender<()>,
    release: std::sync::Mutex<mpsc::Receiver<()>>,
}

impl AutosaveSnapshotSource for Editor09GateSnapshot {
    fn source_path(&self) -> AutosaveSourcePath {
        recovery_source_path("scenes/main.zscene")
    }

    fn capture(&self, _: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError> {
        self.captures.fetch_add(1, Ordering::AcqRel);
        self.started.send(()).unwrap();
        self.release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(5))
            .unwrap();
        Ok(AutosaveSnapshot::new(
            1,
            extension("zscene"),
            self.source_path(),
            AutosaveSnapshotProvenance::capture(0, AutosaveSourceDigest::missing()),
            b"owned final-close fixture".to_vec(),
        ))
    }
}

fn editor09_poll_until(
    service: &EditorAutosaveService,
    drain: &mut crate::core::recovery::FinalAutosaveDrain,
    mut ready: impl FnMut(&crate::core::recovery::FinalAutosaveDrain) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        service.poll_final_autosave(drain);
        if ready(drain) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "final autosave did not become terminal"
        );
        thread::yield_now();
    }
}

#[test]
fn discard_drain_retains_old_real_task_and_does_not_create_a_final_save() {
    use crate::core::jobs::EditorJobShutdownState;
    let root = crate::core::recovery::autosave_shutdown::final_autosave_test_root("discard");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(1, []));
    let service = EditorAutosaveService::new(
        jobs.clone(),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );
    service.poll_project(Some(&root));
    let document = document_id("scene_discard");
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let source = Arc::new(Editor09GateSnapshot {
        captures: AtomicUsize::new(0),
        started: started_tx,
        release: std::sync::Mutex::new(release_rx),
    });
    assert!(service
        .schedule(
            Duration::from_secs(10),
            &[AutosaveDocumentState::from_dirty_for_test(
                document.clone(),
                true
            )],
            |_| 1,
            |_| Some(AutosaveDocumentRequest::new(
                document.clone(),
                AutosaveJobPolicy::for_save_mutex(
                    MutexGroup::parse("save_editor09_discard").unwrap()
                ),
                source.clone(),
            )),
        )
        .unwrap());
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();

    let mut drain =
        service.begin_final_autosave(Some(&root), None, Instant::now() + Duration::from_secs(5));
    service.poll_final_autosave(&mut drain);
    assert!(!drain.is_complete());
    assert!(!drain.ready_to_shutdown_jobs());
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::NotRequested
    );
    assert_eq!(source.captures.load(Ordering::Acquire), 1);
    assert!(!service.release_drained_shutdown_adapters());
    release_tx.send(()).unwrap();
    editor09_poll_until(&service, &mut drain, |drain| drain.is_complete());
    assert!(drain.succeeded());
    assert_eq!(source.captures.load(Ordering::Acquire), 1);
    assert_eq!(drain.report().outcomes().len(), 1);
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::NotRequested
    );

    jobs.request_shutdown();
    let deadline = Instant::now() + Duration::from_secs(5);
    while jobs.shutdown_progress().state() != EditorJobShutdownState::Quiescent {
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert!(service.release_drained_shutdown_adapters());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_final_write_keeps_recovery_owner_and_never_seals_shared_jobs() {
    use crate::core::jobs::EditorJobShutdownState;
    use crate::core::recovery::FinalAutosaveFailure;
    let root = crate::core::recovery::autosave_shutdown::final_autosave_test_root("failed");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(1, []));
    let service = EditorAutosaveService::new(
        jobs.clone(),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );
    service.poll_project(Some(&root));
    let document = document_id("scene_final_failed");
    let source = Arc::new(CountingSnapshotSource::failure());
    let request = AutosaveDocumentRequest::new(
        document.clone(),
        AutosaveJobPolicy::for_save_mutex(MutexGroup::parse("save_editor09_failed").unwrap()),
        source.clone(),
    );
    let mut drain = service.begin_final_autosave(
        Some(&root),
        Some(vec![request]),
        Instant::now() + Duration::from_secs(5),
    );
    editor09_poll_until(&service, &mut drain, |drain| drain.is_complete());
    assert!(matches!(
        drain.failure(),
        Some(FinalAutosaveFailure::DocumentFailed { .. })
    ));
    assert!(!drain.succeeded());
    assert!(!drain.ready_to_shutdown_jobs());
    assert_eq!(drain.remaining_documents(), 1);
    assert_eq!(source.capture_count(), 1);
    let mut same =
        service.begin_final_autosave(Some(&root), None, Instant::now() + Duration::from_secs(5));
    service.poll_final_autosave(&mut same);
    assert_eq!(same.remaining_documents(), 1);
    assert_eq!(source.capture_count(), 1);
    assert!(!service.release_drained_shutdown_adapters());
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::NotRequested
    );
    assert!(jobs.pending_admission_window().is_ok());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn final_deadline_keeps_an_unfinished_real_adapter_owned() {
    use crate::core::jobs::EditorJobShutdownState;
    use crate::core::recovery::FinalAutosaveFailure;
    let root = crate::core::recovery::autosave_shutdown::final_autosave_test_root("deadline");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(1, []));
    let service = EditorAutosaveService::new(
        jobs.clone(),
        AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
    );
    service.poll_project(Some(&root));
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let source = Arc::new(Editor09GateSnapshot {
        captures: AtomicUsize::new(0),
        started: started_tx,
        release: std::sync::Mutex::new(release_rx),
    });
    let document = document_id("scene_deadline");
    assert!(service
        .schedule(
            Duration::from_secs(10),
            &[AutosaveDocumentState::from_dirty_for_test(
                document.clone(),
                true
            )],
            |_| 1,
            |_| Some(AutosaveDocumentRequest::new(
                document.clone(),
                AutosaveJobPolicy::for_save_mutex(
                    MutexGroup::parse("save_editor09_deadline").unwrap(),
                ),
                source.clone(),
            )),
        )
        .unwrap());
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut drain = service.begin_final_autosave(Some(&root), None, Instant::now());
    service.poll_final_autosave(&mut drain);
    assert!(matches!(
        drain.failure(),
        Some(FinalAutosaveFailure::DeadlineExceeded)
    ));
    assert!(!drain.is_complete());
    assert!(!drain.ready_to_shutdown_jobs());
    assert!(!service.release_drained_shutdown_adapters());
    assert_eq!(
        jobs.shutdown_progress().state(),
        EditorJobShutdownState::NotRequested
    );
    release_tx.send(()).unwrap();
    editor09_poll_until(&service, &mut drain, |drain| drain.is_complete());
    assert!(!drain.succeeded());
    assert_eq!(source.captures.load(Ordering::Acquire), 1);
    std::fs::remove_dir_all(root).unwrap();
}
