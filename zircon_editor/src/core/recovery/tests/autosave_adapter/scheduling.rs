use super::*;

#[test]
fn autosave_adapter_materializes_a_bounded_fair_window_for_large_dirty_sets() {
    let root = temporary_root("adapter-bounded-window");
    let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
        EditorJobAdmissionLimits::new(3, 1024, Duration::from_secs(10)),
    ));
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let dirty = (0..10_000)
        .map(|index| {
            AutosaveDocumentState::from_dirty_for_test(
                document_id(&format!("scene_{index:05}")),
                true,
            )
        })
        .collect::<Vec<_>>();
    let source = Arc::new(CountingSnapshotSource::success());
    let save_mutex = MutexGroup::parse("save_scene_window").unwrap();

    let mut first_window = Vec::new();
    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |document| {
                    first_window.push(document.as_str().to_string());
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source.clone(),
                    ))
                },
            )
            .unwrap()
    );
    assert_eq!(first_window, ["scene_00000", "scene_00001", "scene_00002"]);
    assert_eq!(
        wait_for_autosave_completion(&mut adapter, Duration::from_secs(11)).succeeded(),
        3
    );

    let mut second_window = Vec::new();
    assert!(
        adapter
            .schedule(
                Duration::from_secs(21),
                &dirty,
                |_| 1,
                |document| {
                    second_window.push(document.as_str().to_string());
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source.clone(),
                    ))
                },
            )
            .unwrap()
    );
    assert_eq!(second_window, ["scene_00003", "scene_00004", "scene_00005"]);
    assert_eq!(
        wait_for_autosave_completion(&mut adapter, Duration::from_secs(22)).succeeded(),
        3
    );
    assert_eq!(source.capture_count(), 6);
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_applies_the_byte_window_before_request_materialization() {
    let root = temporary_root("adapter-byte-window");
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_admission_limits(EditorJobAdmissionLimits::new(
            8,
            2,
            Duration::from_secs(10),
        )),
    );
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let dirty = (0..100)
        .map(|index| {
            AutosaveDocumentState::from_dirty_for_test(
                document_id(&format!("scene_{index:03}")),
                true,
            )
        })
        .collect::<Vec<_>>();
    let source = Arc::new(CountingSnapshotSource::success());
    let save_mutex = MutexGroup::parse("save_scene_bytes").unwrap();
    let materialized_requests = AtomicUsize::new(0);

    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 2,
                |document| {
                    materialized_requests.fetch_add(1, Ordering::AcqRel);
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source.clone(),
                    ))
                },
            )
            .unwrap()
    );
    assert_eq!(materialized_requests.load(Ordering::Acquire), 1);
    assert_eq!(
        wait_for_autosave_completion(&mut adapter, Duration::from_secs(11)).succeeded(),
        1
    );
    assert_eq!(source.capture_count(), 1);
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_skips_an_oversized_document_without_starving_later_work() {
    let root = temporary_root("adapter-oversized-fairness");
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_admission_limits(EditorJobAdmissionLimits::new(
            3,
            2,
            Duration::from_secs(10),
        )),
    );
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let dirty = [
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_oversized"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_small_a"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_small_b"), true),
    ];
    let source = Arc::new(CountingSnapshotSource::success());
    let save_mutex = MutexGroup::parse("save_scene_bytes").unwrap();
    let mut materialized = Vec::new();

    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |document| {
                    if document.as_str() == "scene_oversized" {
                        3
                    } else {
                        1
                    }
                },
                |document| {
                    materialized.push(document.as_str().to_string());
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source.clone(),
                    ))
                },
            )
            .unwrap()
    );
    assert_eq!(materialized, ["scene_small_a", "scene_small_b"]);
    assert_eq!(
        wait_for_autosave_completion(&mut adapter, Duration::from_secs(11)).succeeded(),
        2
    );
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_rotates_to_a_fitting_document_skipped_by_a_mixed_byte_window() {
    let root = temporary_root("adapter-mixed-byte-fairness");
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_admission_limits(EditorJobAdmissionLimits::new(
            3,
            3,
            Duration::from_secs(10),
        )),
    );
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let dirty = [
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_a"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_b"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_c"), true),
    ];
    let source = Arc::new(CountingSnapshotSource::success());
    let save_mutex = MutexGroup::parse("save_scene_mixed_bytes").unwrap();
    let mut materialized = Vec::new();

    for now in [Duration::from_secs(10), Duration::from_secs(21)] {
        assert!(
            adapter
                .schedule(
                    now,
                    &dirty,
                    |document| match document.as_str() {
                        "scene_a" | "scene_b" => 2,
                        "scene_c" => 1,
                        _ => unreachable!(),
                    },
                    |document| {
                        materialized.push(document.as_str().to_string());
                        Some(AutosaveDocumentRequest::new(
                            document.clone(),
                            AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                            source.clone(),
                        ))
                    },
                )
                .unwrap()
        );
        assert_eq!(
            wait_for_autosave_completion(&mut adapter, now).succeeded(),
            2
        );
    }

    assert_eq!(materialized, ["scene_a", "scene_c", "scene_b", "scene_c"]);
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_does_not_let_an_oversized_document_hide_a_temporary_skip() {
    let root = temporary_root("adapter-combined-byte-fairness");
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default().with_admission_limits(EditorJobAdmissionLimits::new(
            4,
            3,
            Duration::from_secs(10),
        )),
    );
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let dirty = [
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_a"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_b"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_c"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_d"), true),
    ];
    let source = Arc::new(CountingSnapshotSource::success());
    let save_mutex = MutexGroup::parse("save_scene_combined_bytes").unwrap();
    let mut materialized = Vec::new();

    for now in [Duration::from_secs(10), Duration::from_secs(21)] {
        assert!(
            adapter
                .schedule(
                    now,
                    &dirty,
                    |document| match document.as_str() {
                        "scene_a" => 4,
                        "scene_b" | "scene_c" => 2,
                        "scene_d" => 1,
                        _ => unreachable!(),
                    },
                    |document| {
                        materialized.push(document.as_str().to_string());
                        Some(AutosaveDocumentRequest::new(
                            document.clone(),
                            AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                            source.clone(),
                        ))
                    },
                )
                .unwrap()
        );
        assert_eq!(
            wait_for_autosave_completion(&mut adapter, now).succeeded(),
            2
        );
    }

    assert_eq!(materialized, ["scene_b", "scene_d", "scene_c", "scene_d"]);
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_rejects_an_over_age_backlog_before_estimating_requests() {
    let root = temporary_root("adapter-age-window");
    let jobs = test_job_system_with_limits(
        EditorJobLimits::default()
            .with_limit(JobCategory::Import, 1)
            .with_admission_limits(EditorJobAdmissionLimits::new(8, 1024, Duration::ZERO)),
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
            EditorJobSpec::new("queued-save", JobCategory::Import),
            GateJob::new(pending_started_sender, pending_release_receiver),
        )
        .unwrap();

    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let mut adapter = AutosaveJobAdapter::new(
        jobs.clone(),
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let estimates = AtomicUsize::new(0);
    let requests = AtomicUsize::new(0);
    assert!(matches!(
        adapter.schedule(
            Duration::from_secs(10),
            &dirty,
            |_| {
                estimates.fetch_add(1, Ordering::AcqRel);
                1
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
            crate::core::jobs::JobSubmitError::OldestPendingAgeExceeded { max_age_ms: 0 }
        ))
    ));
    assert_eq!(estimates.load(Ordering::Acquire), 0);
    assert_eq!(requests.load(Ordering::Acquire), 0);
    assert!(!adapter.is_in_flight());

    assert!(jobs.cancel(pending.id()));
    assert_eq!(pending.wait(), Err(JobError::Cancelled));
    release_sender.send(()).unwrap();
    assert_eq!(foreground.wait(), Ok(()));
    remove_temporary_root(&root);
}

#[test]
fn autosave_adapter_rejects_shared_job_shutdown_before_materializing_requests() {
    let root = temporary_root("adapter-shared-shutdown");
    let jobs = test_job_system_with_limits(EditorJobLimits::default());
    let _ = jobs.shutdown(Instant::now());
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document_id("scene_main"),
        true,
    )];
    let estimates = AtomicUsize::new(0);
    let requests = AtomicUsize::new(0);

    assert!(matches!(
        adapter.schedule(
            Duration::from_secs(10),
            &dirty,
            |_| {
                estimates.fetch_add(1, Ordering::AcqRel);
                1
            },
            |_| {
                requests.fetch_add(1, Ordering::AcqRel);
                None
            },
        ),
        Err(AutosaveAdmissionError::JobSubmit(
            crate::core::jobs::JobSubmitError::ShuttingDown
        ))
    ));
    assert_eq!(estimates.load(Ordering::Acquire), 0);
    assert_eq!(requests.load(Ordering::Acquire), 0);
    assert!(!adapter.is_in_flight());
    remove_temporary_root(&root);
}
