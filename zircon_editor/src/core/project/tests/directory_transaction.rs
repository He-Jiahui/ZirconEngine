use std::fs;
use std::io;

use super::super::authority::{
    cleanup_failed_transaction_staging, commit_staged_directory, finalize_published_project,
    ProjectCreationLease,
};
use super::super::ProjectAuthorityError;
use super::temp_root;

#[test]
fn project_creation_lease_serializes_the_same_canonical_target_until_drop() {
    let root = temp_root("creation-lease");
    let target = root.join("project");
    let first = ProjectCreationLease::acquire(&target).unwrap();

    assert!(matches!(
        ProjectCreationLease::acquire(&target),
        Err(ProjectAuthorityError::TargetCreationLeaseHeld { ref path }) if path == &target
    ));

    drop(first);
    ProjectCreationLease::acquire(&target).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[cfg(any(windows, target_os = "linux"))]
#[test]
fn project_creation_lease_does_not_serialize_distinct_targets() {
    let root = temp_root("creation-lease-distinct-targets");
    let first = ProjectCreationLease::acquire(&root.join("first")).unwrap();
    let second = ProjectCreationLease::acquire(&root.join("second")).unwrap();

    drop((first, second));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn project_creation_lease_uses_windows_case_insensitive_target_identity() {
    let root = temp_root("creation-lease-case");
    let first_target = root.join("Project");
    let same_target = root.join("project");
    let first = ProjectCreationLease::acquire(&first_target).unwrap();

    assert!(matches!(
        ProjectCreationLease::acquire(&same_target),
        Err(ProjectAuthorityError::TargetCreationLeaseHeld { ref path }) if path == &same_target
    ));

    drop(first);
    ProjectCreationLease::acquire(&same_target).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn project_creation_lease_identity_survives_parent_directory_replacement() {
    let root = temp_root("creation-lease-parent-replacement");
    let moved_root = root.with_extension("moved");
    let target = root.join("project");
    let first = ProjectCreationLease::acquire(&target).unwrap();
    fs::rename(&root, &moved_root).unwrap();
    fs::create_dir(&root).unwrap();

    assert!(matches!(
        ProjectCreationLease::acquire(&target),
        Err(ProjectAuthorityError::TargetCreationLeaseHeld { ref path }) if path == &target
    ));

    drop(first);
    ProjectCreationLease::acquire(&target).unwrap();
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(moved_root).unwrap();
}

#[test]
fn target_that_becomes_non_empty_before_commit_is_restored_without_publishing() {
    let root = temp_root("commit-concurrent-target-write");
    let target = root.join("project");
    let staging = root.join("staging");
    let backup = root.join("backup");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("caller-owned.txt"), "retain").unwrap();
    fs::create_dir(&staging).unwrap();
    fs::write(staging.join("zircon-project.toml"), "staged").unwrap();

    let error = commit_staged_directory(&staging, &target, &backup, true, |from, to| {
        fs::rename(from, to)
    })
    .unwrap_err();

    assert!(matches!(
        error,
        ProjectAuthorityError::TargetNotEmpty { ref path } if path == &target
    ));
    assert_eq!(
        fs::read_to_string(target.join("caller-owned.txt")).unwrap(),
        "retain"
    );
    assert!(
        staging.join("zircon-project.toml").is_file(),
        "the staged project must not publish over the changed target"
    );
    assert!(!backup.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn published_project_finalization_failure_preserves_target_and_backup_paths() {
    let root = temp_root("published-finalization-failure");
    let target = root.join("project");
    let backup = root.join("backup");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("published-project"), "published").unwrap();
    fs::create_dir(&backup).unwrap();
    fs::write(backup.join("caller-owned.txt"), "retain").unwrap();

    let error = finalize_published_project(&target, &backup, true).unwrap_err();

    match &error {
        ProjectAuthorityError::PublishedProjectFinalizationFailed {
            target: published_target,
            backup: recovery_backup,
            source,
        } => {
            assert_eq!(published_target, &target);
            assert_eq!(recovery_backup, &backup);
            assert!(matches!(
                source.as_ref(),
                ProjectAuthorityError::TargetNotEmpty { .. }
            ));
        }
        other => panic!("unexpected finalization error: {other}"),
    }
    assert!(target.join("published-project").is_file());
    assert_eq!(
        fs::read_to_string(backup.join("caller-owned.txt")).unwrap(),
        "retain"
    );
    fs::remove_dir_all(root).unwrap();
}

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
