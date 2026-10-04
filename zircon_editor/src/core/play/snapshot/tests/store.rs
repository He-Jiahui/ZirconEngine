use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::RelPath;

use super::{complete_snapshot_write, play_snapshot_path_error, MaterializedPlayScene};

fn test_output_root(name: &str) -> PathBuf {
    let base = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target-test-output"));
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    base.join(format!(
        "zircon-editor-play-{name}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn failed_snapshot_write_retains_cleanup_owner_until_retry_succeeds() {
    let owned_root = test_output_root("materialize-cleanup-retry");
    if let Some(parent) = owned_root.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&owned_root, "forces remove_dir_all to fail").unwrap();
    let scene = MaterializedPlayScene {
        instance_id: "fault-injected".to_string(),
        path: owned_root.join("play-scene.zrscene.json"),
        relative_path: RelPath::parse(".zircon/play/fault-injected/play-scene.zrscene.json")
            .unwrap(),
        owned_root: Some(owned_root.clone()),
    };

    let failure = complete_snapshot_write(scene, Err("fault-injected write failure".into()))
        .expect_err("failed write and failed cleanup must retain the snapshot owner");
    let (pending_scene, message) = failure.into_parts();
    assert!(message.contains("fault-injected write failure"));
    assert!(message.contains("snapshot cleanup remains pending"));
    let mut pending_scene = pending_scene.expect("cleanup owner must remain retryable");

    fs::remove_file(&owned_root).unwrap();
    fs::create_dir_all(&owned_root).unwrap();
    pending_scene.cleanup().unwrap();
    assert!(!owned_root.exists());
}

#[cfg(windows)]
#[test]
fn snapshot_error_messages_hide_windows_verbatim_operation_paths() {
    assert_eq!(
        play_snapshot_path_error(
            "failed to create play snapshot",
            Path::new(r"\\?\C:\projects\forest\.zircon\play\instance"),
            "access denied",
        ),
        "failed to create play snapshot C:\\projects\\forest\\.zircon\\play\\instance: access denied"
    );
}
