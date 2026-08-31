use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::super::atomic::archive_path_write_authority_contains;
use super::*;
use crate::asset::project::{ProjectPaths, ResolvedProjectPath, ResolvedProjectPathIdentity};
use crate::core::CoreRuntime;
use crate::core::runtime::{
    BoundedKeyedIoTerminal, BoundedKeyedIoWaitResult, TaskPool, TaskPoolDescriptor,
};
use crate::scene::{RuntimeSessionArchive, World};

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn rejected_write_does_not_supersede_the_last_admitted_path_generation() {
    let root = unique_workspace_temp_root("rejected_write_intent");
    let path = root.join("session.zrsession");
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (blocker_started_tx, blocker_started_rx) = mpsc::sync_channel(0);
    let (release_blocker_tx, release_blocker_rx) = mpsc::sync_channel(0);
    scheduler.spawn(move || {
        blocker_started_tx.send(()).unwrap();
        release_blocker_rx.recv().unwrap();
    });
    blocker_started_rx.recv().unwrap();
    let writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits {
            max_entries: 1,
            max_retained_bytes: 1024 * 1024,
        },
        scheduler,
    );
    let admitted = RuntimeSessionArchive::from_world("admitted", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let rejected = RuntimeSessionArchive::from_world("rejected", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let target = resolved(&path);

    let admitted_submission = writer
        .try_submit(admitted, target.clone(), BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    let rejected_submission =
        writer.try_submit(rejected, target, BoundedKeyedIoWorkDeadline::none());
    release_blocker_tx.send(()).unwrap();

    assert!(matches!(
        rejected_submission,
        Err(RuntimeSessionArchiveWriterSubmitError::Admission(
            BoundedKeyedIoAdmissionError::EntryCapacityExceeded
        ))
    ));
    assert_eq!(
        admitted_submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert!(matches!(admitted_submission.take_outcome(), Some(Ok(()))));
    assert!(
        RuntimeSessionArchive::load_from_path(&path)
            .unwrap()
            .contains_slot("admitted")
    );

    drop(writer.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejected_unique_paths_do_not_accumulate_write_authority_state() {
    let root = unique_workspace_temp_root("rejected_path_retirement");
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (blocker_started_tx, blocker_started_rx) = mpsc::sync_channel(0);
    let (release_blocker_tx, release_blocker_rx) = mpsc::sync_channel(0);
    let blocker = scheduler.spawn(move || {
        blocker_started_tx.send(()).unwrap();
        release_blocker_rx.recv().unwrap();
    });
    blocker_started_rx.recv().unwrap();
    let writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits {
            max_entries: 1,
            max_retained_bytes: 1024 * 1024,
        },
        scheduler,
    );
    let artifact = RuntimeSessionArchive::from_world("accepted", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let accepted_target = resolved(root.join("accepted.zrsession"));
    let accepted_identity = ResolvedProjectPathIdentity::from(accepted_target.clone());
    let accepted = writer
        .try_submit(
            artifact.clone(),
            accepted_target,
            BoundedKeyedIoWorkDeadline::none(),
        )
        .unwrap();
    assert!(archive_path_write_authority_contains(&accepted_identity));

    for index in 0..64 {
        let target = resolved(root.join(format!("rejected-{index}.zrsession")));
        let identity = ResolvedProjectPathIdentity::from(target.clone());
        let rejected =
            writer.try_submit(artifact.clone(), target, BoundedKeyedIoWorkDeadline::none());

        assert!(matches!(
            rejected,
            Err(RuntimeSessionArchiveWriterSubmitError::Admission(
                BoundedKeyedIoAdmissionError::EntryCapacityExceeded
            ))
        ));
        assert!(!archive_path_write_authority_contains(&identity));
    }

    release_blocker_tx.send(()).unwrap();
    blocker.wait();
    assert_eq!(
        accepted
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert!(matches!(accepted.take_outcome(), Some(Ok(()))));
    assert!(!archive_path_write_authority_contains(&accepted_identity));

    drop(writer.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn submission_owner_can_cancel_a_write_before_the_worker_starts() {
    let root = unique_workspace_temp_root("cancel_before_start");
    let path = root.join("session.zrsession");
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (blocker_started_tx, blocker_started_rx) = mpsc::sync_channel(0);
    let (release_blocker_tx, release_blocker_rx) = mpsc::sync_channel(0);
    let blocker = scheduler.spawn(move || {
        blocker_started_tx.send(()).unwrap();
        release_blocker_rx.recv().unwrap();
    });
    blocker_started_rx.recv().unwrap();
    let writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits {
            max_entries: 1,
            max_retained_bytes: 1024 * 1024,
        },
        scheduler,
    );
    let artifact = RuntimeSessionArchive::from_world("cancelled", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();

    let submission = writer
        .try_submit(
            artifact,
            resolved(&path),
            BoundedKeyedIoWorkDeadline::none(),
        )
        .unwrap();

    assert_eq!(submission.cancel_before_start(), Ok(()));
    assert_eq!(submission.cancel_before_start(), Ok(()));
    assert_eq!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::CancelledBeforeStart)
    );
    assert!(submission.take_outcome().is_none());

    release_blocker_tx.send(()).unwrap();
    blocker.wait();
    drop(writer.shutdown());

    assert!(submission.take_outcome().is_none());
    assert!(!path.exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn writer_rejects_submission_after_its_runtime_owner_expires() {
    let root = unique_workspace_temp_root("runtime_owner_expired");
    let path = root.join("session.zrsession");
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let writer = RuntimeSessionArchiveWriter::with_runtime(
        RuntimeSessionArchiveWriterLimits::default(),
        &core,
    );
    drop(core);
    drop(runtime);
    let artifact = RuntimeSessionArchive::from_world("expired", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();

    let submission = writer.try_submit(
        artifact,
        resolved(&path),
        BoundedKeyedIoWorkDeadline::none(),
    );

    assert!(matches!(
        submission,
        Err(RuntimeSessionArchiveWriterSubmitError::RuntimeUnavailable)
    ));
    assert!(!path.exists());

    drop(writer.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn queued_write_defers_parent_creation_to_the_io_worker() {
    let root = unique_workspace_temp_root("worker_owned_parent_creation");
    let path = root
        .join("nested")
        .join("archive")
        .join("session.zrsession");
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (blocker_started_tx, blocker_started_rx) = mpsc::sync_channel(0);
    let (release_blocker_tx, release_blocker_rx) = mpsc::sync_channel(0);
    let blocker = scheduler.spawn(move || {
        blocker_started_tx.send(()).unwrap();
        release_blocker_rx.recv().unwrap();
    });
    blocker_started_rx.recv().unwrap();
    let writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits {
            max_entries: 1,
            max_retained_bytes: 1024 * 1024,
        },
        scheduler,
    );
    let artifact = RuntimeSessionArchive::from_world("worker-owned", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let target = resolved(&path);

    let submission = writer
        .try_submit(artifact, target, BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert!(!root.join("nested").exists());
    release_blocker_tx.send(()).unwrap();
    blocker.wait();
    assert_eq!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert!(matches!(submission.take_outcome(), Some(Ok(()))));
    assert!(path.is_file());

    drop(writer.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn writer_has_no_process_global_constructor() {
    let source = include_str!("../writer.rs");

    for forbidden in ["TaskPools::process_default", "pub fn new("] {
        assert!(
            !source.contains(forbidden),
            "archive writer must not retain process fallback `{forbidden}`"
        );
    }
}

#[test]
fn writer_uses_the_shared_atomic_file_owner() {
    let atomic_source = include_str!("../atomic.rs");
    let support_source = include_str!("../support.rs");

    assert!(
        atomic_source.contains("stage_atomic_write(operation_path, artifact.serialized_bytes())")
    );
    assert!(atomic_source.contains("let commit_result = staged.commit();"));
    assert!(
        atomic_source.contains("file_matches_bytes(operation_path, artifact.serialized_bytes())")
    );
    assert!(atomic_source.contains("Weak<ArchivePathWriteState>"));
    assert!(
        !atomic_source
            .contains("Mutex<BTreeMap<ResolvedProjectPathIdentity, CommittedPathRevision>>")
    );
    let stage_offset = atomic_source
        .find("stage_atomic_write(operation_path, artifact.serialized_bytes())")
        .unwrap();
    let path_lock_offset = atomic_source
        .find("let mut committed = ticket.state.lock_revision();")
        .unwrap();
    assert!(stage_offset < path_lock_offset);
    for forbidden in [
        "File::create",
        "BufWriter",
        "prepare_existing_target_backup",
        "restore_existing_target_backup",
        "temporary_archive_path",
        "fs::rename",
    ] {
        assert!(
            !atomic_source.contains(forbidden),
            "archive writer must not retain private atomic-file behavior `{forbidden}`"
        );
    }
    assert!(!support_source.contains("temporary_archive_path"));
}

#[test]
fn writer_submission_consumes_a_prepared_path_without_caller_side_filesystem_work() {
    let source = include_str!("../writer.rs");
    let closure_offset = source.find("Box::new(move ||").unwrap();

    assert!(source.contains("target: ResolvedProjectPath"));
    for forbidden in [
        "std::fs",
        "canonicalize(",
        "ProjectPaths::resolve_path",
        "metadata(",
    ] {
        assert!(
            !source[..closure_offset].contains(forbidden),
            "writer admission must not perform caller-side filesystem work `{forbidden}`"
        );
    }
}

fn resolved(path: impl AsRef<Path>) -> ResolvedProjectPath {
    ProjectPaths::resolve_path(path).unwrap()
}

fn unique_workspace_temp_root(label: &str) -> PathBuf {
    let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::current_dir()
        .expect("dynamic scene tests require a workspace current directory")
        .join(".codex")
        .join("tmp")
        .join(format!(
            "runtime11_dynamic_scene_{label}_{}_{}",
            std::process::id(),
            id
        ));
    std::fs::create_dir_all(&root).unwrap();
    root
}
