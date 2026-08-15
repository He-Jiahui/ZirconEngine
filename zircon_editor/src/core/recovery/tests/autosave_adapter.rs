use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::super::{
    AutosaveAdmissionError, AutosaveDocumentId, AutosaveDocumentRequest, AutosaveDocumentState,
    AutosaveJobAdapter, AutosaveJobPolicy, AutosavePolicy, AutosaveScheduler, AutosaveSnapshot,
    AutosaveSnapshotSource, AutosaveStore,
};
use super::{document_id, extension, recovery_source_path, remove_temporary_root, temporary_root};
use crate::core::jobs::{
    EditorJob, EditorJobAdmissionLimits, EditorJobLimits, EditorJobSpec, JobCategory, JobContext,
    JobError, MutexGroup, test_job_system_with_limits,
};

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

#[test]
fn autosave_completion_pump_inspects_only_the_explicit_ticket_budget() {
    let root = temporary_root("adapter-completion-budget");
    let jobs =
        test_job_system_with_limits(EditorJobLimits::resolved(16, []).with_admission_limits(
            EditorJobAdmissionLimits::new(100, 1_024, Duration::from_secs(10)),
        ));
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
    let save_mutex = MutexGroup::parse("save_completion_budget").unwrap();

    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |document| {
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source.clone(),
                    ))
                },
            )
            .unwrap()
    );

    let first = adapter.pump_completed_with_budget(Duration::from_secs(11), 8);
    assert_eq!(first.inspected_tickets(), 8);
    assert!(first.pending() != 0);
    let completion = wait_for_autosave_completion(&mut adapter, Duration::from_secs(11));
    assert_eq!(completion.succeeded(), 100);
    assert_eq!(completion.failed(), 0);
    remove_temporary_root(&root);
}

#[test]
fn autosave_completion_budget_preserves_zero_budget_and_rotates_a_blocked_head() {
    let root = temporary_root("adapter-completion-rotation");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(4, []));
    let blocked_mutex = MutexGroup::parse("save_blocked_head").unwrap();
    let ready_mutex = MutexGroup::parse("save_ready_tail").unwrap();
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("autosave-completion-blocker", JobCategory::Import)
                .with_mutex_group(blocked_mutex.clone()),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let blocked_source = Arc::new(CountingSnapshotSource::success());
    let ready_source = Arc::new(CountingSnapshotSource::success());
    let dirty = [
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_a"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_b"), true),
    ];
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |document| {
                    let (save_mutex, source) = if document.as_str() == "scene_a" {
                        (blocked_mutex.clone(), blocked_source.clone())
                    } else {
                        (ready_mutex.clone(), ready_source.clone())
                    };
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex),
                        source,
                    ))
                },
            )
            .unwrap()
    );
    wait_for_capture_count(&ready_source, 1);

    let zero = adapter.pump_completed_with_budget(Duration::from_secs(11), 0);
    assert_eq!(zero.inspected_tickets(), 0);
    assert_eq!(zero.pending(), 2);
    assert_eq!(zero.succeeded(), 0);

    let blocked = adapter.pump_completed_with_budget(Duration::from_secs(11), 1);
    assert_eq!(blocked.inspected_tickets(), 1);
    assert_eq!(blocked.pending(), 2);
    assert_eq!(blocked.succeeded(), 0);
    let ready = wait_for_autosave_completion_state(
        &mut adapter,
        Duration::from_secs(11),
        1,
        |completion| completion.pending() == 1 && completion.succeeded() == 1,
    );
    assert_eq!(ready.pending(), 1);
    assert_eq!(ready.succeeded(), 1);

    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
    let completion = wait_for_autosave_completion(&mut adapter, Duration::from_secs(11));
    assert_eq!(completion.succeeded(), 2);
    assert_eq!(completion.failed(), 0);
    remove_temporary_root(&root);
}

#[test]
fn autosave_completion_counts_accumulate_then_reset_for_the_next_interval() {
    let root = temporary_root("adapter-completion-reset");
    let jobs = test_job_system_with_limits(EditorJobLimits::resolved(4, []));
    let succeeded_source = Arc::new(CountingSnapshotSource::success());
    let failed_source = Arc::new(CountingSnapshotSource::failure());
    let save_mutex = MutexGroup::parse("save_completion_reset").unwrap();
    let dirty = [
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_a"), true),
        AutosaveDocumentState::from_dirty_for_test(document_id("scene_b"), true),
    ];
    let mut adapter = AutosaveJobAdapter::new(
        jobs,
        AutosaveStore::new(&root),
        AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
    );
    assert!(
        adapter
            .schedule(
                Duration::from_secs(10),
                &dirty,
                |_| 1,
                |document| {
                    let source: Arc<dyn AutosaveSnapshotSource> = if document.as_str() == "scene_a"
                    {
                        succeeded_source.clone()
                    } else {
                        failed_source.clone()
                    };
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        source,
                    ))
                },
            )
            .unwrap()
    );
    wait_for_capture_count(&succeeded_source, 1);
    wait_for_capture_count(&failed_source, 1);

    let first = wait_for_autosave_completion_state(
        &mut adapter,
        Duration::from_secs(11),
        1,
        |completion| completion.pending() == 1 && completion.succeeded() + completion.failed() == 1,
    );
    assert_eq!(first.pending(), 1);
    assert_eq!(first.succeeded() + first.failed(), 1);
    let terminal = wait_for_autosave_completion_state(
        &mut adapter,
        Duration::from_secs(11),
        1,
        |completion| {
            completion.pending() == 0 && completion.succeeded() == 1 && completion.failed() == 1
        },
    );
    assert_eq!(terminal.succeeded(), 1);
    assert_eq!(terminal.failed(), 1);
    assert_eq!(terminal.pending(), 0);
    assert!(!adapter.is_in_flight());

    let reset = adapter.pump_completed_with_budget(Duration::from_secs(11), 1);
    assert_eq!(reset.succeeded(), 0);
    assert_eq!(reset.failed(), 0);
    assert_eq!(reset.pending(), 0);
    assert!(
        adapter
            .schedule(
                Duration::from_secs(21),
                &dirty[..1],
                |_| 1,
                |document| {
                    Some(AutosaveDocumentRequest::new(
                        document.clone(),
                        AutosaveJobPolicy::for_save_mutex(save_mutex.clone()),
                        Arc::new(CountingSnapshotSource::success()),
                    ))
                },
            )
            .unwrap()
    );
    assert_eq!(
        wait_for_autosave_completion(&mut adapter, Duration::from_secs(22)).succeeded(),
        1
    );
    remove_temporary_root(&root);
}

fn wait_for_autosave_completion(
    adapter: &mut AutosaveJobAdapter,
    now: Duration,
) -> super::super::AutosaveCompletion {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let completion = adapter.pump_completed(now);
        if completion.pending() == 0 && completion.succeeded() + completion.failed() != 0 {
            return completion;
        }
        thread::sleep(Duration::from_millis(1));
    }
    panic!("autosave job did not reach a terminal result");
}

fn wait_for_autosave_completion_state(
    adapter: &mut AutosaveJobAdapter,
    now: Duration,
    budget: usize,
    target: impl Fn(super::super::AutosaveCompletion) -> bool,
) -> super::super::AutosaveCompletion {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let completion = adapter.pump_completed_with_budget(now, budget);
        assert!(completion.inspected_tickets() <= budget);
        if target(completion) {
            return completion;
        }
        assert!(
            Instant::now() < deadline,
            "autosave completion state did not reach the expected result"
        );
        thread::yield_now();
    }
}

fn wait_for_capture_count(source: &CountingSnapshotSource, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while source.capture_count() < expected {
        assert!(
            Instant::now() < deadline,
            "autosave source did not reach {expected} captures"
        );
        thread::yield_now();
    }
}

struct GateJob {
    started: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
}

impl GateJob {
    fn new(started: mpsc::Sender<()>, release: mpsc::Receiver<()>) -> Self {
        Self { started, release }
    }
}

impl EditorJob for GateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        self.release.recv().unwrap();
        Ok(())
    }
}

struct CountingSnapshotSource {
    captures: AtomicUsize,
    failure: bool,
}

impl CountingSnapshotSource {
    fn success() -> Self {
        Self {
            captures: AtomicUsize::new(0),
            failure: false,
        }
    }

    fn failure() -> Self {
        Self {
            captures: AtomicUsize::new(0),
            failure: true,
        }
    }

    fn capture_count(&self) -> usize {
        self.captures.load(Ordering::Acquire)
    }
}

impl AutosaveSnapshotSource for CountingSnapshotSource {
    fn capture(&self, _document: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError> {
        self.captures.fetch_add(1, Ordering::AcqRel);
        if self.failure {
            return Err(JobError::failed(std::io::Error::other("snapshot failure")));
        }
        Ok(AutosaveSnapshot::new(
            1,
            extension("zscene"),
            recovery_source_path("scenes/main.zscene"),
            b"autosave snapshot".to_vec(),
        ))
    }
}
