use std::fs;
use std::io;

use super::super::authority::commit_staged_directory;
use super::super::ProjectAuthorityError;
use super::temp_root;

#[test]
fn failed_commit_restores_the_original_empty_target() {
    let root = temp_root("commit-restore");
    let target = root.join("project");
    let staging = root.join("staging");
    let backup = root.join("backup");
    fs::create_dir(&target).unwrap();
    fs::create_dir(&staging).unwrap();
    let mut call = 0;

    let error = commit_staged_directory(&staging, &target, &backup, true, |from, to| {
        call += 1;
        if call == 2 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "injected commit failure",
            ))
        } else {
            fs::rename(from, to)
        }
    })
    .unwrap_err();

    assert!(matches!(error, ProjectAuthorityError::Io { .. }));
    assert!(target.is_dir());
    assert!(!backup.exists());
    assert!(staging.is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_restore_returns_typed_error_and_preserves_the_only_backup() {
    let root = temp_root("commit-rollback-failure");
    let target = root.join("project");
    let staging = root.join("staging");
    let backup = root.join("backup");
    fs::create_dir(&target).unwrap();
    fs::create_dir(&staging).unwrap();
    let mut call = 0;

    let error = commit_staged_directory(&staging, &target, &backup, true, |from, to| {
        call += 1;
        if call >= 2 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "injected transaction failure",
            ))
        } else {
            fs::rename(from, to)
        }
    })
    .unwrap_err();

    assert!(matches!(
        error,
        ProjectAuthorityError::CommitRollbackFailed { .. }
    ));
    assert!(!target.exists());
    assert!(backup.is_dir());
    assert!(staging.is_dir());
    fs::remove_dir_all(root).unwrap();
}
