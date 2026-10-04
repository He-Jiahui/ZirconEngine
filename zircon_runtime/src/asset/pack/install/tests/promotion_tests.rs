use std::fs;
use std::path::PathBuf;

use super::*;
use crate::asset::pack::{ZrPackDeltaInstaller, ZrPackDeltaWriter, ZrPackInputAsset, ZrPackWriter};

struct Fixture {
    root: PathBuf,
    staged: PathBuf,
    installed: PathBuf,
    backup: PathBuf,
    old: Vec<u8>,
    new: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-pack-recovery-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let staged = root.join("staging/pack.zrpack");
        let installed = root.join("installed/pack.zrpack");
        let backup = root.join("backup/pack.zrpack");
        fs::create_dir_all(staged.parent().unwrap()).unwrap();
        fs::create_dir_all(installed.parent().unwrap()).unwrap();
        fs::create_dir_all(backup.parent().unwrap()).unwrap();
        let pack = |bytes: &[u8]| {
            ZrPackWriter::write([ZrPackInputAsset::new("data/value.bin", bytes.to_vec())])
                .unwrap()
                .bytes
        };
        let old = pack(b"old");
        let new = pack(b"new");
        fs::write(&installed, &old).unwrap();
        fs::write(&staged, &new).unwrap();
        fs::write(&backup, b"prior backup").unwrap();
        Self {
            root,
            staged,
            installed,
            backup,
            old,
            new,
        }
    }

    fn recover(&self, with_backup: bool) {
        ZrPackDeltaInstaller::recover_pending_promotion(
            &self.staged,
            &self.installed,
            with_backup.then_some(&self.backup),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn pack_promotion_failure_without_backup_preserves_installed_bytes() {
    let fixture = Fixture::new();
    let error = promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        None,
        TransactionFault::BeforeCommit(0),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ZrPackDeltaInstallError::TransactionFailed { .. }
    ));
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.old);
    assert_eq!(fs::read(&fixture.staged).unwrap(), fixture.new);
    fixture.recover(false);
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.old);
}

#[test]
fn pack_promotion_restart_rolls_back_uncommitted_replacement_and_staging_retirement() {
    for with_backup in [false, true] {
        let installed_index = usize::from(with_backup);
        for fault in [
            TransactionFault::CrashAfterStaging(0),
            TransactionFault::CrashAfterTargetReplace(installed_index),
            TransactionFault::CrashAfterRetiredDelete(installed_index),
            TransactionFault::CrashAfterCommit(installed_index),
        ] {
            let fixture = Fixture::new();
            assert!(promote_with_fault(
                &fixture.staged,
                &fixture.installed,
                with_backup.then_some(fixture.backup.as_path()),
                fault
            )
            .is_err());
            fixture.recover(with_backup);
            assert_eq!(
                fs::read(&fixture.installed).unwrap(),
                fixture.old,
                "{fault:?}"
            );
            assert_eq!(fs::read(&fixture.staged).unwrap(), fixture.new, "{fault:?}");
            assert_eq!(
                fs::read(&fixture.backup).unwrap(),
                b"prior backup",
                "{fault:?}"
            );
            fixture.recover(with_backup);
        }
    }
}

#[test]
fn pack_promotion_restart_finishes_committed_publication() {
    for with_backup in [false, true] {
        for fault in [
            TransactionFault::CrashAfterAllCommitted,
            TransactionFault::CrashAfterCleanup,
        ] {
            let fixture = Fixture::new();
            assert!(promote_with_fault(
                &fixture.staged,
                &fixture.installed,
                with_backup.then_some(fixture.backup.as_path()),
                fault
            )
            .is_err());
            fixture.recover(with_backup);
            assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
            assert!(!fixture.staged.exists());
            if with_backup {
                assert_eq!(fs::read(&fixture.backup).unwrap(), fixture.old);
            }
            fixture.recover(with_backup);
        }
    }
}

#[test]
fn pack_promotion_rejects_changed_recovery_authority_without_touching_files() {
    let fixture = Fixture::new();
    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        Some(&fixture.backup),
        TransactionFault::CrashAfterTargetReplace(1)
    )
    .is_err());
    let installed_before = fs::read(&fixture.installed).unwrap();
    let backup_before = fs::read(&fixture.backup).unwrap();
    assert!(ZrPackDeltaInstaller::recover_pending_promotion(
        &fixture.staged,
        &fixture.installed,
        None::<&Path>
    )
    .is_err());
    assert_eq!(fs::read(&fixture.installed).unwrap(), installed_before);
    assert_eq!(fs::read(&fixture.backup).unwrap(), backup_before);
    fixture.recover(true);
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.old);
}

#[test]
fn pack_promotion_rejects_aliased_destination_without_deleting_source() {
    let fixture = Fixture::new();
    assert!(matches!(
        promote_staged_pack(&fixture.installed, &fixture.installed, None::<&Path>),
        Err(ZrPackDeltaInstallError::InvalidPromotionPaths(_))
    ));
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.old);
}

#[test]
fn pack_promotion_automatically_recovers_before_retry() {
    let fixture = Fixture::new();
    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        None,
        TransactionFault::CrashAfterTargetReplace(0)
    )
    .is_err());
    let report = ZrPackDeltaInstaller::promote_staged_pack(
        &fixture.staged,
        &fixture.installed,
        None::<&Path>,
    )
    .unwrap();
    assert_eq!(
        report.promotion_method,
        ZrPackPromotionMethod::AtomicReplacement
    );
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
    assert!(!fixture.staged.exists());
}

#[test]
fn delta_install_restart_after_commit_resumes_receipt_without_overwriting_backup() {
    let fixture = Fixture::new();
    let delta = ZrPackDeltaWriter::write(
        &ZrPackReader::from_bytes(fixture.old.clone()).unwrap(),
        &ZrPackReader::from_bytes(fixture.new.clone()).unwrap(),
    )
    .unwrap();
    let delta_path = fixture.root.join("update.zrpd");
    fs::write(&delta_path, delta.bytes).unwrap();
    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        Some(&fixture.backup),
        TransactionFault::CrashAfterAllCommitted
    )
    .is_err());
    let (staging, promotion) = ZrPackDeltaInstaller::install_delta(
        &fixture.installed,
        &delta_path,
        &fixture.staged,
        &fixture.installed,
        Some(&fixture.backup),
    )
    .unwrap();
    assert_eq!(
        promotion.promotion_method,
        ZrPackPromotionMethod::AlreadyInstalled
    );
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
    assert_eq!(fs::read(&fixture.backup).unwrap(), fixture.old);
    let receipt_path = fixture.root.join("install.json");
    assert!(
        ZrPackDeltaInstaller::write_install_receipt(&fixture.installed, &staging, &promotion)
            .is_err()
    );
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
    ZrPackDeltaInstaller::write_install_receipt(&receipt_path, &staging, &promotion).unwrap();
    assert_eq!(
        ZrPackDeltaInstaller::read_install_receipt(&receipt_path)
            .unwrap()
            .promotion_method,
        ZrPackPromotionMethod::AlreadyInstalled
    );
    let (_, second) = ZrPackDeltaInstaller::install_delta(
        &fixture.installed,
        &delta_path,
        &fixture.staged,
        &fixture.installed,
        Some(&fixture.backup),
    )
    .unwrap();
    assert_eq!(
        second.promotion_method,
        ZrPackPromotionMethod::AlreadyInstalled
    );
    assert_eq!(fs::read(&fixture.backup).unwrap(), fixture.old);
}

#[test]
fn delta_install_restart_before_commit_recovers_base_before_rebuilding() {
    let fixture = Fixture::new();
    let delta = ZrPackDeltaWriter::write(
        &ZrPackReader::from_bytes(fixture.old.clone()).unwrap(),
        &ZrPackReader::from_bytes(fixture.new.clone()).unwrap(),
    )
    .unwrap();
    let delta_path = fixture.root.join("update.zrpd");
    fs::write(&delta_path, delta.bytes).unwrap();
    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        None,
        TransactionFault::CrashAfterTargetReplace(0)
    )
    .is_err());
    ZrPackDeltaInstaller::install_delta(
        &fixture.installed,
        &delta_path,
        &fixture.staged,
        &fixture.installed,
        None::<&Path>,
    )
    .unwrap();
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
    assert!(!fixture.staged.exists());
}

#[test]
fn delta_install_rejects_new_backup_path_after_no_backup_commit() {
    let fixture = Fixture::new();
    let delta = ZrPackDeltaWriter::write(
        &ZrPackReader::from_bytes(fixture.old.clone()).unwrap(),
        &ZrPackReader::from_bytes(fixture.new.clone()).unwrap(),
    )
    .unwrap();
    let delta_path = fixture.root.join("update.zrpd");
    let retry_backup = fixture.root.join("retry-backup/pack.zrpack");
    fs::write(&delta_path, delta.bytes).unwrap();

    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        None,
        TransactionFault::CrashAfterAllCommitted
    )
    .is_err());

    let error = ZrPackDeltaInstaller::install_delta(
        &fixture.installed,
        &delta_path,
        &fixture.staged,
        &fixture.installed,
        Some(&retry_backup),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ZrPackDeltaInstallError::ReadFailed { path, .. } if path == retry_backup
    ));
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
    assert!(!retry_backup.exists());
}

#[test]
fn delta_install_rejects_wrong_backup_after_committed_publication() {
    let fixture = Fixture::new();
    let delta = ZrPackDeltaWriter::write(
        &ZrPackReader::from_bytes(fixture.old.clone()).unwrap(),
        &ZrPackReader::from_bytes(fixture.new.clone()).unwrap(),
    )
    .unwrap();
    let delta_path = fixture.root.join("update.zrpd");
    fs::write(&delta_path, delta.bytes).unwrap();

    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        None,
        TransactionFault::CrashAfterAllCommitted
    )
    .is_err());
    fs::write(&fixture.backup, &fixture.new).unwrap();

    let error = ZrPackDeltaInstaller::install_delta(
        &fixture.installed,
        &delta_path,
        &fixture.staged,
        &fixture.installed,
        Some(&fixture.backup),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ZrPackDeltaInstallError::BackupPackMismatch { path, .. } if path == fixture.backup
    ));
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
    assert_eq!(fs::read(&fixture.backup).unwrap(), fixture.new);
}

#[test]
fn delta_install_rejects_corrupt_backup_after_committed_publication() {
    let fixture = Fixture::new();
    let delta = ZrPackDeltaWriter::write(
        &ZrPackReader::from_bytes(fixture.old.clone()).unwrap(),
        &ZrPackReader::from_bytes(fixture.new.clone()).unwrap(),
    )
    .unwrap();
    let delta_path = fixture.root.join("update.zrpd");
    fs::write(&delta_path, delta.bytes).unwrap();

    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        None,
        TransactionFault::CrashAfterAllCommitted
    )
    .is_err());
    fs::write(&fixture.backup, b"not a pack").unwrap();

    let error = ZrPackDeltaInstaller::install_delta(
        &fixture.installed,
        &delta_path,
        &fixture.staged,
        &fixture.installed,
        Some(&fixture.backup),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ZrPackDeltaInstallError::BackupPackMismatch { path, .. } if path == fixture.backup
    ));
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.new);
    assert_eq!(fs::read(&fixture.backup).unwrap(), b"not a pack");
}

#[test]
fn pack_promotion_corrupt_journal_preserves_installed_and_staged() {
    let fixture = Fixture::new();
    assert!(promote_with_fault(
        &fixture.staged,
        &fixture.installed,
        None,
        TransactionFault::CrashAfterStaging(0)
    )
    .is_err());
    let paths = PromotionPaths::new(&fixture.staged, &fixture.installed, None).unwrap();
    let journal = fs::read_dir(&paths.journal)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "zrjournal")
        })
        .unwrap();
    fs::write(journal, b"invalid durable journal").unwrap();
    assert!(ZrPackDeltaInstaller::recover_pending_promotion(
        &fixture.staged,
        &fixture.installed,
        None::<&Path>
    )
    .is_err());
    assert_eq!(fs::read(&fixture.installed).unwrap(), fixture.old);
    assert_eq!(fs::read(&fixture.staged).unwrap(), fixture.new);
}
