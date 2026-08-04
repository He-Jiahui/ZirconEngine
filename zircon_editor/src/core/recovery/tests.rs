use std::collections::BTreeSet;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use super::{
    AutosaveAdmissionError, AutosaveDocumentId, AutosaveDocumentRequest, AutosaveDocumentState,
    AutosaveExtension, AutosaveJobAdapter, AutosaveJobPolicy, AutosavePolicy, AutosaveScheduler,
    AutosaveSnapshot, AutosaveSnapshotSource, AutosaveStore, RestoreAction, RestoreCandidate,
    RestoreFlow, RestoreFlowError, RestoreResolution, RestoreStartup, SessionGuard,
    SessionGuardError, SessionLockDurability, SessionLockInspection,
};
use crate::core::jobs::{
    EditorJob, EditorJobAdmissionLimits, EditorJobLimits, EditorJobSpec, JobCategory, JobContext,
    JobError, JobPriority, MutexGroup, test_job_system_with_limits,
};

fn document_id(value: &str) -> AutosaveDocumentId {
    AutosaveDocumentId::parse(value).expect("test document id should be valid")
}

fn extension(value: &str) -> AutosaveExtension {
    AutosaveExtension::parse(value).expect("test extension should be valid")
}

fn expected_lock_durability() -> SessionLockDurability {
    #[cfg(windows)]
    {
        SessionLockDurability::PublishedWithDurabilityUncertainty
    }
    #[cfg(not(windows))]
    {
        SessionLockDurability::Published
    }
}

#[test]
fn autosave_requires_elapsed_interval_and_a_dirty_document() {
    let policy = AutosavePolicy::new(Duration::from_secs(10)).unwrap();
    let mut scheduler = AutosaveScheduler::new(policy);
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let clean = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        false,
    )];

    assert!(scheduler.plan(Duration::from_secs(9), &dirty).is_none());
    assert!(scheduler.plan(Duration::from_secs(10), &clean).is_none());

    let plan = scheduler
        .plan(Duration::from_secs(10), &dirty)
        .expect("a due dirty document must create an autosave plan");
    assert_eq!(plan.documents(), &[document.clone()]);
    assert!(scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(11), &dirty).is_none());

    scheduler.mark_finished(Duration::from_secs(10));
    assert!(!scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(19), &dirty).is_none());
    assert!(scheduler.plan(Duration::from_secs(20), &dirty).is_some());
}

#[test]
fn autosave_execution_failure_releases_the_in_flight_plan_on_the_normal_interval() {
    let policy = AutosavePolicy::new(Duration::from_secs(10)).unwrap();
    let mut scheduler = AutosaveScheduler::new(policy);
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document_id("scene_main"),
        true,
    )];

    assert!(scheduler.plan(Duration::from_secs(10), &dirty).is_some());
    scheduler.mark_finished(Duration::from_secs(12));
    assert!(!scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(21), &dirty).is_none());
    assert!(scheduler.plan(Duration::from_secs(22), &dirty).is_some());
}

#[test]
fn autosave_submission_failure_releases_the_in_flight_plan_for_retry() {
    let policy = AutosavePolicy::new(Duration::from_secs(10)).unwrap();
    let mut scheduler = AutosaveScheduler::new(policy);
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document_id("scene_main"),
        true,
    )];

    assert!(scheduler.plan(Duration::from_secs(10), &dirty).is_some());
    assert!(scheduler.is_in_flight());
    scheduler.mark_submission_failed();
    assert!(!scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(10), &dirty).is_some());
}

#[test]
fn autosave_dirty_inputs_are_projected_from_editor03_history_state() {
    let source = include_str!("autosave.rs");

    assert!(source.contains("pub fn from_history_dirty"));
    assert!(source.contains("HistoryDirtyState"));
    assert!(!source.contains("pub fn dirty("));
    assert!(!source.contains("pub fn clean("));
}

#[test]
fn autosave_job_policy_uses_background_misc_and_the_save_mutex_group() {
    let save_mutex = MutexGroup::parse("save_scene_main").unwrap();
    let policy = AutosaveJobPolicy::for_save_mutex(save_mutex);

    assert_eq!(policy.category(), JobCategory::Misc);
    assert_eq!(policy.priority(), JobPriority::Background);
    assert_eq!(policy.save_mutex_group().as_str(), "save_scene_main");

    let _job_spec = policy.build_job_spec(&document_id("scene_main"));
}

#[test]
fn autosave_writes_only_the_project_autosave_tree_and_preserves_source_bytes() {
    let root = temporary_root("source-guard");
    let source_path = root.join("scenes").join("main.zscene");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, b"authoritative scene source").unwrap();
    let before = fs::read(&source_path).unwrap();
    let store = AutosaveStore::new(&root);

    let snapshot = store
        .write_snapshot(
            &document_id("scene_main"),
            1,
            &extension("zscene"),
            b"autosave snapshot",
        )
        .unwrap();

    assert_eq!(fs::read(&source_path).unwrap(), before);
    assert_eq!(fs::read(&snapshot).unwrap(), b"autosave snapshot");
    assert_eq!(
        snapshot,
        root.join(".zircon")
            .join("autosave")
            .join("scene_main")
            .join("1.zscene")
    );
    remove_temporary_root(&root);
}

#[test]
fn autosave_rotates_each_document_to_the_latest_three_sequences() {
    let root = temporary_root("rotation");
    let store = AutosaveStore::new(&root);
    let document = document_id("world_01");
    let zscene_extension = extension("zscene");

    for sequence in 1..=4 {
        store
            .write_snapshot(
                &document,
                sequence,
                &zscene_extension,
                format!("snapshot-{sequence}").as_bytes(),
            )
            .unwrap();
    }

    let directory = root.join(".zircon").join("autosave").join("world_01");
    let names = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        BTreeSet::from([
            "2.zscene".to_string(),
            "3.zscene".to_string(),
            "4.zscene".to_string(),
        ])
    );
    remove_temporary_root(&root);
}

#[test]
fn autosave_rejects_reusing_an_existing_snapshot_sequence() {
    let root = temporary_root("duplicate-sequence");
    let store = AutosaveStore::new(&root);
    let document = document_id("scene_main");
    let zscene_extension = extension("zscene");
    let backup_extension = extension("backup");

    store
        .write_snapshot(&document, 1, &zscene_extension, b"first snapshot")
        .unwrap();
    assert!(
        store
            .write_snapshot(&document, 1, &zscene_extension, b"replacement snapshot")
            .is_err()
    );
    assert!(
        store
            .write_snapshot(
                &document,
                1,
                &backup_extension,
                b"different-extension snapshot",
            )
            .is_err()
    );
    assert_eq!(
        fs::read(root.join(".zircon/autosave/scene_main/1.zscene")).unwrap(),
        b"first snapshot"
    );
    remove_temporary_root(&root);
}

#[test]
fn autosave_sequence_reservation_allows_only_one_concurrent_extension() {
    let root = temporary_root("concurrent-sequence");
    let store = AutosaveStore::new(&root);
    let start = Arc::new(Barrier::new(2));
    let first_store = store.clone();
    let first_start = Arc::clone(&start);
    let first = thread::spawn(move || {
        first_start.wait();
        first_store.write_snapshot(
            &document_id("scene_main"),
            1,
            &extension("zscene"),
            b"zscene snapshot",
        )
    });
    let second_store = store;
    let second_start = Arc::clone(&start);
    let second = thread::spawn(move || {
        second_start.wait();
        second_store.write_snapshot(
            &document_id("scene_main"),
            1,
            &extension("backup"),
            b"backup snapshot",
        )
    });

    let results = [first.join().unwrap(), second.join().unwrap()];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);

    let snapshots = fs::read_dir(root.join(".zircon/autosave/scene_main"))
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<BTreeSet<_>>();
    assert_eq!(snapshots.len(), 1);
    assert!(snapshots.iter().all(|name| !name.starts_with('.')));
    remove_temporary_root(&root);
}

#[test]
fn autosave_rejects_path_traversal_identifiers_and_extensions() {
    assert!(AutosaveDocumentId::parse("../source").is_err());
    assert!(AutosaveDocumentId::parse("scene/main").is_err());
    assert!(AutosaveDocumentId::parse(" ").is_err());
    assert!(AutosaveExtension::parse("../source").is_err());
    assert!(AutosaveExtension::parse("scene.data").is_err());
    assert!(AutosaveExtension::parse(" ").is_err());
}

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
                [AutosaveDocumentRequest::new(
                    document,
                    AutosaveJobPolicy::for_save_mutex(save_mutex),
                    source.clone(),
                )
                .with_estimated_pending_bytes(32)],
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
            [AutosaveDocumentRequest::new(
                document,
                AutosaveJobPolicy::for_save_mutex(MutexGroup::parse("save_scene_main").unwrap()),
                source.clone(),
            )],
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
                [AutosaveDocumentRequest::new(
                    document.clone(),
                    AutosaveJobPolicy::for_save_mutex(
                        MutexGroup::parse("save_scene_main").unwrap()
                    ),
                    Arc::new(CountingSnapshotSource::failure()),
                )],
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
            [AutosaveDocumentRequest::new(
                document,
                AutosaveJobPolicy::for_save_mutex(MutexGroup::parse("save_scene_main").unwrap()),
                Arc::new(CountingSnapshotSource::success()),
            )],
        ),
        Err(AutosaveAdmissionError::ShuttingDown)
    ));
    remove_temporary_root(&root);
}

#[test]
fn session_guard_persists_heartbeat_and_requires_explicit_residual_takeover() {
    let root = temporary_root("session-guard");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let mut guard = SessionGuard::acquire_at(&root, start).unwrap();
    let initial = guard.record().clone();
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == initial
    ));
    assert!(matches!(
        SessionGuard::acquire_at(&root, start),
        Err(SessionGuardError::AlreadyHeld { .. })
    ));

    assert_eq!(
        guard
            .refresh_heartbeat_at(start + Duration::from_secs(1))
            .unwrap(),
        expected_lock_durability()
    );
    assert_eq!(guard.record().heartbeat_unix_millis(), 11_000);
    assert_eq!(guard.release().unwrap(), expected_lock_durability());
    assert!(guard.is_released());
    assert_eq!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Missing
    );
    remove_temporary_root(&root);
}

#[test]
fn residual_takeover_failure_keeps_the_selected_lock() {
    let root = temporary_root("session-guard-takeover-failure");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let guard = SessionGuard::acquire_at(&root, start).unwrap();
    let expected = guard.record().clone();
    drop(guard);

    assert!(matches!(
        SessionGuard::replace_residual_at(
            &root,
            &expected,
            std::time::UNIX_EPOCH - Duration::from_secs(1),
        ),
        Err(SessionGuardError::ClockBeforeUnixEpoch)
    ));
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == expected
    ));
    remove_temporary_root(&root);
}

#[test]
fn residual_takeover_persists_a_new_guard_record() {
    let root = temporary_root("session-guard-takeover-success");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let guard = SessionGuard::acquire_at(&root, start).unwrap();
    let expected = guard.record().clone();
    drop(guard);

    let mut replacement =
        SessionGuard::replace_residual_at(&root, &expected, start + Duration::from_secs(1))
            .unwrap();
    let replacement_record = replacement.record().clone();
    assert_ne!(replacement_record, expected);
    assert!(matches!(
        SessionGuard::inspect(&root).unwrap(),
        SessionLockInspection::Residual(record) if record == replacement_record
    ));

    assert_eq!(replacement.release().unwrap(), expected_lock_durability());
    remove_temporary_root(&root);
}

#[test]
fn concurrent_residual_takeover_keeps_exactly_one_live_guard() {
    let root = temporary_root("session-guard-concurrent-takeover");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let guard = SessionGuard::acquire_at(&root, start).unwrap();
    let expected = Arc::new(guard.record().clone());
    drop(guard);

    let start_takeover = Arc::new(Barrier::new(3));
    let hold_results = Arc::new(Barrier::new(3));
    let (sender, receiver) = mpsc::channel();
    let first_root = root.clone();
    let first_expected = Arc::clone(&expected);
    let first_start_takeover = Arc::clone(&start_takeover);
    let first_hold_results = Arc::clone(&hold_results);
    let first_sender = sender.clone();
    let first = thread::spawn(move || {
        first_start_takeover.wait();
        let result = SessionGuard::replace_residual_at(
            &first_root,
            &first_expected,
            start + Duration::from_secs(1),
        );
        first_sender.send(result.is_ok()).unwrap();
        first_hold_results.wait();
    });
    let second_root = root.clone();
    let second_expected = Arc::clone(&expected);
    let second_start_takeover = Arc::clone(&start_takeover);
    let second_hold_results = Arc::clone(&hold_results);
    let second = thread::spawn(move || {
        second_start_takeover.wait();
        let result = SessionGuard::replace_residual_at(
            &second_root,
            &second_expected,
            start + Duration::from_secs(2),
        );
        sender.send(result.is_ok()).unwrap();
        second_hold_results.wait();
    });

    start_takeover.wait();
    let successes = [receiver.recv().unwrap(), receiver.recv().unwrap()]
        .into_iter()
        .filter(|success| *success)
        .count();
    assert_eq!(successes, 1);
    hold_results.wait();
    first.join().unwrap();
    second.join().unwrap();
    remove_temporary_root(&root);
}

#[test]
fn live_guard_rejects_takeover_before_heartbeat_and_release() {
    let root = temporary_root("session-guard-live-owner");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(10);
    let residual = SessionGuard::acquire_at(&root, start).unwrap();
    let expected = residual.record().clone();
    drop(residual);

    let mut guard =
        SessionGuard::replace_residual_at(&root, &expected, start + Duration::from_secs(1))
            .unwrap();
    let successor = guard.record().clone();
    assert!(matches!(
        SessionGuard::replace_residual_at(&root, &expected, start + Duration::from_secs(2)),
        Err(SessionGuardError::AlreadyHeld { .. })
    ));
    assert_eq!(
        guard
            .refresh_heartbeat_at(start + Duration::from_secs(3))
            .unwrap(),
        expected_lock_durability()
    );
    assert_eq!(guard.release().unwrap(), expected_lock_durability());
    assert!(matches!(
        SessionGuard::replace_residual_at(&root, &successor, start + Duration::from_secs(4)),
        Err(SessionGuardError::OwnershipLost { .. })
    ));
    remove_temporary_root(&root);
}

#[test]
fn session_guard_rejects_duplicate_persisted_record_fields() {
    let root = temporary_root("session-guard-duplicate-fields");
    let directory = root.join(".zircon");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("session.lock"),
        "version=1\nprocess_id=1\nprocess_id=2\ninstance_id=1-10-1\nheartbeat_unix_millis=10000\n",
    )
    .unwrap();

    assert!(matches!(
        SessionGuard::inspect(&root),
        Err(SessionGuardError::InvalidRecord { .. })
    ));
    remove_temporary_root(&root);
}

#[cfg(windows)]
#[test]
fn windows_session_guard_lease_uses_the_cross_session_namespace() {
    let root = temporary_root("session-guard-global-namespace");
    let name = super::session_guard::session_mutex_name_for_test(&root);

    assert!(name.starts_with("Global\\ZirconEngineProjectSession-"));
}

#[test]
fn restore_flow_requires_a_residual_lock_and_one_explicit_action_per_document() {
    let root = temporary_root("restore-flow");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(20);
    let mut guard = SessionGuard::acquire_at(&root, start).unwrap();
    let lock = SessionGuard::inspect(&root).unwrap();
    let newer = RestoreCandidate::new(
        document_id("scene_main"),
        root.join("scene.zscene"),
        root.join(".zircon/autosave/scene_main/3.zscene"),
        Some(start),
        start + Duration::from_secs(1),
    );
    let older = RestoreCandidate::new(
        document_id("scene_old"),
        root.join("old.zscene"),
        root.join(".zircon/autosave/scene_old/2.zscene"),
        Some(start + Duration::from_secs(2)),
        start + Duration::from_secs(1),
    );
    let startup = RestoreFlow::detect(lock, [newer.clone(), older]).unwrap();
    assert!(matches!(
        startup,
        RestoreStartup::RecoveryRequired { ref candidates, .. }
            if candidates == &vec![newer.clone()]
    ));
    assert!(matches!(
        RestoreFlow::plan(&startup, std::iter::empty::<RestoreResolution>()),
        Err(RestoreFlowError::MissingResolution { .. })
    ));
    let plan = RestoreFlow::plan(
        &startup,
        [RestoreResolution::new(
            document_id("scene_main"),
            RestoreAction::OpenComparison,
        )],
    )
    .unwrap();
    assert_eq!(plan.resolutions().len(), 1);
    assert_eq!(
        plan.resolutions()[0].action(),
        RestoreAction::OpenComparison
    );
    assert_eq!(
        RestoreFlow::detect(SessionLockInspection::Missing, [newer]).unwrap(),
        RestoreStartup::NoRecoveryNeeded
    );
    assert_eq!(guard.release().unwrap(), expected_lock_durability());
    remove_temporary_root(&root);
}

#[test]
fn restore_flow_rejects_duplicate_and_unexpected_document_choices() {
    let root = temporary_root("restore-flow-invalid-choices");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(20);
    let guard = SessionGuard::acquire_at(&root, start).unwrap();
    let lock = SessionGuard::inspect(&root).unwrap();
    drop(guard);
    let candidate = RestoreCandidate::new(
        document_id("scene_main"),
        root.join("scene.zscene"),
        root.join(".zircon/autosave/scene_main/3.zscene"),
        Some(start),
        start + Duration::from_secs(1),
    );

    assert!(matches!(
        RestoreFlow::detect(lock.clone(), [candidate.clone(), candidate.clone()]),
        Err(RestoreFlowError::DuplicateCandidate { .. })
    ));

    let startup = RestoreFlow::detect(lock, [candidate]).unwrap();
    let document = document_id("scene_main");
    assert!(matches!(
        RestoreFlow::plan(
            &startup,
            [
                RestoreResolution::new(document.clone(), RestoreAction::RestoreAutosave),
                RestoreResolution::new(document.clone(), RestoreAction::DiscardAutosave),
            ],
        ),
        Err(RestoreFlowError::DuplicateResolution { .. })
    ));
    assert!(matches!(
        RestoreFlow::plan(
            &startup,
            [
                RestoreResolution::new(document, RestoreAction::RestoreAutosave),
                RestoreResolution::new(document_id("unexpected"), RestoreAction::OpenComparison,),
            ],
        ),
        Err(RestoreFlowError::UnexpectedResolution { .. })
    ));
    remove_temporary_root(&root);
}

#[test]
fn restore_flow_preserves_residual_takeover_without_candidates() {
    let root = temporary_root("restore-flow-residual-takeover");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(20);
    let guard = SessionGuard::acquire_at(&root, start).unwrap();
    let expected = guard.record().clone();
    let lock = SessionGuard::inspect(&root).unwrap();
    drop(guard);
    let candidate = RestoreCandidate::new(
        document_id("scene_main"),
        root.join("scene.zscene"),
        root.join(".zircon/autosave/scene_main/3.zscene"),
        Some(start + Duration::from_secs(1)),
        start,
    );

    let startup = RestoreFlow::detect(lock, [candidate]).unwrap();
    assert!(matches!(
        startup,
        RestoreStartup::ResidualTakeoverRequired { ref residual_lock }
            if residual_lock == &expected
    ));
    assert_eq!(startup.residual_lock(), Some(&expected));
    assert!(RestoreFlow::plan(&startup, std::iter::empty::<RestoreResolution>()).is_ok());
    assert!(matches!(
        RestoreFlow::plan(
            &startup,
            [RestoreResolution::new(
                document_id("unexpected"),
                RestoreAction::DiscardAutosave,
            )],
        ),
        Err(RestoreFlowError::UnexpectedResolution { .. })
    ));
    remove_temporary_root(&root);
}

fn wait_for_autosave_completion(
    adapter: &mut AutosaveJobAdapter,
    now: Duration,
) -> super::AutosaveCompletion {
    for _ in 0..1_000 {
        let completion = adapter.pump_completed(now);
        if completion.pending() == 0 && completion.succeeded() + completion.failed() != 0 {
            return completion;
        }
        thread::yield_now();
    }
    panic!("autosave job did not reach a terminal result");
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
            b"autosave snapshot".to_vec(),
        ))
    }
}

fn temporary_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-autosave-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos()
    ))
}

fn remove_temporary_root(path: &std::path::Path) {
    let _ = fs::remove_dir_all(path);
}
