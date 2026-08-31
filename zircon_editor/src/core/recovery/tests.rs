use std::collections::BTreeSet;
use std::fs;
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionLifecycleV1;
use zircon_runtime_interface::project::session_lock::ProjectSessionPrincipalV1;
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use super::{
    AutosaveDocumentId, AutosaveDocumentState, AutosaveExtension, AutosaveJobPolicy,
    AutosavePolicy, AutosaveScheduler, AutosaveSnapshotProvenance, AutosaveSourceDigest,
    AutosaveSourcePath, AutosaveStore, RestoreAction, RestoreCandidate, RestoreFlow,
    RestoreFlowError, RestoreFreshness, RestoreResolution, RestoreStartup, SessionAdmissionRequest,
    SessionGuard, SessionGuardAdmission, SessionGuardError, SessionLockDurability,
    SessionLockInspection,
};
use crate::core::jobs::{JobCategory, JobPriority, MutexGroup};

mod autosave_adapter;
mod session_guard;

fn document_id(value: &str) -> AutosaveDocumentId {
    AutosaveDocumentId::parse(value).expect("test document id should be valid")
}

fn extension(value: &str) -> AutosaveExtension {
    AutosaveExtension::parse(value).expect("test extension should be valid")
}

fn recovery_source_path(value: &str) -> AutosaveSourcePath {
    AutosaveSourcePath::parse(value).expect("test recovery source path should be valid")
}

fn autosave_snapshot_provenance() -> AutosaveSnapshotProvenance {
    AutosaveSnapshotProvenance::capture(0, AutosaveSourceDigest::missing())
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

fn test_session_admission() -> SessionAdmissionRequest {
    let operation = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fixture operation");
    SessionAdmissionRequest::new(
        operation,
        ProjectSessionPrincipalV1::Welcome,
        ZrRuntimeBuildSetId::parse(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("fixture BuildSet"),
    )
}

fn claim_acquired(
    project_root: impl AsRef<std::path::Path>,
    now: std::time::SystemTime,
) -> SessionGuard {
    let admission = test_session_admission();
    match SessionGuard::claim_at(project_root, &admission, now).expect("claim session guard") {
        SessionGuardAdmission::Acquired(guard) => guard,
        SessionGuardAdmission::Active { .. } | SessionGuardAdmission::Residual(_) => {
            panic!("fresh test project root must acquire its session guard")
        }
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
    let source = include_str!("autosave/policy.rs");

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
            &recovery_source_path("scenes/main.zscene"),
            1,
            &extension("zscene"),
            &autosave_snapshot_provenance(),
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
                &recovery_source_path("scenes/world_01.zscene"),
                sequence,
                &zscene_extension,
                &autosave_snapshot_provenance(),
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
            "2.snapshot.json".to_string(),
            "3.zscene".to_string(),
            "3.snapshot.json".to_string(),
            "4.zscene".to_string(),
            "4.snapshot.json".to_string(),
            "recovery.json".to_string(),
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
        .write_snapshot(
            &document,
            &recovery_source_path("scenes/main.zscene"),
            1,
            &zscene_extension,
            &autosave_snapshot_provenance(),
            b"first snapshot",
        )
        .unwrap();
    assert!(
        store
            .write_snapshot(
                &document,
                &recovery_source_path("scenes/main.zscene"),
                1,
                &zscene_extension,
                &autosave_snapshot_provenance(),
                b"replacement snapshot",
            )
            .is_err()
    );
    assert!(
        store
            .write_snapshot(
                &document,
                &recovery_source_path("scenes/main.zscene"),
                1,
                &backup_extension,
                &autosave_snapshot_provenance(),
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
            &recovery_source_path("scenes/main.zscene"),
            1,
            &extension("zscene"),
            &autosave_snapshot_provenance(),
            b"zscene snapshot",
        )
    });
    let second_store = store;
    let second_start = Arc::clone(&start);
    let second = thread::spawn(move || {
        second_start.wait();
        second_store.write_snapshot(
            &document_id("scene_main"),
            &recovery_source_path("scenes/main.zscene"),
            1,
            &extension("backup"),
            &autosave_snapshot_provenance(),
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
    let snapshot_files = snapshots
        .iter()
        .filter(|name| {
            !name.ends_with(".snapshot.json")
                && name
                    .split_once('.')
                    .and_then(|(sequence, _)| sequence.parse::<u64>().ok())
                    .is_some()
        })
        .collect::<Vec<_>>();
    assert_eq!(snapshot_files.len(), 1);
    assert!(snapshots.iter().all(|name| !name.starts_with('.')));
    remove_temporary_root(&root);
}

#[test]
fn autosave_sequence_reservation_rejects_same_sequence_from_independent_stores() {
    let root = temporary_root("independent-store-sequence");
    let start = Arc::new(Barrier::new(2));
    let first_store = AutosaveStore::new(&root);
    let first_start = Arc::clone(&start);
    let first = thread::spawn(move || {
        first_start.wait();
        first_store.write_snapshot(
            &document_id("scene_main"),
            &recovery_source_path("scenes/main.zscene"),
            1,
            &extension("zscene"),
            &autosave_snapshot_provenance(),
            b"zscene snapshot",
        )
    });
    let second_store = AutosaveStore::new(&root);
    let second_start = Arc::clone(&start);
    let second = thread::spawn(move || {
        second_start.wait();
        second_store.write_snapshot(
            &document_id("scene_main"),
            &recovery_source_path("scenes/main.zscene"),
            1,
            &extension("backup"),
            &autosave_snapshot_provenance(),
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
        .filter(|name| {
            !name.ends_with(".snapshot.json")
                && name
                    .split_once('.')
                    .and_then(|(sequence, _)| sequence.parse::<u64>().ok())
                    .is_some()
        })
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 1);
    remove_temporary_root(&root);
}

#[test]
fn autosave_stale_sequence_marker_blocks_reuse_but_not_recovery_discovery() {
    let root = temporary_root("stale-sequence-marker");
    let store = AutosaveStore::new(&root);
    let document = document_id("scene_main");
    let source_path = recovery_source_path("scenes/main.zscene");
    let snapshot = store
        .write_snapshot(
            &document,
            &source_path,
            1,
            &extension("zscene"),
            &autosave_snapshot_provenance(),
            b"first snapshot",
        )
        .unwrap();
    let directory = snapshot.parent().unwrap();
    let stale_marker = directory.join(".2.autosave-reservation");
    fs::write(&stale_marker, b"interrupted writer").unwrap();

    assert_eq!(store.next_sequence(&document, 1).unwrap(), 3);

    assert!(matches!(
        store.write_snapshot(
            &document,
            &source_path,
            2,
            &extension("zscene"),
            &autosave_snapshot_provenance(),
            b"second snapshot",
        ),
        Err(super::AutosaveError::SnapshotSequenceUnavailable { sequence: 2, .. })
    ));

    let report = store.recovery_catalog().unwrap();
    let candidates = report.candidates();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].autosave_path(), snapshot.as_path());
    assert!(stale_marker.exists());
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
fn restore_flow_requires_a_residual_lock_and_one_explicit_action_per_document() {
    let root = temporary_root("restore-flow");
    let start = std::time::UNIX_EPOCH + Duration::from_secs(20);
    let mut guard = claim_acquired(&root, start);
    let lock = SessionGuard::inspect(&root).unwrap();
    let newer = RestoreCandidate::new(
        document_id("scene_main"),
        root.join("scene.zscene"),
        root.join(".zircon/autosave/scene_main/3.zscene"),
        RestoreFreshness::SnapshotAheadOfSource,
    );
    let older = RestoreCandidate::new(
        document_id("scene_old"),
        root.join("old.zscene"),
        root.join(".zircon/autosave/scene_old/2.zscene"),
        RestoreFreshness::SnapshotAlreadyCommitted,
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
    let guard = claim_acquired(&root, start);
    let lock = SessionGuard::inspect(&root).unwrap();
    drop(guard);
    let candidate = RestoreCandidate::new(
        document_id("scene_main"),
        root.join("scene.zscene"),
        root.join(".zircon/autosave/scene_main/3.zscene"),
        RestoreFreshness::SnapshotAheadOfSource,
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
    let guard = claim_acquired(&root, start);
    let expected = guard.record().clone();
    let lock = SessionGuard::inspect(&root).unwrap();
    drop(guard);
    let candidate = RestoreCandidate::new(
        document_id("scene_main"),
        root.join("scene.zscene"),
        root.join(".zircon/autosave/scene_main/3.zscene"),
        RestoreFreshness::SnapshotAlreadyCommitted,
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

fn temporary_root(label: &str) -> std::path::PathBuf {
    std::env::current_dir()
        .expect("current directory should be available")
        .join("target")
        .join(format!(
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
