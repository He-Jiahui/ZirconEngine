use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use zircon_runtime_interface::{
    ProfileSnapshot, PROFILE_HOTSPOTS_FILE, PROFILE_TIMELINE_NATIVE_FILE,
    PROFILE_TIMELINE_PERFETTO_FILE,
};

use super::{export_snapshot, export_snapshot_with_after_native, ProfileExportError};

fn managed_test_root(label: &str) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .expect("managed profile tests require CARGO_TARGET_DIR")
        .join(format!(
            "zircon-profile-single-flight-{label}-{}",
            std::process::id()
        ))
}

fn snapshot(root: &Path, session_id: &str, frame_budget_ms: f64) -> ProfileSnapshot {
    ProfileSnapshot {
        output_root: root.to_string_lossy().into_owned(),
        session_id: session_id.to_string(),
        frame_budget_ms,
        ..ProfileSnapshot::default()
    }
}

fn reset_root(root: &Path) {
    let _ = fs::remove_dir_all(root);
    fs::create_dir_all(root).expect("create managed test root");
}

#[test]
fn same_session_export_serializes_entire_optional_artifact_transition_across_path_aliases() {
    let root = managed_test_root("same-session");
    reset_root(&root);
    let first = snapshot(&root, "same-session", 7.0);
    let second = snapshot(&root.join("."), "same-session", 9.0);
    let (paused_tx, paused_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let first_worker = thread::spawn(move || {
        export_snapshot_with_after_native(&first, true, || {
            paused_tx.send(()).expect("signal native artifact written");
            release_rx.recv().expect("release first export");
        })
    });
    paused_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("first export reached the native-file phase");

    let (owner_contention_tx, owner_contention_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let second_worker = thread::spawn(move || {
        super::session_owner::after_next_owner_would_block(move || {
            owner_contention_tx
                .send(())
                .expect("signal real owner try_lock returned WouldBlock");
        });
        let report = export_snapshot(&second, false);
        done_tx.send(()).expect("signal second export completed");
        report
    });
    let owner_contention_observed = owner_contention_rx.recv_timeout(Duration::from_secs(5));
    let second_finished_while_first_paused = done_rx.recv_timeout(Duration::from_millis(500));
    release_tx.send(()).expect("release first export");

    let first_report = first_worker
        .join()
        .expect("first worker")
        .expect("first export");
    let second_report = second_worker
        .join()
        .expect("second worker")
        .expect("second export");
    assert_eq!(
        owner_contention_observed,
        Ok(()),
        "the test hook must follow a real owner try_lock WouldBlock result"
    );
    assert_eq!(
        second_finished_while_first_paused,
        Err(RecvTimeoutError::Timeout),
        "the second export must remain incomplete while the first export holds the owner"
    );
    assert!(first_report
        .files
        .iter()
        .any(|file| file == PROFILE_TIMELINE_PERFETTO_FILE));
    assert!(!second_report
        .files
        .iter()
        .any(|file| file == PROFILE_TIMELINE_PERFETTO_FILE));
    let export_dir = Path::new(&second_report.export_dir);
    assert!(!export_dir.join(PROFILE_TIMELINE_PERFETTO_FILE).exists());
    let native: ProfileSnapshot = serde_json::from_slice(
        &fs::read(export_dir.join(PROFILE_TIMELINE_NATIVE_FILE)).expect("final native timeline"),
    )
    .expect("parse final native timeline");
    assert_eq!(native.frame_budget_ms, 9.0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn different_session_exports_can_complete_while_another_session_is_paused() {
    let root = managed_test_root("different-sessions");
    reset_root(&root);
    let first = snapshot(&root, "session-a", 7.0);
    let second = snapshot(&root, "session-b", 9.0);
    let (paused_tx, paused_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let first_worker = thread::spawn(move || {
        export_snapshot_with_after_native(&first, false, || {
            paused_tx.send(()).expect("signal first session paused");
            release_rx.recv().expect("release first session");
        })
    });
    paused_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("first session reached native-file phase");

    let (done_tx, done_rx) = mpsc::channel();
    let second_worker = thread::spawn(move || {
        let report = export_snapshot(&second, false);
        done_tx.send(()).expect("signal second session completed");
        report
    });
    let second_finished_while_first_paused = done_rx.recv_timeout(Duration::from_secs(5));
    release_tx.send(()).expect("release first session");

    first_worker
        .join()
        .expect("first worker")
        .expect("first export");
    let second_report = second_worker
        .join()
        .expect("second worker")
        .expect("second export");
    assert_eq!(second_finished_while_first_paused, Ok(()));
    assert!(Path::new(&second_report.export_dir)
        .join(PROFILE_TIMELINE_NATIVE_FILE)
        .is_file());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_export_releases_the_session_owner_for_a_retry() {
    let root = managed_test_root("error-release");
    reset_root(&root);
    let snapshot = snapshot(&root, "retry-session", 7.0);
    let first = export_snapshot(&snapshot, false).expect("create the session directory");
    let blocked_path = Path::new(&first.export_dir).join(PROFILE_HOTSPOTS_FILE);
    fs::remove_file(&blocked_path).expect("remove prior hotspots file");
    fs::create_dir(&blocked_path).expect("block hotspot file creation");

    let error = export_snapshot(&snapshot, false).expect_err("blocked export must fail");
    assert!(matches!(error, ProfileExportError::WriteFile { .. }));
    fs::remove_dir(&blocked_path).expect("remove blocker");
    let retry = export_snapshot(&snapshot, false).expect("retry after error");
    assert!(Path::new(&retry.export_dir)
        .join(PROFILE_HOTSPOTS_FILE)
        .is_file());
    let _ = fs::remove_dir_all(root);
}
