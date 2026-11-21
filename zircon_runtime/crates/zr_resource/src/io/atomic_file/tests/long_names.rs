//! Source-derived #159 regressions, not a port of an existing UE long-name test.
//! UE SaveConfigFileWrapper bounds its temporary prefix and retains the target separately;
//! the filesystem publication and durability contracts here remain Zircon's own.

use std::path::PathBuf;

use super::{atomic_write_new, atomic_write_with_fault, fs, test_root, AtomicWriteFault};

fn existing_long_leaf(label: &str, leaf: &str) -> (PathBuf, PathBuf) {
    assert_eq!(leaf.len(), 255, "fixture must exercise the byte boundary");
    assert!(leaf.encode_utf16().count() <= 255);
    let root = test_root(label);
    assert!(!root.exists(), "each test must own a fresh directory");
    fs::create_dir_all(&root).unwrap();
    let target = root.join(leaf);
    // A direct create/read proves the final leaf is legal on the actual test volume.
    // A failure here is an environment prerequisite, not the atomic-writer RED.
    fs::write(&target, b"original generation")
        .expect("the volume must admit the final long filename as a regular file");
    assert_eq!(fs::read(&target).unwrap(), b"original generation");
    (root, target)
}

fn ascii_leaf() -> String {
    format!("{}.json", "a".repeat(250))
}

#[test]
fn atomic_long_leaf_new_publication_preserves_a_legal_final_name() {
    let (root, target) = existing_long_leaf("atomic_long_leaf_new", &ascii_leaf());
    fs::remove_file(&target).unwrap();
    assert!(!target.exists());

    let result = atomic_write_new(&target, b"published generation");

    assert!(
        result.is_ok(),
        "atomic publication of {target:?}: {result:?}"
    );
    assert_eq!(fs::read(&target).unwrap(), b"published generation");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_long_leaf_missing_target_uses_the_normal_commit_path() {
    let (root, target) = existing_long_leaf("atomic_long_leaf_missing_commit", &ascii_leaf());
    fs::remove_file(&target).unwrap();
    assert!(!target.exists());

    let result =
        atomic_write_with_fault(&target, b"normal commit generation", AtomicWriteFault::None);

    assert!(
        result.is_ok(),
        "normal missing-target commit of {target:?}: {result:?}"
    );
    assert_eq!(fs::read(&target).unwrap(), b"normal commit generation");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_long_leaf_replacement_preserves_a_legal_final_name() {
    let (root, target) = existing_long_leaf("atomic_long_leaf_replace", &ascii_leaf());

    let result =
        atomic_write_with_fault(&target, b"replacement generation", AtomicWriteFault::None);

    assert!(
        result.is_ok(),
        "atomic replacement of {target:?}: {result:?}"
    );
    assert_eq!(fs::read(&target).unwrap(), b"replacement generation");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_long_leaf_multibyte_name_is_not_truncated_or_retargeted() {
    // 255 UTF-8 bytes and 245 UTF-16 units, including surrogate pairs on Windows.
    let leaf = format!("{}{}.json", "\u{1f980}".repeat(5), "a".repeat(230));
    let (root, target) = existing_long_leaf("atomic_long_leaf_unicode", &leaf);

    let result = atomic_write_with_fault(&target, b"unicode generation", AtomicWriteFault::None);

    assert!(
        result.is_ok(),
        "atomic replacement of {target:?}: {result:?}"
    );
    assert_eq!(fs::read(&target).unwrap(), b"unicode generation");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_long_leaf_write_fault_keeps_original_and_cleans_owned_staging() {
    let (root, target) = existing_long_leaf("atomic_long_leaf_write_fault", &ascii_leaf());

    let error = atomic_write_with_fault(&target, b"unpublished", AtomicWriteFault::Write)
        .expect_err("the write fault must still be reached after staging allocation");

    assert!(
        error
            .to_string()
            .contains("injected atomic file write failure"),
        "the regression must reach the injected write, not fail to name staging: {error}"
    );
    assert_eq!(fs::read(&target).unwrap(), b"original generation");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn atomic_long_leaf_replace_fault_restores_the_original_target() {
    let (root, target) = existing_long_leaf("atomic_long_leaf_replace_fault", &ascii_leaf());

    let error = atomic_write_with_fault(
        &target,
        b"unpublished",
        AtomicWriteFault::ReplaceAfterBackup,
    )
    .expect_err("the replace-after-backup fault must remain a recoverable error");

    assert!(
        error.to_string().contains("restored from backup"),
        "the regression must reach the real backup/restore path: {error}"
    );
    assert_eq!(fs::read(&target).unwrap(), b"original generation");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn atomic_long_leaf_new_publication_rejects_existing_without_replacing_original() {
    let (root, target) = existing_long_leaf("atomic_long_leaf_new_existing", &ascii_leaf());

    let error = atomic_write_new(&target, b"must not replace")
        .expect_err("the bounded staging name must not weaken no-clobber publication");

    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&target).unwrap(), b"original generation");
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}
