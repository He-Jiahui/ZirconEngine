use std::fs;

use super::*;

#[test]
fn journal_owners_cannot_delete_each_others_same_target_artifacts() {
    let root = super::test_directory("cross-owner-artifact-partition");
    let first_owner = root.join("first-journal");
    let second_owner = root.join("second-journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&first_owner).unwrap();
    fs::create_dir_all(&second_owner).unwrap();
    let first_transaction_id =
        super::super::super::pathing::transaction_id_for_test(&first_owner, 1);
    let second_transaction_id =
        super::super::super::pathing::transaction_id_for_test(&second_owner, 1);
    let first_staging = super::super::super::pathing::transaction_sibling(
        &target,
        "project",
        "stage",
        &first_transaction_id,
    );
    let second_staging = super::super::super::pathing::transaction_sibling(
        &target,
        "project",
        "stage",
        &second_transaction_id,
    );
    fs::write(&first_staging, b"first-owner-staging").unwrap();
    let mut report = DurableCommitReport::default();

    let _ = commit_prepared_files(
        &second_owner,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"second-owner-publication".to_vec(),
        )],
        TransactionFault::None,
        &mut report,
    )
    .expect("the second owner publishes only through its own artifact namespace");

    assert_ne!(first_transaction_id, second_transaction_id);
    assert_ne!(first_staging, second_staging);
    assert_eq!(fs::read(&first_staging).unwrap(), b"first-owner-staging");
    assert_eq!(fs::read(&target).unwrap(), b"second-owner-publication");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn intent_rejects_generated_artifact_alias_before_journal_creation() {
    let root = super::test_directory("artifact-alias-preflight");
    fs::create_dir_all(&root).unwrap();
    let first = root.join("first.zmeta");
    let transaction_id = super::super::super::pathing::transaction_id_for_test(&root, 1);
    let colliding_target = super::super::super::pathing::transaction_sibling(
        &first,
        "project",
        "stage",
        &transaction_id,
    );

    let error = create_intent(
        &root,
        "project",
        &transaction_id,
        &[
            PreparedFileWrite::new(first, b"first".to_vec()),
            PreparedFileWrite::new(colliding_target, b"second".to_vec()),
        ],
    )
    .expect_err("all live and generated paths must be unique before persistence");

    assert!(error.to_string().contains("live and artifact paths alias"));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn intent_rejects_generated_artifact_ancestor_before_journal_creation() {
    let root = super::test_directory("artifact-ancestor-preflight");
    fs::create_dir_all(&root).unwrap();
    let first = root.join("first.zmeta");
    let transaction_id = super::super::super::pathing::transaction_id_for_test(&root, 1);
    let artifact_parent = super::super::super::pathing::transaction_sibling(
        &first,
        "project",
        "stage",
        &transaction_id,
    );
    let descendant_target = artifact_parent.join("child.zmeta");

    let error = create_intent(
        &root,
        "project",
        &transaction_id,
        &[
            PreparedFileWrite::new(first, b"first".to_vec()),
            PreparedFileWrite::new(descendant_target, b"second".to_vec()),
        ],
    )
    .expect_err("all generated files must form an antichain before persistence");

    assert!(error.to_string().contains("ancestor or descendant"));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn commit_rejects_owner_lock_collision_before_creating_the_lock() {
    let root = super::test_directory("owner-lock-alias-preflight");
    fs::create_dir_all(&root).unwrap();
    let owner_lock =
        super::super::super::owner_lock::owner_lock_path(&root).expect("owner lock path");
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &root,
        "project",
        vec![PreparedFileWrite::new(owner_lock.clone(), b"live".to_vec())],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("live target must not alias the transaction owner lock");

    assert!(error.to_string().contains("owner lock namespace"));
    assert!(!owner_lock.exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn commit_rejects_retirement_owner_lock_collision_before_creating_the_lock() {
    let root = super::test_directory("retirement-owner-lock-alias-preflight");
    let journal = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&root).unwrap();
    let owner_lock =
        super::super::super::owner_lock::owner_lock_path(&journal).expect("owner lock path");
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(target, b"live".to_vec()).retiring(owner_lock.clone())],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("retirement must not alias the transaction owner lock");

    assert!(error.to_string().contains("owner lock namespace"));
    assert!(!owner_lock.exists());
    assert!(!journal.exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn commit_rejects_non_utf8_target_before_creating_the_journal_owner() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::ffi::OsStringExt;

    let root = super::test_directory("non-utf8-target-preflight");
    fs::create_dir_all(&root).unwrap();
    let mut target = OsString::from_vec(root.as_os_str().as_bytes().to_vec());
    target.push(OsString::from_vec(vec![b'/', b'a', 0x80]));
    let target = std::path::PathBuf::from(target);
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &root.join("journal"),
        "project",
        vec![PreparedFileWrite::new(target, b"live".to_vec())],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("non-UTF-8 paths are outside the TOML journal wire contract");

    assert!(error.to_string().contains("UTF-8 encodable"));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn commit_rejects_unpaired_surrogate_target_before_creating_the_journal_owner() {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::ffi::OsStringExt;

    let root = super::test_directory("unpaired-surrogate-target-preflight");
    fs::create_dir_all(&root).unwrap();
    let mut units = root.as_os_str().encode_wide().collect::<Vec<_>>();
    units.extend([u16::from(b'\\'), 0xd800]);
    let target = std::path::PathBuf::from(std::ffi::OsString::from_wide(&units));
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &root.join("journal"),
        "project",
        vec![PreparedFileWrite::new(target, b"live".to_vec())],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("unpaired UTF-16 paths are outside the TOML journal wire contract");

    assert!(error.to_string().contains("UTF-8 encodable"));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}
