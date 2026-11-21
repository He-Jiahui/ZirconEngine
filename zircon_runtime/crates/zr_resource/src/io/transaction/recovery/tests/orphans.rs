//! Only the compact journal-staging class joins the existing bounded orphan recovery path.

use std::num::NonZeroU64;

use super::super::super::pathing::{journal_owner_token, journal_path, valid_tag};
use super::*;
use crate::io::atomic_file::{is_compact_journal_staging_path, sibling_file_name_for_test};

fn maximum_journal_target(root: &Path) -> (PathBuf, String) {
    let tag = "a".repeat(32);
    assert!(valid_tag(&tag));
    let owner = fs::canonicalize(root).unwrap();
    let transaction_id = format!("{}-{}-{}", journal_owner_token(&owner), u32::MAX, u64::MAX);
    let target = journal_path(
        &owner,
        &owner.join("registry.zrproject"),
        &tag,
        &transaction_id,
    );
    assert_eq!(target.file_name().unwrap().to_str().unwrap().len(), 216);
    (target, tag)
}

fn maximum_atomic_name(target: &Path, role: &str) -> String {
    sibling_file_name_for_test(target, role, u32::MAX, NonZeroU64::new(u64::MAX).unwrap()).unwrap()
}

#[test]
fn maximum_tag_and_full_wire_compact_journal_staging_is_reported_then_removed() {
    let root = test_directory("compact-max-wire-orphan");
    fs::create_dir_all(&root).unwrap();
    let (target, tag) = maximum_journal_target(&root);
    fs::write(&target, b"ordinary maximum journal leaf preflight").unwrap();
    assert_eq!(
        fs::read(&target).unwrap(),
        b"ordinary maximum journal leaf preflight"
    );
    fs::remove_file(&target).unwrap();
    let name = maximum_atomic_name(&target, "staging");
    assert_eq!(name.len(), 119);
    assert!(name.ends_with(&format!(".zrjournal.zr2-staging-{}-{}", u32::MAX, u64::MAX)));
    let orphan = root.join(&name);
    assert!(is_compact_journal_staging_path(&orphan));
    assert!(crate::io::is_atomic_write_transaction_path(&orphan));
    fs::write(&orphan, b"torn compact intent").unwrap();
    let resolved_orphan = fs::canonicalize(&root).unwrap().join(&name);
    let mut policy = AcceptNothing;

    assert_eq!(
        detect_pending_transactions(&root, &tag, &mut policy).unwrap(),
        vec![resolved_orphan]
    );
    assert_eq!(fs::read(&orphan).unwrap(), b"torn compact intent");
    recover_pending_transactions(&root, &tag, &mut policy).unwrap();

    assert!(!orphan.exists());
    assert!(!target.exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generic_wrong_role_or_malformed_compact_artifact_remains_unsupported_and_untouched() {
    for case in [
        "generic-staging",
        "generic-backup",
        "journal-target-backup",
        "journal-class-backup",
        "wrong-role",
        "bad-digest",
        "short-digest",
        "noncanonical-pid",
        "overflow-pid",
        "zero-sequence",
        "noncanonical-sequence",
        "overflow-sequence",
        "extra-suffix",
    ] {
        let root = test_directory(case);
        fs::create_dir_all(&root).unwrap();
        let (target, tag) = maximum_journal_target(&root);
        let valid_name = maximum_atomic_name(&target, "staging");
        let data_target = root.join("d".repeat(255));
        let unsupported_name = match case {
            "generic-staging" => maximum_atomic_name(&data_target, "staging"),
            "generic-backup" => maximum_atomic_name(&data_target, "backup"),
            "journal-target-backup" => maximum_atomic_name(&target, "backup"),
            "journal-class-backup" => valid_name.replace(".zr2-staging-", ".zr2-backup-"),
            "wrong-role" => valid_name.replace(".zr2-staging-", ".zr2-rollback-"),
            "bad-digest" => format!(".g{}", &valid_name[2..]),
            "short-digest" => format!(".{}", &valid_name[2..]),
            "noncanonical-pid" => valid_name.replace("-4294967295-", "-04294967295-"),
            "overflow-pid" => valid_name.replace("-4294967295-", "-4294967296-"),
            "zero-sequence" => valid_name.replace("-18446744073709551615", "-0"),
            "noncanonical-sequence" => valid_name.replace("-18446744073709551615", "-01"),
            "overflow-sequence" => {
                valid_name.replace("-18446744073709551615", "-18446744073709551616")
            }
            "extra-suffix" => format!("{valid_name}.extra"),
            _ => unreachable!(),
        };
        let valid = root.join(valid_name);
        let unsupported = root.join(unsupported_name);
        assert!(!is_compact_journal_staging_path(&unsupported), "{case}");
        fs::write(
            &valid,
            b"valid compact orphan must survive an invalid batch",
        )
        .unwrap();
        fs::write(&unsupported, b"unsupported evidence must not be cleaned").unwrap();
        let mut policy = AcceptNothing;

        let error = detect_pending_transactions(&root, &tag, &mut policy)
            .expect_err("unknown compact artifacts are not journal orphans");
        assert!(
            matches!(error, DurableTransactionError::InvalidJournal { reason, .. }
            if reason.contains("unsupported file")),
            "{case}"
        );
        let error = recover_pending_transactions(&root, &tag, &mut policy)
            .expect_err("an invalid batch must preserve even the valid compact orphan");
        assert!(
            matches!(error, DurableTransactionError::InvalidJournal { reason, .. }
            if reason.contains("unsupported file")),
            "{case}"
        );

        assert_eq!(
            fs::read(&valid).unwrap(),
            b"valid compact orphan must survive an invalid batch",
            "{case}"
        );
        assert_eq!(
            fs::read(&unsupported).unwrap(),
            b"unsupported evidence must not be cleaned",
            "{case}"
        );
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2, "{case}");
        fs::remove_dir_all(root).unwrap();
    }
}
