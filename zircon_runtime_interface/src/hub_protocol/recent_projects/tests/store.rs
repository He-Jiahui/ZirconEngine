use std::cell::Cell;
use std::fs;
use std::time::SystemTime;

use crate::project::{ProjectManifestSummary, PROJECT_MANIFEST_FORMAT_VERSION};

use super::{
    HubRecentProjectsLoadDisposition, HubRecentProjectsStore, HubRecentProjectsStoreError,
    HubRecentProjectsWritePolicy, HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1,
};
use crate::hub_protocol::HubRecentProjectV1;

#[test]
fn corrupted_projection_is_quarantined_and_rebuilt_during_a_bounded_mutation() {
    let root = temporary_root("corrupt");
    let path = root.join("recent_projects.json");
    fs::create_dir_all(&root).expect("create fixture root");
    fs::write(&path, b"not-json").expect("write corrupt fixture");
    let store = HubRecentProjectsStore::new(path.clone());

    let result = store
        .update(
            HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_millis(50)),
            |registry| registry.record(project()),
        )
        .expect("rebuild corrupt recent projection");

    assert_eq!(
        result.load_disposition(),
        HubRecentProjectsLoadDisposition::RebuildRequiredAfterCorruption
    );
    assert!(result.quarantined_path().is_some());
    assert_eq!(result.registry().projects, vec![project()]);
    assert_eq!(store.load_projection().unwrap().registry().revision(), 1);
    fs::remove_dir_all(root).expect("remove fixture root");
}

#[test]
fn oversized_projection_is_nonblocking_and_reports_a_rebuild_requirement() {
    let root = temporary_root("oversized");
    let path = root.join("recent_projects.json");
    fs::create_dir_all(&root).expect("create fixture root");
    fs::write(
        &path,
        vec![b'x'; HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1 + 1],
    )
    .expect("write oversized fixture");
    let store = HubRecentProjectsStore::new(path);

    let projection = store.load_projection().expect("bounded projection result");

    assert_eq!(
        projection.disposition(),
        HubRecentProjectsLoadDisposition::RebuildRequiredAfterOversize
    );
    assert!(projection.registry().projects.is_empty());
    fs::remove_dir_all(root).expect("remove fixture root");
}

#[test]
fn clean_compare_rejects_rebuildable_input_changed_after_observation() {
    for (label, bytes, disposition) in [
        (
            "malformed-clean-cas",
            b"not-json".to_vec(),
            HubRecentProjectsLoadDisposition::RebuildRequiredAfterCorruption,
        ),
        (
            "oversized-clean-cas",
            vec![b'x'; HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1 + 1],
            HubRecentProjectsLoadDisposition::RebuildRequiredAfterOversize,
        ),
    ] {
        let root = temporary_root(label);
        let path = root.join("recent_projects.json");
        fs::create_dir_all(&root).expect("create fixture root");
        let store = HubRecentProjectsStore::new(&path);
        let observed = store
            .load_projection()
            .expect("observe clean absent registry");
        assert_eq!(
            observed.disposition(),
            HubRecentProjectsLoadDisposition::Clean
        );
        assert_eq!(observed.registry().revision(), 0);
        fs::write(&path, &bytes).expect("replace observed registry before CAS");
        let callback_called = Cell::new(false);

        let error = store
            .compare_and_update_if_clean(
                HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_millis(50)),
                0,
                |_| {
                    callback_called.set(true);
                    Ok(())
                },
            )
            .expect_err("rebuildable input must not admit the clean-only update");

        assert!(matches!(
            error,
            HubRecentProjectsStoreError::ProjectionNotClean {
                disposition: actual,
                ..
            } if actual == disposition
        ));
        assert!(!callback_called.get());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(!fs::read_dir(&root).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".corrupt-")));
        fs::remove_dir_all(root).expect("remove fixture root");
    }
}

#[cfg(windows)]
#[test]
fn expired_windows_writer_attempts_release_named_mutex_handles() {
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn GetProcessHandleCount(process: *mut c_void, count: *mut u32) -> i32;
    }

    fn process_handle_count() -> u32 {
        let mut count = 0;
        // SAFETY: the pseudo process handle is valid for this process and `count` is writable.
        let success = unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut count) };
        assert_ne!(success, 0, "read current process handle count");
        count
    }

    let root = temporary_root("expired-windows-handles");
    let store = HubRecentProjectsStore::new(root.join("recent_projects.json"));
    let before = process_handle_count();
    for _ in 0..128 {
        let error = store
            .update(
                HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::ZERO),
                |_| Ok(()),
            )
            .expect_err("an expired policy should stop before mutation");
        assert!(matches!(
            error,
            HubRecentProjectsStoreError::LeaseDeadlineExceeded { .. }
        ));
    }
    let after = process_handle_count();
    assert!(
        after <= before + 32,
        "expired writer attempts retained {} process handles",
        after.saturating_sub(before),
    );
}

#[test]
fn cancelled_write_returns_a_typed_terminal_error_before_mutation() {
    let root = temporary_root("cancelled");
    let store = HubRecentProjectsStore::new(root.join("recent_projects.json"));
    let cancelled = || true;

    let error = store
        .update(
            HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_secs(1))
                .with_cancellation(&cancelled),
            |registry| registry.record(project()),
        )
        .expect_err("cancelled writer must not mutate history");

    assert!(matches!(
        error,
        HubRecentProjectsStoreError::LeaseCancelled { .. }
    ));
    assert!(!store.path().exists());
}

#[test]
fn compare_and_update_rejects_a_stale_projection_revision() {
    let root = temporary_root("stale-revision");
    let store = HubRecentProjectsStore::new(root.join("recent_projects.json"));
    let first = store
        .update(
            HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_millis(50)),
            |registry| registry.record(project()),
        )
        .expect("write initial projection");

    let error = store
        .compare_and_update(
            HubRecentProjectsWritePolicy::with_timeout(std::time::Duration::from_millis(50)),
            first.previous_revision(),
            |registry| registry.remove("E:/Projects/Game"),
        )
        .expect_err("stale revision must not replace a newer projection");

    assert!(matches!(
        error,
        HubRecentProjectsStoreError::RevisionConflict {
            expected_revision: 0,
            actual_revision: 1,
            ..
        }
    ));
    std::fs::remove_dir_all(root).expect("remove fixture root");
}

fn project() -> HubRecentProjectV1 {
    HubRecentProjectV1::new(
        ProjectManifestSummary {
            name: "Game".to_string(),
            engine_version_req: None,
            template_receipt: None,
            default_scene: "res://scenes/main.scene.toml".to_string(),
            format_version: PROJECT_MANIFEST_FORMAT_VERSION,
            project_guid: None,
        },
        "E:/Projects/Game",
        42,
    )
    .expect("fixture recent project")
}

fn temporary_root(label: &str) -> PathBuf {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("recent-project filesystem tests require coordinator-managed CARGO_TARGET_DIR");
    PathBuf::from(target_directory).join(format!(
        "zircon-recent-store-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("current time")
            .as_nanos(),
    ))
}

use std::path::PathBuf;
