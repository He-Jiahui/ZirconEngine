use std::fs;
use std::io;

use super::super::ProjectAuthorityError;
use super::super::authority::{
    cleanup_failed_transaction_staging, commit_staged_directory, rollback_committed_project,
};
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

#[test]
fn failed_post_commit_restore_preserves_the_original_backup_and_published_project() {
    let root = temp_root("open-rollback-restore-failure");
    let target = root.join("project");
    let staging = root.join("staging");
    let backup = root.join("backup");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("new-project"), "published").unwrap();
    fs::create_dir(&backup).unwrap();
    let mut call = 0;

    let error = rollback_committed_project(&staging, &target, &backup, true, |from, to| {
        call += 1;
        if call == 2 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "injected original-target restore failure",
            ))
        } else {
            fs::rename(from, to)
        }
    })
    .unwrap_err();

    assert!(matches!(
        error,
        ProjectAuthorityError::PostCommitRollbackFailed {
            ref from,
            ref to,
            backup: Some(ref original_empty_target),
            ..
        } if from == &backup && to == &target && original_empty_target == &backup
    ));
    assert!(!target.exists());
    assert!(
        backup.is_dir(),
        "the caller's original empty target is retained"
    );
    assert!(
        staging.join("new-project").is_file(),
        "the failed published project remains recoverable instead of being deleted"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_post_commit_staging_move_preserves_the_published_project_and_backup() {
    let root = temp_root("open-rollback-staging-move-failure");
    let target = root.join("project");
    let staging = root.join("staging");
    let backup = root.join("backup");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("new-project"), "published").unwrap();
    fs::create_dir(&backup).unwrap();

    let error = rollback_committed_project(&staging, &target, &backup, true, |_from, _to| {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "injected published-project staging move failure",
        ))
    })
    .unwrap_err();

    assert!(matches!(
        error,
        ProjectAuthorityError::PostCommitRollbackFailed {
            ref from,
            ref to,
            backup: Some(ref original_empty_target),
            ..
        } if from == &target && to == &staging && original_empty_target == &backup
    ));
    assert!(
        target.join("new-project").is_file(),
        "the published project remains recoverable when it cannot move to staging"
    );
    assert!(
        backup.is_dir(),
        "the caller's original empty target is retained"
    );
    assert!(!staging.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_staging_creation_preserves_a_directory_not_owned_by_the_transaction() {
    let root = temp_root("staging-ownership");
    let staging = root.join("staging");
    fs::create_dir(&staging).unwrap();
    let retained_file = staging.join("retain-me");
    fs::write(&retained_file, "prior transaction state").unwrap();

    cleanup_failed_transaction_staging(&staging, false, false);

    assert!(
        retained_file.is_file(),
        "cleanup must not delete a staging directory that this transaction did not create"
    );
    fs::remove_dir_all(root).unwrap();
}
