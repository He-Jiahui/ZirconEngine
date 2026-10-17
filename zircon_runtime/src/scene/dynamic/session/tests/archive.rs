use super::RuntimeSessionArchive;

#[test]
fn invalid_runtime_session_archive_generation_caches_its_seal_rejection() {
    let archive = RuntimeSessionArchive::from_payload(u32::MAX, Vec::new());

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

struct MutationFixture(std::path::PathBuf, std::path::PathBuf);

impl Drop for MutationFixture {
    fn drop(&mut self) {
        let root = &self.0;
        let parent = &self.1;
        if !parent.is_absolute() || root.parent() != Some(parent.as_path()) {
            return;
        }
        let Ok(metadata) = std::fs::symlink_metadata(root) else {
            return;
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return;
            }
        }
        let Ok(resolved) = root.canonicalize() else {
            return;
        };
        if resolved != *root || resolved.parent() != Some(parent.as_path()) {
            return;
        }
        let _ = std::fs::remove_dir_all(root);
    }
}

fn mutation_fixture(label: &str) -> MutationFixture {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should follow Unix epoch")
        .as_nanos();
    let parent = std::env::temp_dir()
        .canonicalize()
        .expect("registered temporary parent must already exist");
    assert!(
        parent.is_absolute(),
        "temporary parent must be canonical and absolute"
    );
    let root = parent.join(format!(
        "zircon-session-mutation-{label}-{}-{nonce}",
        std::process::id(),
    ));
    assert_eq!(root.parent(), Some(parent.as_path()));
    std::fs::create_dir(&root).expect("fixture must exclusively create a new direct child");
    MutationFixture(root, parent)
}

#[test]
fn concurrent_path_mutations_reject_stale_read_before_lost_update() {
    use super::super::io::mutate_archive_at_path_with_report_atomically;
    use crate::scene::{
        RuntimeSessionArchiveError, RuntimeSessionMetadata, RuntimeSessionSlot, World,
    };
    use std::sync::mpsc;
    use std::time::Duration;

    let fixture = mutation_fixture("concurrent");
    let path = fixture.0.join("session.zrsession");
    let mut initial = RuntimeSessionArchive::from_world("left", &World::empty()).unwrap();
    initial
        .push_slot(
            RuntimeSessionSlot::from_world_with_metadata(
                "right",
                &World::empty(),
                RuntimeSessionMetadata::default(),
            )
            .unwrap(),
        )
        .unwrap();
    initial.save_to_path_atomically(&path).unwrap();

    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_a_tx, release_a_rx) = mpsc::channel();
    let (release_b_tx, release_b_rx) = mpsc::channel();
    let (done_a_tx, done_a_rx) = mpsc::channel();
    let (done_b_tx, done_b_rx) = mpsc::channel();
    let path_a = path.clone();
    let path_b = fixture
        .0
        .join("uncreated")
        .join("..")
        .join("session.zrsession");
    let ready_a = ready_tx.clone();
    let ready_b = ready_tx.clone();
    drop(ready_tx);
    let a = std::thread::spawn(move || {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            mutate_archive_at_path_with_report_atomically(&path_a, |archive| {
                ready_a.send(0).map_err(std::io::Error::other)?;
                release_a_rx
                    .recv_timeout(Duration::from_secs(5))
                    .map_err(std::io::Error::other)?;
                archive.rename_slot("left", "left-new")?;
                Ok(())
            })
        }));
        let _ = done_a_tx.send(());
        outcome
    });
    let b = std::thread::spawn(move || {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            mutate_archive_at_path_with_report_atomically(&path_b, |archive| {
                ready_b.send(1).map_err(std::io::Error::other)?;
                release_b_rx
                    .recv_timeout(Duration::from_secs(5))
                    .map_err(std::io::Error::other)?;
                archive.rename_slot("right", "right-new")?;
                Ok(())
            })
        }));
        let _ = done_b_tx.send(());
        outcome
    });
    let first_ready = ready_rx.recv_timeout(Duration::from_secs(5));
    let second_ready = ready_rx.recv_timeout(Duration::from_secs(5));
    let both_loaded = matches!(
        (&first_ready, &second_ready),
        (Ok(0), Ok(1)) | (Ok(1), Ok(0))
    );
    if both_loaded {
        let _ = release_a_tx.send(());
    }
    drop(release_a_tx);
    let a_completed = done_a_rx.recv_timeout(Duration::from_secs(5));
    if both_loaded && a_completed.is_ok() {
        let _ = release_b_tx.send(());
    }
    drop(release_b_tx);
    let b_completed = done_b_rx.recv_timeout(Duration::from_secs(5));
    // Close every control channel and collect both threads before assertions.
    drop(ready_rx);
    drop(done_a_rx);
    drop(done_b_rx);
    let result_a = a.join();
    let result_b = b.join();
    assert!(
        both_loaded,
        "both callbacks must observe the old file before A commits"
    );
    assert!(
        a_completed.is_ok() && b_completed.is_ok(),
        "both workers must terminate within the bounded protocol"
    );
    result_a
        .expect("A thread must join")
        .expect("A callback must not panic")
        .expect("A commits first");
    let rejected = result_b
        .expect("B thread must join")
        .expect("B callback must not panic");
    assert!(matches!(
        rejected,
        Err(RuntimeSessionArchiveError::StalePathCommit { .. })
    ));
    let after_a = RuntimeSessionArchive::load_from_path(&path).unwrap();
    assert!(after_a.contains_slot("left-new"));
    assert!(after_a.contains_slot("right"));
    assert!(!after_a.contains_slot("left"));
    assert!(!after_a.contains_slot("right-new"));
    mutate_archive_at_path_with_report_atomically(&path, |archive| {
        archive.rename_slot("right", "right-new")?;
        Ok(())
    })
    .expect("a fresh read can retry B without losing A");
    let combined = RuntimeSessionArchive::load_from_path(&path).unwrap();
    assert!(combined.contains_slot("left-new") && combined.contains_slot("right-new"));
    assert!(!combined.contains_slot("left") && !combined.contains_slot("right"));
}

#[test]
fn failed_path_mutation_preserves_bytes_and_releases_reservation() {
    use super::super::io::mutate_archive_at_path_with_report_atomically;
    let fixture = mutation_fixture("error");
    let path = fixture.0.join("session.zrsession");
    RuntimeSessionArchive::from_world("slot", &crate::scene::World::empty())
        .unwrap()
        .save_to_path_atomically(&path)
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let result = mutate_archive_at_path_with_report_atomically(&path, |archive| {
        archive.rename_slot("missing", "other")?;
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    mutate_archive_at_path_with_report_atomically(&path, |archive| {
        archive.rename_slot("slot", "after-error")?;
        Ok(())
    })
    .expect("failed callback must release its path reservation");
    assert!(RuntimeSessionArchive::load_from_path(&path)
        .unwrap()
        .contains_slot("after-error"));
}

#[test]
fn panicking_path_mutation_releases_reservation_without_publication() {
    use super::super::io::mutate_archive_at_path_with_report_atomically;
    let fixture = mutation_fixture("panic");
    let path = fixture.0.join("session.zrsession");
    RuntimeSessionArchive::from_world("slot", &crate::scene::World::empty())
        .unwrap()
        .save_to_path_atomically(&path)
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mutate_archive_at_path_with_report_atomically(
            &path,
            |archive| -> Result<(), crate::scene::RuntimeSessionArchiveError> {
                archive.rename_slot("slot", "unpublished")?;
                panic!("intentional mutation callback panic");
            },
        )
    }));
    assert!(result.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    mutate_archive_at_path_with_report_atomically(&path, |archive| {
        archive.rename_slot("slot", "after-panic")?;
        Ok(())
    })
    .expect("panicking callback must release its path reservation");
    let loaded = RuntimeSessionArchive::load_from_path(&path).unwrap();
    assert!(loaded.contains_slot("after-panic"));
    assert!(!loaded.contains_slot("unpublished"));
}
