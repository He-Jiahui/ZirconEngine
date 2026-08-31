use std::fs;
use std::io;

use super::*;

#[test]
fn pre_active_abort_preserves_original_operation_when_cleanup_transition_fails() {
    let root = super::test_directory("abort-transition-error-contract");
    let journal = root.join("journal-as-directory");
    let original_path = root.join("generation.zmeta");
    fs::create_dir_all(&journal).unwrap();
    let original = DurableTransactionError::operation(
        TransactionPhase::Commit,
        &original_path,
        io::Error::new(io::ErrorKind::InvalidData, "injected commit failure"),
    );

    let error = abort_pre_active(&journal, &[], original, true);
    let message = error.to_string();

    assert!(matches!(
        &error,
        DurableTransactionError::Operation {
            phase: TransactionPhase::Commit,
            path,
            source,
            ..
        } if path == &original_path && source.kind() == io::ErrorKind::InvalidData
    ));
    let DurableTransactionError::Operation { source, .. } = &error else {
        unreachable!("the preceding assertion proved the operation variant")
    };
    let context = source
        .get_ref()
        .expect("cleanup context must retain the original I/O error");
    let original_source = std::error::Error::source(context)
        .and_then(|source| source.downcast_ref::<io::Error>())
        .expect("the cleanup context source must be the original I/O error");
    assert_eq!(original_source.kind(), io::ErrorKind::InvalidData);
    assert!(message.contains("failed to record pre-active cleanup transition"));
    assert!(journal.is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pre_active_abort_uses_journal_first_cleanup_for_uncertain_tail() {
    let root = super::test_directory("abort-uncertain-journal-tail");
    let journal = root.join("generation.zrjournal");
    let target = root.join("generation.zmeta");
    let staging = root.join("generation.stage");
    fs::create_dir_all(&root).unwrap();
    fs::write(&journal, b"intent with uncertain append tail").unwrap();
    fs::write(&staging, b"staged-generation").unwrap();
    let intents = vec![JournalIntent {
        target,
        staging: staging.clone(),
        backup: root.join("generation.backup"),
        rollback_staging: root.join("generation.rollback"),
        retirements: Vec::new(),
    }];
    let original = DurableTransactionError::operation(
        TransactionPhase::Stage,
        &journal,
        std::io::Error::other("injected stage failure"),
    );

    let error = abort_pre_active(&journal, &intents, original, false);

    assert!(error.to_string().contains("injected stage failure"));
    assert!(!journal.exists());
    assert!(!staging.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pre_active_uncertain_cleanup_preserves_original_phase_and_error_kind() {
    let root = super::test_directory("abort-preserves-error-contract");
    let journal = root.join("generation.zrjournal");
    fs::create_dir_all(&root).unwrap();
    fs::write(&journal, b"intent with uncertain append tail").unwrap();
    let original = DurableTransactionError::operation(
        TransactionPhase::Commit,
        &journal,
        std::io::Error::new(std::io::ErrorKind::InvalidData, "bounded append"),
    );

    let error = abort_pre_active(&journal, &[], original, false);

    assert!(matches!(
        error,
        DurableTransactionError::Operation {
            phase: TransactionPhase::Commit,
            source,
            ..
        } if source.kind() == std::io::ErrorKind::InvalidData
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pre_active_abort_keeps_journal_when_artifact_cleanup_fails() {
    let root = super::test_directory("abort-artifact-cleanup");
    let journal_directory = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&journal_directory).unwrap();
    let transaction_id =
        super::super::super::pathing::transaction_id_for_test(&journal_directory, 1);
    let (journal, intents) = create_intent(
        &journal_directory,
        "project",
        &transaction_id,
        &[PreparedFileWrite::new(target, b"new-generation".to_vec())],
    )
    .unwrap();
    fs::create_dir(&intents[0].staging).unwrap();
    let original = DurableTransactionError::operation(
        TransactionPhase::Stage,
        &journal,
        std::io::Error::other("injected stage failure"),
    );

    let error = abort_pre_active(&journal, &intents, original, true);

    assert!(error
        .to_string()
        .contains("failed to clean staged transaction artifacts"));
    assert!(journal.is_file());
    assert!(intents[0].staging.is_dir());
    fs::remove_dir_all(root).unwrap();
}
