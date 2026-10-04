use std::fs;

use super::*;

#[test]
fn missing_child_under_existing_root_is_rejected_without_creating() {
    let root = temp_dir("local-path-root");
    let missing_child = root.join("missing").join("child");

    let result = reject_inside_root(&root, &missing_child, HubMessage::raw_text("inside"));

    assert!(result.is_err());
    assert!(!missing_child.exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn create_owned_dir_rejects_existing_directory_with_caller_message() {
    let root = temp_dir("local-path-owned-existing");
    let existing = root.join("existing");
    fs::create_dir(&existing).unwrap();

    let error =
        create_owned_dir(&existing, || HubMessage::raw_text("custom already exists")).unwrap_err();

    assert_eq!(error.to_string(), "custom already exists");
    assert!(existing.is_dir());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cleanup_dir_on_error_removes_dir_only_on_error() {
    let root = temp_dir("local-path-cleanup");
    let failed_dir = root.join("failed");
    fs::create_dir(&failed_dir).unwrap();
    fs::write(failed_dir.join("partial.txt"), "partial").unwrap();

    let error =
        cleanup_dir_on_error::<()>(&failed_dir, Err(HubError::message("copy failed"))).unwrap_err();

    assert_eq!(error.to_string(), "copy failed");
    assert!(!failed_dir.exists());

    let successful_dir = root.join("successful");
    fs::create_dir(&successful_dir).unwrap();
    let value = cleanup_dir_on_error(&successful_dir, Ok(42)).unwrap();

    assert_eq!(value, 42);
    assert!(successful_dir.is_dir());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cleanup_failure_is_reported_instead_of_claiming_terminal_rollback() {
    let root = temp_dir("local-path-cleanup-failure");
    let not_a_directory = root.join("owned-output");
    fs::write(&not_a_directory, "still present").unwrap();

    let error =
        cleanup_dir_on_error::<()>(&not_a_directory, Err(HubError::message("operation failed")))
            .unwrap_err();

    assert!(error.to_string().contains("operation failed"));
    assert!(error.to_string().contains("failed to remove owned output"));
    assert!(not_a_directory.is_file());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sibling_with_shared_prefix_is_not_inside_root() {
    let parent = temp_dir("local-path-parent");
    let root = parent.join("Game");
    let sibling = parent.join("GameBuild").join("out");
    fs::create_dir_all(&root).unwrap();

    reject_inside_root(&root, &sibling, HubMessage::raw_text("inside")).unwrap();

    fs::remove_dir_all(parent).unwrap();
}

fn temp_dir(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zircon-hub-{label}-{}",
        crate::projects::now_unix_ms()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}
