use super::*;

#[test]
fn autosave_adapter_defers_snapshot_capture_until_the_admitted_mutex_turn() {
    let root = temporary_root("adapter-admission");
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Import, 1));
    let save_mutex = MutexGroup::parse("save_scene_main").unwrap();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let foreground = jobs
        .submit(
            EditorJobSpec::new("foreground-save", JobCategory::Import)
                .with_mutex_group(save_mutex.clone()),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let source = Arc::new(CountingSnapshotSource::success());
    let mut adapter = AutosaveJobAdapter::new(
        jobs.clone(),
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 32,
                |requested| {
                    assert_eq!(requested, &document);
                    Some(AutosaveDocumentRequest::new(
                        requested.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source.clone(),
                    ))
                },
            )
            .unwrap()
    );
    assert_eq!(source.capture_count(), 0);
    assert!(adapter.is_in_flight());

    release_sender.send(()).unwrap();
    assert_eq!(foreground.wait(), Ok(()));
    let completion = wait_for_autosave_completion(&mut adapter, Duration::from_secs(11));
    assert_eq!(completion.succeeded(), 1);
    assert_eq!(completion.failed(), 0);
    assert_eq!(source.capture_count(), 1);
    assert!(root.join(".zircon/autosave/scene_main/1.zscene").is_file());
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_releases_single_flight_when_atomic_admission_is_rejected() {
    let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
        EditorJobAdmissionLimits::new(0, 1024, Duration::from_secs(10)),
    ));
    let root = temporary_root("adapter-rejected");
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let source = Arc::new(CountingSnapshotSource::success());

    assert!(matches!(
        adapter.schedule(
            Duration::from_secs(10),
            &dirty,
            |_| 1,
            |requested| {
                assert_eq!(requested, &document);
                Some(AutosaveDocumentRequest::new(
                    requested.clone(),
                    AutosaveJobPolicy::for_save_mutex(
                        MutexGroup::parse("save_scene_main").unwrap(),
                    ),
                    source.clone(),
                ))
            },
        ),
        Err(AutosaveAdmissionError::JobSubmit(
            crate::core::jobs::JobSubmitError::AdmissionEntryLimitExceeded { limit: 0 }
        ))
    ));
    assert!(!adapter.is_in_flight());
    assert_eq!(source.capture_count(), 0);
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_reserves_capacity_before_materializing_requests() {
    let root = temporary_root("adapter-atomic-reservation");
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Import, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(1, 8, Duration::from_secs(10))),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let foreground = jobs
        .submit(
            EditorJobSpec::new("foreground-save", JobCategory::Import),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();

    let mut adapter = AutosaveJobAdapter::new(
        jobs.clone(),
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let requests = AtomicUsize::new(0);
    let mut queued = None;

    assert!(matches!(
        adapter.schedule(
            Duration::from_secs(10),
            &dirty,
            |_| {
                queued = Some(
                    jobs.submit(
                        EditorJobSpec::new("capacity-race", JobCategory::Import)
                            .with_estimated_bytes(8),
                        GateJob::new(mpsc::channel().0, mpsc::channel().1),
                    )
                    .unwrap(),
                );
                8
            },
            |requested| {
                requests.fetch_add(1, Ordering::AcqRel);
                Some(AutosaveDocumentRequest::new(
                    requested.clone(),
                    AutosaveJobPolicy::for_save_mutex(
                        MutexGroup::parse("save_scene_main").unwrap(),
                    ),
                    Arc::new(CountingSnapshotSource::success()),
                ))
            },
        ),
        Err(AutosaveAdmissionError::JobSubmit(
            crate::core::jobs::JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 }
        ))
    ));
    assert_eq!(requests.load(Ordering::Acquire), 0);
    assert!(!adapter.is_in_flight());

    let queued = queued.expect("the estimate hook must consume capacity once");
    assert!(jobs.cancel(queued.id()));
    assert_eq!(queued.wait(), Err(JobError::Cancelled));
    release_sender.send(()).unwrap();
    assert_eq!(foreground.wait(), Ok(()));
    remove_temporary_root(&root);
}

#[test]
fn autosave_preflight_returns_false_when_pending_byte_capacity_is_exhausted() {
    let root = temporary_root("adapter-byte-full-preflight");
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Import, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(
                8,
                16,
                Duration::from_secs(10),
            )),
    );
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let foreground = jobs
        .submit(
            EditorJobSpec::new("foreground-save", JobCategory::Import),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let (pending_started_sender, _pending_started_receiver) = mpsc::channel();
    let (_pending_release_sender, pending_release_receiver) = mpsc::channel();
    let pending = jobs
        .submit(
            EditorJobSpec::new("queued-save", JobCategory::Import).with_estimated_bytes(16),
            GateJob::new(pending_started_sender, pending_release_receiver),
        )
        .unwrap();
    let adapter = AutosaveJobAdapter::new(
        jobs.clone(),
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );

    assert!(!adapter.preflight_schedule(Duration::from_secs(10)).unwrap());
    assert!(!adapter.is_in_flight());

    assert!(jobs.cancel(pending.id()));
    assert_eq!(pending.wait(), Err(JobError::Cancelled));
    release_sender.send(()).unwrap();
    assert_eq!(foreground.wait(), Ok(()));
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_advances_after_a_write_failure_and_shutdown_rejects_new_work() {
    let root = temporary_root("adapter-failure-shutdown");
    let jobs = test_job_system_with_limits(EditorJobLimits::default());
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |requested| {
                    assert_eq!(requested, &document);
                    Some(AutosaveDocumentRequest::new(
                        requested.clone(),
                        AutosaveJobPolicy::for_save_mutex(
                            MutexGroup::parse("save_scene_main").unwrap(),
                        ),
                        Arc::new(CountingSnapshotSource::failure()),
                    ))
                },
            )
            .unwrap()
    );
    let completion = wait_for_autosave_completion(&mut adapter, Duration::from_secs(12));
    assert_eq!(completion.succeeded(), 0);
    assert_eq!(completion.failed(), 1);
    assert!(!adapter.is_in_flight());

    assert_eq!(adapter.begin_shutdown(), Vec::new());
    assert!(!adapter.is_accepting());
    assert!(matches!(
        adapter.schedule(
            Duration::from_secs(22),
            &dirty,
            |_| 1,
            |requested| {
                Some(AutosaveDocumentRequest::new(
                    requested.clone(),
                    AutosaveJobPolicy::for_save_mutex(
                        MutexGroup::parse("save_scene_main").unwrap(),
                    ),
                    Arc::new(CountingSnapshotSource::success()),
                ))
            },
        ),
        Err(AutosaveAdmissionError::ShuttingDown)
    ));
    remove_temporary_root(&root);
}
