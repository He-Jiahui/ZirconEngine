use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;
use crate::asset::project::{ProjectPaths, ResolvedProjectPath, ResolvedProjectPathIdentity};
use crate::core::CoreRuntime;
use crate::core::runtime::{
    BoundedKeyedIoTerminal, BoundedKeyedIoWaitResult, BoundedKeyedIoWorkDeadline, JobScheduler,
    RetainedByteBudgetError, TaskPool, TaskPoolDescriptor,
};
use crate::scene::{RuntimeSessionArchive, World};

const TEST_ARCHIVE_RESERVATION_BYTES: usize = 64 * 1024;

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn archive_reader_uses_the_project_path_authority() {
    let root = unique_workspace_temp_root("path");
    let prepared = resolved(root.join("nested").join("..").join("session.zrsession"));
    let direct = resolved(root.join("session.zrsession"));

    assert_eq!(prepared.operation_path(), direct.operation_path());
    assert_eq!(
        ResolvedProjectPathIdentity::from(prepared),
        ResolvedProjectPathIdentity::from(direct)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_path_submissions_share_one_runtime_ticket_and_one_result_reservation() {
    let root = unique_workspace_temp_root("single_flight");
    let path = root.join("session.zrsession");
    RuntimeSessionArchive::from_world("shared", &World::empty())
        .unwrap()
        .save_to_path_atomically(&path)
        .unwrap();
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
    let reader = RuntimeSessionArchiveReader::with_scheduler(test_limits(), scheduler);
    let direct_path = resolved(&path);
    let alias_path = resolved(root.join("uncreated").join("..").join("session.zrsession"));

    let first = reader
        .try_submit(direct_path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    let second = reader
        .try_submit(alias_path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert_eq!(first.ticket().id(), second.ticket().id());
    assert_eq!(
        reader.diagnostics().retained_result_bytes,
        TEST_ARCHIVE_RESERVATION_BYTES
    );

    release_blocker_tx.send(()).unwrap();
    blocker.wait();
    assert_eq!(
        first
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    let first_result = succeeded(first.outcome().unwrap());
    let second_result = succeeded(second.outcome().unwrap());
    assert!(first_result.archive().contains_slot("shared"));
    assert!(second_result.archive().contains_slot("shared"));

    drop(first);
    drop(second);
    drop(first_result);
    assert_eq!(
        reader.diagnostics().retained_result_bytes,
        TEST_ARCHIVE_RESERVATION_BYTES
    );
    drop(second_result);
    assert_eq!(reader.diagnostics().retained_result_bytes, 0);

    drop(reader.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn retained_result_budget_rejects_a_distinct_path_until_the_result_is_released() {
    let root = unique_workspace_temp_root("result_budget");
    let first_path = root.join("first.zrsession");
    let second_path = root.join("second.zrsession");
    for (slot, path) in [("first", &first_path), ("second", &second_path)] {
        RuntimeSessionArchive::from_world(slot, &World::empty())
            .unwrap()
            .save_to_path_atomically(path)
            .unwrap();
    }
    let reader = RuntimeSessionArchiveReader::with_scheduler(
        test_limits(),
        JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::io().with_worker_threads(1),
        )),
    );
    let first = reader
        .try_submit(resolved(&first_path), BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    assert_eq!(
        first
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    let result = succeeded(first.outcome().unwrap());

    let rejected = reader.try_submit(resolved(&second_path), BoundedKeyedIoWorkDeadline::none());

    assert!(matches!(
        rejected,
        Err(RuntimeSessionArchiveReaderSubmitError::ResultBytes(
            RetainedByteBudgetError::CapacityExceeded { .. }
        ))
    ));
    drop(first);
    drop(result);
    let accepted = reader
        .try_submit(resolved(second_path), BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    assert_eq!(
        accepted
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );

    drop(accepted);
    drop(reader.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cancelled_queued_read_releases_its_result_reservation_without_opening_the_path() {
    let root = unique_workspace_temp_root("cancelled");
    let path_buf = root.join("missing.zrsession");
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
    let reader = RuntimeSessionArchiveReader::with_scheduler(test_limits(), scheduler);
    let submission = reader
        .try_submit(resolved(&path_buf), BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert_eq!(submission.cancel_shared_before_start(), Ok(()));
    assert_eq!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::CancelledBeforeStart)
    );
    assert!(submission.outcome().is_none());
    assert_eq!(reader.diagnostics().retained_result_bytes, 0);
    assert!(!path_buf.exists());

    release_blocker_tx.send(()).unwrap();
    blocker.wait();
    drop(submission);
    drop(reader.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_read_releases_its_result_reservation_and_preserves_the_typed_error() {
    let root = unique_workspace_temp_root("failed");
    let path = resolved(root.join("missing.zrsession"));
    let reader = RuntimeSessionArchiveReader::with_scheduler(
        test_limits(),
        JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::io().with_worker_threads(1),
        )),
    );
    let submission = reader
        .try_submit(path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert!(matches!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(_))
    ));
    match submission.outcome().unwrap() {
        RuntimeSessionArchiveReadOutcome::Failed(error) => {
            assert!(matches!(
                error.as_ref(),
                RuntimeSessionArchiveError::Io(io_error)
                    if io_error.kind() == std::io::ErrorKind::NotFound
            ));
        }
        RuntimeSessionArchiveReadOutcome::Succeeded(_) => panic!("missing archive must fail"),
    }
    assert_eq!(reader.diagnostics().retained_result_bytes, 0);

    drop(submission);
    drop(reader.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_terminal_read_can_retry_after_the_archive_appears() {
    let root = unique_workspace_temp_root("failed_retry");
    let path_buf = root.join("session.zrsession");
    let path = resolved(&path_buf);
    let reader = RuntimeSessionArchiveReader::with_scheduler(
        test_limits(),
        JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::io().with_worker_threads(1),
        )),
    );
    let first = reader
        .try_submit(path.clone(), BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert!(matches!(
        first
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(_))
    ));
    RuntimeSessionArchive::from_world("retry", &World::empty())
        .unwrap()
        .save_to_path_atomically(&path_buf)
        .unwrap();

    let retry = reader
        .try_submit(path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert_ne!(first.ticket().id(), retry.ticket().id());
    assert_eq!(
        retry
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert!(
        succeeded(retry.outcome().unwrap())
            .archive()
            .contains_slot("retry")
    );

    drop(first);
    drop(retry);
    drop(reader.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn successful_terminal_read_does_not_hide_a_later_file_revision() {
    let root = unique_workspace_temp_root("successful_refresh");
    let path_buf = root.join("session.zrsession");
    RuntimeSessionArchive::from_world("first", &World::empty())
        .unwrap()
        .save_to_path_atomically(&path_buf)
        .unwrap();
    let reader = RuntimeSessionArchiveReader::with_scheduler(
        RuntimeSessionArchiveReaderLimits {
            max_entries: 4,
            max_archive_bytes: TEST_ARCHIVE_RESERVATION_BYTES,
            max_retained_result_bytes: TEST_ARCHIVE_RESERVATION_BYTES * 2,
        },
        JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::io().with_worker_threads(1),
        )),
    );
    let path = resolved(&path_buf);
    let first = reader
        .try_submit(path.clone(), BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    assert_eq!(
        first
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    let first_result = succeeded(first.outcome().unwrap());
    assert!(first_result.archive().contains_slot("first"));

    RuntimeSessionArchive::from_world("second", &World::empty())
        .unwrap()
        .save_to_path_atomically(&path_buf)
        .unwrap();
    let refreshed = reader
        .try_submit(path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert_ne!(first.ticket().id(), refreshed.ticket().id());
    assert_eq!(
        refreshed
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    let refreshed_result = succeeded(refreshed.outcome().unwrap());
    assert!(refreshed_result.archive().contains_slot("second"));
    assert!(!refreshed_result.archive().contains_slot("first"));

    drop(first);
    drop(refreshed);
    drop(first_result);
    drop(refreshed_result);
    drop(reader.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reader_rejects_submission_after_its_runtime_owner_expires() {
    let root = unique_workspace_temp_root("runtime_owner_expired");
    let path = resolved(root.join("session.zrsession"));
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let reader = RuntimeSessionArchiveReader::with_runtime(test_limits(), &core);
    drop(core);
    drop(runtime);

    assert!(matches!(
        reader.try_submit(path, BoundedKeyedIoWorkDeadline::none()),
        Err(RuntimeSessionArchiveReaderSubmitError::RuntimeUnavailable)
    ));
    assert_eq!(reader.diagnostics().retained_result_bytes, 0);

    drop(reader.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reader_submission_path_performs_filesystem_work_only_inside_the_lane_closure() {
    let source = include_str!("service.rs");
    let closure_offset = source.find("Box::new(move ||").unwrap();
    let load_offset = closure_offset
        + source[closure_offset..]
            .find("load_from_path_with_limit(")
            .unwrap();

    assert!(load_offset > closure_offset);
    for forbidden in ["std::fs", "File::open", "canonicalize("] {
        assert!(
            !source[..closure_offset].contains(forbidden),
            "reader admission must not perform caller-side filesystem work `{forbidden}`"
        );
    }
}

fn test_limits() -> RuntimeSessionArchiveReaderLimits {
    RuntimeSessionArchiveReaderLimits {
        max_entries: 4,
        max_archive_bytes: TEST_ARCHIVE_RESERVATION_BYTES,
        max_retained_result_bytes: TEST_ARCHIVE_RESERVATION_BYTES,
    }
}

fn succeeded(outcome: RuntimeSessionArchiveReadOutcome) -> RuntimeSessionArchiveReadArtifact {
    match outcome {
        RuntimeSessionArchiveReadOutcome::Succeeded(artifact) => artifact,
        RuntimeSessionArchiveReadOutcome::Failed(error) => panic!("archive read failed: {error}"),
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
            "runtime11_dynamic_scene_reader_{label}_{}_{}",
            std::process::id(),
            id
        ));
    std::fs::create_dir_all(&root).unwrap();
    root
}
