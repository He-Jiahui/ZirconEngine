use std::fs;

use super::*;

struct AcceptAll;

impl super::super::super::recovery::RecoveryPolicy for AcceptAll {
    fn validate_document(
        &self,
        _journal_path: &std::path::Path,
        _document: &super::super::super::schema::JournalDocument,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn active_missing_existing_target_recovery_restores_valid_backup() {
    let output_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "zircon-durable-active-committing-missing-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&target, b"old-generation").unwrap();

    let mut report = DurableCommitReport::default();
    commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::CrashAfterTargetReplace(0),
        &mut report,
    )
    .expect_err("the interruption must retain an active committing journal");
    fs::remove_file(&target).unwrap();

    let recovered = super::super::super::recovery::recover_pending_transactions(
        &journal,
        "project",
        &mut AcceptAll,
    )
    .expect("active rollback may restore a missing target from its validated backup");

    assert_eq!(recovered.rollback_count(), 1);
    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}
