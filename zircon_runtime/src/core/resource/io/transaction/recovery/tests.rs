use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use super::*;

struct AcceptNothing;

impl RecoveryPolicy for AcceptNothing {
    fn validate_document(
        &self,
        _journal_path: &Path,
        _document: &JournalDocument,
    ) -> Result<(), String> {
        Err("test policy has no valid publication targets".to_owned())
    }
}

#[test]
fn reserved_atomic_intent_orphan_is_reported_then_removed_by_recovery() {
    let root = test_directory("intent-orphan");
    fs::create_dir_all(&root).unwrap();
    let orphan = root.join("..registry.zr-project-journal-10-2.zrjournal.zr-staging-123-4");
    fs::write(&orphan, b"torn intent").unwrap();
    let mut policy = AcceptNothing;

    assert_eq!(
        detect_pending_transactions(&root, "project", &mut policy).unwrap(),
        vec![orphan.clone()]
    );
    recover_pending_transactions(&root, "project", &mut policy).unwrap();

    assert!(!orphan.exists());
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn oversized_journal_is_rejected_from_metadata_before_reading_payload() {
    let root = test_directory("oversized-journal");
    fs::create_dir_all(&root).unwrap();
    let journal = root.join("oversized.zrjournal");
    let file = File::create(&journal).unwrap();
    file.set_len((MAX_JOURNAL_BYTES + 1) as u64).unwrap();
    drop(file);
    let mut policy = AcceptNothing;

    let error = detect_pending_transactions(&root, "project", &mut policy).unwrap_err();

    assert!(matches!(
        error,
        DurableTransactionError::InvalidJournal { reason, .. }
            if reason.contains("bounded size")
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lexical_target_alias_in_journal_is_rejected_before_recovery_io() {
    let root = test_directory("journal-target-alias");
    let asset_root = root.join("assets");
    let journal_path = root.join("generation.zrjournal");
    let target = asset_root.join("generation.zmeta");
    let aliased_target = asset_root.join("nested/../generation.zmeta");
    fs::create_dir_all(asset_root.join("nested")).unwrap();
    let document = JournalDocument {
        state: JournalState::Intent,
        target_existed: Some(false),
        original_digest: None,
        new_digest: None,
        retired_digest: None,
        target: aliased_target,
        staging: root.join("generation.stage"),
        backup: root.join("generation.backup"),
        rollback_staging: root.join("generation.rollback"),
        retired_path: None,
        retired_backup: None,
        retired_rollback_staging: None,
    };
    let mut identities = BTreeSet::new();

    let error =
        validate_document_paths(&journal_path, "project", "1-1", &document, &mut identities)
            .expect_err("recovery must not execute an aliased immutable intent");

    assert!(matches!(
        error,
        DurableTransactionError::InvalidJournal { reason, .. }
            if reason.contains("not a normalized physical path")
    ));
    assert!(identities.is_empty());
    assert!(!target.exists());
    assert!(!journal_path.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn active_recovery_uses_journal_first_cleanup_when_transition_append_fails() {
    let root = test_directory("active-append-failure");
    let journal_path = root.join("generation.zrjournal");
    let target = root.join("generation.zmeta");
    let staging = root.join("generation.stage");
    let backup = root.join("generation.backup");
    let rollback_staging = root.join("generation.rollback");
    fs::create_dir_all(&root).unwrap();
    fs::write(&journal_path, b"active journal").unwrap();
    fs::write(&target, b"new-generation").unwrap();
    fs::write(&staging, b"new-generation").unwrap();
    fs::write(&backup, b"old-generation").unwrap();
    let journal = FoldedTransactionJournal {
        tag: "project".to_owned(),
        transaction_id: "1-1".to_owned(),
        phase: JournalPhase::Active,
        documents: vec![JournalDocument {
            state: JournalState::Committed,
            target_existed: Some(true),
            original_digest: Some(blake3::hash(b"old-generation").to_hex().to_string()),
            new_digest: Some(blake3::hash(b"new-generation").to_hex().to_string()),
            retired_digest: None,
            target: target.clone(),
            staging: staging.clone(),
            backup: backup.clone(),
            rollback_staging: rollback_staging.clone(),
            retired_path: None,
            retired_backup: None,
            retired_rollback_staging: None,
        }],
    };
    let state_attempts = std::cell::Cell::new(0);

    recover_active_journal_with(
        &journal_path,
        &journal,
        |index| {
            assert_eq!(index, 0);
            state_attempts.set(state_attempts.get() + 1);
            Err(DurableTransactionError::operation(
                TransactionPhase::Recovery,
                &journal_path,
                io::Error::other("injected bounded append failure"),
            ))
        },
        |_| panic!("phase append must stop after a state append failure"),
    )
    .expect("restored live files allow journal-first cleanup without another append");

    assert_eq!(state_attempts.get(), 1);
    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert!(!journal_path.exists());
    assert!(!staging.exists());
    assert!(!backup.exists());
    assert!(!rollback_staging.exists());
    fs::remove_dir_all(root).unwrap();
}

fn test_directory(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-durable-transaction-{name}-{}-{}",
        std::process::id(),
        crate::core::resource::io::NEXT_ATOMIC_FILE_ID
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed,)
    ))
}
