use std::fs;
use std::io::ErrorKind;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use super::test_root;
use crate::io::atomic_file::{
    atomic_write_with_fault, recover_missing_target_from_backup, stage_atomic_write,
    AtomicWriteFault,
};

const TARGET: &str = "config.json";
const OTHER_TARGET: &str = "config.json.zr-backup-other";
const BACKUP: &str = ".config.json.zr-backup-42-1";
const SAVED: &[u8] = br#"{"value":"saved"}"#;
const CURRENT: &[u8] = br#"{"value":"current"}"#;
const FOREIGN: &[u8] = br#"{"value":"foreign"}"#;

#[test]
fn recovery_ignores_foreign_staging_singleton() {
    let root = create_root("zr_owner_foreign_stage");
    let target = root.join(TARGET);
    let other = root.join(OTHER_TARGET);
    let pending = stage_atomic_write(&other, FOREIGN).expect("stage the other target");
    let staging = produced_foreign_sibling(&other, "staging");
    assert_file(&staging, FOREIGN);
    assert_missing(&target);
    assert_missing(&other);

    assert_no_recovery(&target, &staging, FOREIGN);
    assert_missing(&other);

    // Only the pending writer may clean up the staging file it still owns.
    drop(pending);
    assert_missing(&staging);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_ignores_foreign_backup_singleton() {
    let root = create_root("zr_owner_foreign_backup");
    let target = root.join(TARGET);
    let other = root.join(OTHER_TARGET);
    write_and_read(&other, FOREIGN);
    let error = atomic_write_with_fault(&other, CURRENT, AtomicWriteFault::ParentSync)
        .expect_err("the foreign write must fail at the injected post-publication barrier");
    assert_eq!(error.kind(), ErrorKind::Other);
    assert_eq!(
        error.to_string(),
        "injected atomic file parent-directory sync failure"
    );
    assert_file(&other, CURRENT);
    let backup = produced_foreign_sibling(&other, "backup");
    assert_file(&backup, FOREIGN);
    assert_missing(&target);

    assert_no_recovery(&target, &backup, FOREIGN);
    assert_file(&other, CURRENT);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_restores_one_same_owner_backup() {
    // Legacy PID spelling is unbounded; sequence digits can have leading zeros.
    for tail in ["42-1", "00018446744073709551616-0007"] {
        let root = create_root("zr_owner_unique");
        let target = root.join(TARGET);
        let backup = root.join(format!(".{TARGET}.zr-backup-{tail}"));
        write_and_read(&backup, SAVED);
        assert_missing(&target);

        assert!(recover_missing_target_from_backup(&target).unwrap());

        assert_file(&target, SAVED);
        assert_missing(&backup);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn recovery_rejects_two_same_owner_backups() {
    let root = create_root("zr_owner_ambiguous");
    let target = root.join(TARGET);
    let first = root.join(BACKUP);
    let second = root.join(".config.json.zr-backup-42-2");
    write_and_read(&first, SAVED);
    write_and_read(&second, CURRENT);
    assert_missing(&target);

    let error = recover_missing_target_from_backup(&target)
        .expect_err("two same-owner backups must be ambiguous");
    assert_eq!(error.kind(), ErrorKind::InvalidData);

    assert_missing(&target);
    assert_file(&first, SAVED);
    assert_file(&second, CURRENT);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_preserves_existing_target_and_backup() {
    let root = create_root("zr_owner_existing");
    let target = root.join(TARGET);
    let backup = root.join(BACKUP);
    write_and_read(&target, CURRENT);
    write_and_read(&backup, SAVED);

    assert!(!recover_missing_target_from_backup(&target).unwrap());

    assert_file(&target, CURRENT);
    assert_file(&backup, SAVED);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_restores_only_same_owner_backup_when_foreign_is_present() {
    let root = create_root("zr_owner_mixed");
    let target = root.join(TARGET);
    let backup = root.join(BACKUP);
    let foreign = root.join(format!(".{OTHER_TARGET}.zr-backup-42-2"));
    write_and_read(&backup, SAVED);
    write_and_read(&foreign, FOREIGN);
    assert_missing(&target);

    let result = recover_missing_target_from_backup(&target);
    assert!(
        matches!(&result, Ok(true)),
        "a foreign sibling must not make the same-owner backup ambiguous: {result:?}"
    );

    assert_file(&target, SAVED);
    assert_missing(&backup);
    assert_file(&foreign, FOREIGN);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_ignores_invalid_legacy_backup_numbers() {
    for tail in ["x-1", "42-0", "42-18446744073709551616"] {
        let root = create_root("zr_owner_invalid");
        let target = root.join(TARGET);
        let invalid = root.join(format!(".{TARGET}.zr-backup-{tail}"));
        write_and_read(&invalid, SAVED);
        assert_missing(&target);

        assert_no_recovery(&target, &invalid, SAVED);
        fs::remove_dir_all(root).unwrap();
    }
}

fn create_root(name: &str) -> PathBuf {
    let root = test_root(name);
    fs::create_dir_all(root.parent().unwrap()).unwrap();
    fs::create_dir(&root).unwrap();
    root
}

fn produced_foreign_sibling(other: &Path, role: &str) -> PathBuf {
    // Exclude only the other target's data path, not any transaction-looking files.
    let mut siblings = fs::read_dir(other.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.as_path() != other);
    let candidate = siblings.next().expect("one foreign artifact on disk");
    assert!(siblings.next().is_none(), "unexpected extra sibling");
    let leaf = candidate.file_name().unwrap().to_str().unwrap();
    let prefix = format!(".{OTHER_TARGET}.zr-{role}-{}-", std::process::id());
    let sequence = leaf
        .strip_prefix(&prefix)
        .expect("exact legacy foreign owner, role and PID");
    assert!(
        sequence.bytes().all(|byte| byte.is_ascii_digit())
            && sequence.parse::<NonZeroU64>().is_ok(),
        "invalid legacy sequence in {leaf}"
    );
    candidate
}

fn write_and_read(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    assert_file(path, bytes);
}

fn assert_file(path: &Path, bytes: &[u8]) {
    let name = path.display();
    let metadata =
        fs::symlink_metadata(path).unwrap_or_else(|error| panic!("stat {name}: {error}"));
    assert!(metadata.is_file(), "{name} is not a regular file");
    let actual = fs::read(path).unwrap_or_else(|error| panic!("read {name}: {error}"));
    assert_eq!(actual.as_slice(), bytes, "contents of {name}");
}

fn assert_missing(path: &Path) {
    let metadata = fs::symlink_metadata(path);
    assert!(
        matches!(&metadata, Err(error) if error.kind() == ErrorKind::NotFound),
        "expected NotFound for {}: {metadata:?}",
        path.display()
    );
}

fn assert_no_recovery(target: &Path, candidate: &Path, bytes: &[u8]) {
    let result = recover_missing_target_from_backup(target);
    assert!(
        matches!(&result, Ok(false)),
        "recovery must ignore {}: {result:?}; target read: {:?}; candidate read: {:?}",
        candidate.display(),
        fs::read(target),
        fs::read(candidate)
    );
    assert_missing(target);
    assert_file(candidate, bytes);
}

#[test]
fn recovery_restores_backup_for_owner_with_reserved_marker() {
    let root = create_root("zr_owner_marker");
    let target = root.join(OTHER_TARGET);
    let backup = root.join(format!(".{OTHER_TARGET}.zr-backup-42-1"));
    write_and_read(&backup, SAVED);
    assert_missing(&target);

    assert!(recover_missing_target_from_backup(&target).unwrap());

    assert_file(&target, SAVED);
    assert_missing(&backup);
    fs::remove_dir_all(root).unwrap();
}
