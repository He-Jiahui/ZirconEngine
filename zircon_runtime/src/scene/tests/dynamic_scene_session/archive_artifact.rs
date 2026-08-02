use std::time::{Duration, Instant};

use crate::core::runtime::{
    BoundedKeyedIoTerminal, BoundedKeyedIoWaitResult, BoundedKeyedIoWorkDeadline, JobScheduler,
    TaskPool, TaskPoolDescriptor,
};
use crate::scene::{
    NodeKind, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveWriter,
    RuntimeSessionArchiveWriterLimits, World,
};

use super::unique_temp_root;

#[test]
fn runtime_session_archive_seals_one_shared_artifact_per_generation() {
    let mut source = World::empty();
    source.spawn_node(NodeKind::Mesh);
    let archive = RuntimeSessionArchive::from_world("manual", &source)
        .expect("world capture should produce an archive generation");

    let first = archive
        .sealed_artifact()
        .expect("archive generation should seal");
    let second = archive
        .sealed_artifact()
        .expect("stable generation should reuse its sealed artifact");

    assert_eq!(first.generation(), second.generation());
    assert!(first.shares_payload_with(&second));
    assert_eq!(first.manifest().slot_count(), 1);
    assert_eq!(first.statistics().total_entity_count, 1);
    assert_eq!(first.serialized_bytes(), second.serialized_bytes());

    let diagnostics = second.diagnostics();
    assert_eq!(diagnostics.capture_count, 1);
    assert_eq!(diagnostics.normalize_count, 1);
    assert_eq!(diagnostics.validate_count, 1);
    assert_eq!(diagnostics.serialize_count, 1);
    assert_eq!(diagnostics.internal_json_roundtrip_count, 0);
}

#[test]
fn runtime_session_archive_mutation_publishes_a_new_generation() {
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_world("manual", &source)
        .expect("world capture should produce an archive generation");
    let before = archive
        .sealed_artifact()
        .expect("initial generation should seal");

    archive
        .touch_slot("manual", 42)
        .expect("touch should mutate the archive");
    let after = archive
        .sealed_artifact()
        .expect("mutated generation should seal independently");

    assert!(after.generation() > before.generation());
    assert!(!after.shares_payload_with(&before));
    assert_eq!(
        after
            .manifest()
            .slot("manual")
            .unwrap()
            .metadata
            .updated_at_unix_millis,
        Some(42)
    );
}

#[test]
fn stale_runtime_session_archive_artifact_cannot_overwrite_newer_path_generation() {
    let root = unique_temp_root("stale_archive_artifact");
    let path = root.join("session.zrsession");
    let source = World::empty();
    let mut archive = RuntimeSessionArchive::from_world("old", &source).unwrap();
    let old = archive
        .sealed_artifact()
        .expect("initial lineage revision should seal");
    archive
        .rename_slot("old", "new")
        .expect("rename should publish the next lineage revision");
    let new = archive
        .sealed_artifact()
        .expect("new lineage revision should seal");

    new.save_to_path_atomically(&path)
        .expect("newer artifact should publish");
    let error = old
        .save_to_path_atomically(&path)
        .expect_err("stale artifact must not overwrite the newer lineage revision");

    assert!(matches!(
        error,
        RuntimeSessionArchiveError::StaleArtifactRevision {
            artifact_revision,
            committed_revision,
        } if artifact_revision == old.revision()
            && committed_revision == new.revision()
    ));
    let loaded = RuntimeSessionArchive::load_from_path(&path).unwrap();
    assert!(loaded.contains_slot("new"));
    assert!(!loaded.contains_slot("old"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_session_archive_path_aliases_share_one_causal_commit_identity() {
    let root = unique_temp_root("archive_path_alias");
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let canonical_path = root.join("session.zrsession");
    let alias_path = nested.join("..").join("session.zrsession");
    let mut archive = RuntimeSessionArchive::from_world("old", &World::empty()).unwrap();
    let old = archive.sealed_artifact().unwrap();
    archive.rename_slot("old", "new").unwrap();
    let new = archive.sealed_artifact().unwrap();

    new.save_to_path_atomically(&canonical_path)
        .expect("newer artifact should publish");
    let error = old
        .save_to_path_atomically(&alias_path)
        .expect_err("an alias must not bypass the committed lineage revision");

    assert!(matches!(
        error,
        RuntimeSessionArchiveError::StaleArtifactRevision { .. }
    ));
    let loaded = RuntimeSessionArchive::load_from_path(&canonical_path).unwrap();
    assert!(loaded.contains_slot("new"));
    assert!(!loaded.contains_slot("old"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn oversized_runtime_session_archive_generation_is_rejected_once_and_never_retried() {
    let archive = RuntimeSessionArchive::from_world("bounded", &World::empty()).unwrap();

    let first = archive
        .sealed_artifact_with_limit_for_test(1)
        .expect_err("one byte cannot hold a session archive");
    let second = archive
        .sealed_artifact_with_limit_for_test(usize::MAX)
        .expect_err("a rejected generation must not serialize again under a larger retry limit");

    assert!(matches!(
        first,
        RuntimeSessionArchiveError::ArtifactTooLarge {
            limit_bytes: 1,
            estimated_bytes,
        } if estimated_bytes > 1
    ));
    assert!(matches!(
        second,
        RuntimeSessionArchiveError::ArtifactTooLarge { limit_bytes: 1, .. }
    ));
    assert_eq!(archive.artifact_diagnostics().serialize_count, 1);
}

#[test]
fn invalid_runtime_session_archive_generation_caches_its_seal_rejection() {
    let mut archive = RuntimeSessionArchive::from_world("invalid", &World::empty()).unwrap();
    archive.format_version = u32::MAX;

    let first = archive
        .sealed_artifact()
        .expect_err("unsupported format must reject the generation");
    let second = archive
        .sealed_artifact()
        .expect_err("deterministic validation rejection must be cached");

    assert_eq!(first.to_string(), second.to_string());
    let diagnostics = archive.artifact_diagnostics();
    assert_eq!(diagnostics.validate_count, 0);
    assert_eq!(diagnostics.serialize_count, 0);
}

#[test]
fn runtime_session_archive_writer_uses_the_shared_bounded_io_lane() {
    let root = unique_temp_root("archive_writer_lane");
    let path = root.join("session.zrsession");
    let artifact = RuntimeSessionArchive::from_world("lane", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits {
            max_entries: 2,
            max_retained_bytes: 1024 * 1024,
        },
        scheduler,
    );

    let submission = writer
        .try_submit(artifact, &path, BoundedKeyedIoWorkDeadline::none())
        .expect("bounded writer should admit the small sealed artifact");
    assert_eq!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    submission
        .take_outcome()
        .expect("terminal work must publish an outcome")
        .expect("archive write should succeed");
    assert!(
        RuntimeSessionArchive::load_from_path(&path)
            .unwrap()
            .contains_slot("lane")
    );

    drop(writer.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_session_archive_writer_orders_distinct_lineages_by_path_submission() {
    let root = unique_temp_root("archive_writer_path_order");
    let path = root.join("session.zrsession");
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (blocker_started_tx, blocker_started_rx) = std::sync::mpsc::sync_channel(0);
    let (release_blocker_tx, release_blocker_rx) = std::sync::mpsc::sync_channel(0);
    scheduler.spawn(move || {
        blocker_started_tx.send(()).unwrap();
        release_blocker_rx.recv().unwrap();
    });
    blocker_started_rx.recv().unwrap();
    let writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits::default(),
        scheduler,
    );
    let first = RuntimeSessionArchive::from_world("first", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let second = RuntimeSessionArchive::from_world("second", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();

    let first_submission = writer
        .try_submit(first, &path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    let second_submission = writer
        .try_submit(second, &path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();

    assert!(matches!(
        first_submission.ticket().terminal(),
        Some(BoundedKeyedIoTerminal::Superseded { .. })
    ));
    release_blocker_tx.send(()).unwrap();
    assert_eq!(
        second_submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(second_submission.take_outcome(), Some(Ok(())));
    assert!(
        RuntimeSessionArchive::load_from_path(&path)
            .unwrap()
            .contains_slot("second")
    );

    drop(writer.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_session_archive_path_intent_orders_multiple_writers_and_direct_saves() {
    let root = unique_temp_root("archive_writer_cross_lane_order");
    let path = root.join("session.zrsession");
    let first_scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (blocker_started_tx, blocker_started_rx) = std::sync::mpsc::sync_channel(0);
    let (release_blocker_tx, release_blocker_rx) = std::sync::mpsc::sync_channel(0);
    first_scheduler.spawn(move || {
        blocker_started_tx.send(()).unwrap();
        release_blocker_rx.recv().unwrap();
    });
    blocker_started_rx.recv().unwrap();
    let first_writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits::default(),
        first_scheduler,
    );
    let second_writer = RuntimeSessionArchiveWriter::with_scheduler(
        RuntimeSessionArchiveWriterLimits::default(),
        JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::io().with_worker_threads(1),
        )),
    );
    let first = RuntimeSessionArchive::from_world("first", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let second = RuntimeSessionArchive::from_world("second", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();
    let third = RuntimeSessionArchive::from_world("third", &World::empty())
        .unwrap()
        .sealed_artifact()
        .unwrap();

    let first_submission = first_writer
        .try_submit(first, &path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    let second_submission = second_writer
        .try_submit(second, &path, BoundedKeyedIoWorkDeadline::none())
        .unwrap();
    assert_eq!(
        second_submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(second_submission.take_outcome(), Some(Ok(())));

    third.save_to_path_atomically(&path).unwrap();
    release_blocker_tx.send(()).unwrap();
    assert!(matches!(
        first_submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Failed(_))
    ));
    assert!(matches!(
        first_submission.take_outcome(),
        Some(Err(RuntimeSessionArchiveError::StalePathWrite { .. }))
    ));
    let loaded = RuntimeSessionArchive::load_from_path(&path).unwrap();
    assert!(loaded.contains_slot("third"));

    drop(first_writer.shutdown());
    drop(second_writer.shutdown());
    std::fs::remove_dir_all(root).unwrap();
}
