//! Compact recovery is exact spelling/role matching, not a new authority for legacy prefixes.

use super::super::pathing::{native_leaf_token, unique_sibling_path};
use super::{fs, recover_missing_target_from_backup, test_root};

#[test]
fn compact_backup_restores_the_exact_complete_long_target() {
    let root = test_root("compact_exact_restore");
    fs::create_dir_all(&root).unwrap();
    let target = root.join(format!("{}.json", "a".repeat(250)));
    fs::write(&target, b"ordinary preflight").unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"ordinary preflight");
    fs::remove_file(&target).unwrap();
    let backup = unique_sibling_path(&root, &target, "backup").unwrap();
    assert!(backup
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .contains(".zr2-backup-"));
    fs::write(&backup, b"exact old generation").unwrap();

    assert!(recover_missing_target_from_backup(&target).unwrap());

    assert_eq!(fs::read(&target).unwrap(), b"exact old generation");
    assert!(!backup.exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn legacy_early_nonce_restores_long_ascii_and_utf16_fit_names() {
    for leaf in [
        format!("{}.json", "a".repeat(235)),
        format!("{}.json", "é".repeat(235)),
    ] {
        let root = test_root("legacy_early_nonce_restore");
        fs::create_dir_all(&root).unwrap();
        let target = root.join(&leaf);
        fs::write(&target, b"ordinary preflight").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"ordinary preflight");
        fs::remove_file(&target).unwrap();
        let backup_leaf = format!(".{leaf}.zr-backup-1-1");
        assert_eq!(backup_leaf.encode_utf16().count(), 255);
        let backup = root.join(backup_leaf);
        fs::write(&backup, b"legacy early generation").unwrap();

        assert!(recover_missing_target_from_backup(&target).unwrap());

        assert_eq!(fs::read(&target).unwrap(), b"legacy early generation");
        assert!(!backup.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn legacy_and_compact_true_backups_remain_ambiguous_and_untouched() {
    let root = test_root("mixed_scheme_ambiguity");
    fs::create_dir_all(&root).unwrap();
    // This leaf fits v1's early nonce but requires compact naming at the full wire width.
    let leaf = format!("{}.json", "a".repeat(215));
    let target = root.join(&leaf);
    let legacy = root.join(format!(".{leaf}.zr-backup-1-1"));
    let compact = root.join(format!(
        ".{}.zr2-backup-{}-{}",
        native_leaf_token(&target).unwrap(),
        u32::MAX,
        u64::MAX
    ));
    fs::write(&legacy, b"legacy recovery evidence").unwrap();
    fs::write(&compact, b"compact recovery evidence").unwrap();

    let error = recover_missing_target_from_backup(&target)
        .expect_err("the scan must collect both true schemes, not choose the current budget");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("2 backup candidates"));
    assert!(!target.exists());
    assert_eq!(fs::read(&legacy).unwrap(), b"legacy recovery evidence");
    assert_eq!(fs::read(&compact).unwrap(), b"compact recovery evidence");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn multiple_compact_true_backups_remain_ambiguous_and_untouched() {
    let root = test_root("compact_scheme_ambiguity");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("a".repeat(255));
    let first = unique_sibling_path(&root, &target, "backup").unwrap();
    fs::write(&first, b"first compact evidence").unwrap();
    let second = unique_sibling_path(&root, &target, "backup").unwrap();
    fs::write(&second, b"second compact evidence").unwrap();

    let error = recover_missing_target_from_backup(&target)
        .expect_err("recovery must not choose between two exact compact backups");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("2 backup candidates"));
    assert!(!target.exists());
    assert_eq!(fs::read(&first).unwrap(), b"first compact evidence");
    assert_eq!(fs::read(&second).unwrap(), b"second compact evidence");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compact_backup_for_same_prefix_other_long_target_is_not_recovered() {
    let root = test_root("compact_distinct_native_owners");
    fs::create_dir_all(&root).unwrap();
    let prefix = "a".repeat(254);
    let first = root.join(format!("{prefix}x"));
    let second = root.join(format!("{prefix}y"));
    let backup = unique_sibling_path(&root, &first, "backup").unwrap();
    fs::write(&backup, b"first target only").unwrap();

    assert!(!recover_missing_target_from_backup(&second).unwrap());
    assert!(!first.exists());
    assert!(!second.exists());
    assert_eq!(fs::read(&backup).unwrap(), b"first target only");
    assert!(recover_missing_target_from_backup(&first).unwrap());
    assert_eq!(fs::read(&first).unwrap(), b"first target only");
    assert!(!second.exists());
    assert!(!backup.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn literal_digest_short_target_and_compact_owner_cannot_cross_schemes() {
    let root = test_root("compact_literal_digest_isolation");
    fs::create_dir_all(&root).unwrap();
    let long_target = root.join("a".repeat(255));
    let owner = native_leaf_token(&long_target).unwrap();
    let short_target = root.join(&owner);
    let compact = unique_sibling_path(&root, &long_target, "backup").unwrap();
    fs::write(&compact, b"long source generation").unwrap();

    assert!(!recover_missing_target_from_backup(&short_target).unwrap());
    assert!(!short_target.exists());
    assert_eq!(fs::read(&compact).unwrap(), b"long source generation");

    let legacy = root.join(format!(".{owner}.zr-backup-42-1"));
    fs::write(&legacy, b"literal short source generation").unwrap();
    assert!(recover_missing_target_from_backup(&long_target).unwrap());
    assert_eq!(fs::read(&long_target).unwrap(), b"long source generation");
    assert_eq!(
        fs::read(&legacy).unwrap(),
        b"literal short source generation"
    );
    assert!(!short_target.exists());
    assert!(recover_missing_target_from_backup(&short_target).unwrap());
    assert_eq!(
        fs::read(&short_target).unwrap(),
        b"literal short source generation"
    );
    assert!(!legacy.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compact_staging_wrong_role_class_or_malformed_nonce_never_restores() {
    let root = test_root("compact_recovery_rejects_shapes");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("a".repeat(255));
    let owner = native_leaf_token(&target).unwrap();
    let names = [
        format!(".{owner}.zr2-staging-42-1"),
        format!(".{owner}.zrjournal.zr2-staging-42-1"),
        format!(".{owner}.zrjournal.zr2-backup-42-1"),
        format!(".{owner}.zr2-rollback-42-1"),
        format!(".{owner}.zr2-backup-042-1"),
        format!(".{owner}.zr2-backup-4294967296-1"),
        format!(".{owner}.zr2-backup-42-0"),
        format!(".{owner}.zr2-backup-42-01"),
        format!(".{owner}.zr2-backup-42-18446744073709551616"),
        format!(".{owner}.zr2-backup-42-1.extra"),
        format!(".{}.zr2-backup-42-1", owner.to_uppercase()),
    ];
    for name in &names {
        fs::write(root.join(name), name.as_bytes()).unwrap();
    }

    assert!(!recover_missing_target_from_backup(&target).unwrap());

    assert!(!target.exists());
    for name in &names {
        assert_eq!(fs::read(root.join(name)).unwrap(), name.as_bytes());
    }
    assert_eq!(fs::read_dir(&root).unwrap().count(), names.len());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn matching_compact_non_file_backup_is_rejected_without_mutation() {
    let root = test_root("compact_recovery_non_file");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("a".repeat(255));
    let backup = unique_sibling_path(&root, &target, "backup").unwrap();
    fs::create_dir(&backup).unwrap();

    let error = recover_missing_target_from_backup(&target)
        .expect_err("exact compact ownership does not authorize publishing a non-file");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(!target.exists());
    assert!(backup.is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn existing_target_keeps_compact_backup_evidence_untouched() {
    let root = test_root("compact_recovery_existing_target");
    fs::create_dir_all(&root).unwrap();
    let target = root.join("a".repeat(255));
    let backup = unique_sibling_path(&root, &target, "backup").unwrap();
    fs::write(&target, b"live generation").unwrap();
    fs::write(&backup, b"retained recovery evidence").unwrap();

    assert!(!recover_missing_target_from_backup(&target).unwrap());

    assert_eq!(fs::read(&target).unwrap(), b"live generation");
    assert_eq!(fs::read(&backup).unwrap(), b"retained recovery evidence");
    fs::remove_dir_all(root).unwrap();
}
