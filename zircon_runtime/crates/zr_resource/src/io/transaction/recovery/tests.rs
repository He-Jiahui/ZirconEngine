use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use super::super::error::{DurableTransactionError, TransactionPhase};
use super::super::journal::MAX_JOURNAL_BYTES;
use super::super::schema::{
    FoldedTransactionJournal, JOURNAL_VERSION, JournalDocument, JournalIntent, JournalPhase,
    JournalState, TransactionJournal,
};
use super::replay::recover_active_journal_with;
use super::validation::{validate_document_paths, validate_journals};
use super::*;

const TEST_TRANSACTION_ID: &str =
    "0000000000000000000000000000000000000000000000000000000000000000-1-1";

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
    let resolved_orphan = fs::canonicalize(&root)
        .unwrap()
        .join(orphan.file_name().unwrap());

    assert_eq!(
        detect_pending_transactions(&root, "project", &mut policy).unwrap(),
        vec![resolved_orphan]
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
fn version_six_journal_is_rejected_before_artifact_or_target_mutation() {
    let root = test_directory("version-six-hard-cut");
    let journal_directory = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&journal_directory).unwrap();
    fs::write(&target, b"live-generation").unwrap();
    let transaction_id = super::super::pathing::transaction_id_for_test(&journal_directory, 1);
    let journal_path = write_intent_journal(
        &journal_directory,
        &target,
        &transaction_id,
        JOURNAL_VERSION - 1,
    );
    let mut policy = AcceptNothing;

    let error = detect_pending_transactions(&journal_directory, "project", &mut policy)
        .expect_err("version 6 has no compatibility recovery path");

    assert!(matches!(
        error,
        DurableTransactionError::InvalidJournal { reason, .. }
            if reason == "unsupported durable transaction journal version"
    ));
    assert_eq!(fs::read(&target).unwrap(), b"live-generation");
    assert!(journal_path.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn two_component_transaction_id_is_rejected_before_artifact_mutation() {
    let root = test_directory("legacy-transaction-id-hard-cut");
    let journal_directory = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&journal_directory).unwrap();
    fs::write(&target, b"live-generation").unwrap();
    let journal_path = write_intent_journal(&journal_directory, &target, "42-1", JOURNAL_VERSION);
    let mut policy = AcceptNothing;

    let error = detect_pending_transactions(&journal_directory, "project", &mut policy)
        .expect_err("the v6 transaction id wire must not be accepted by v7");

    assert!(matches!(
        error,
        DurableTransactionError::InvalidJournal { reason, .. }
            if reason == "invalid transaction id"
    ));
    assert_eq!(fs::read(&target).unwrap(), b"live-generation");
    assert!(journal_path.exists());
    fs::remove_dir_all(root).unwrap();
}

fn write_intent_journal(
    journal_directory: &Path,
    target: &Path,
    transaction_id: &str,
    version: u32,
) -> PathBuf {
    let staging =
        super::super::pathing::transaction_sibling(target, "project", "stage", transaction_id);
    let backup =
        super::super::pathing::transaction_sibling(target, "project", "backup", transaction_id);
    let rollback_staging = super::super::pathing::transaction_sibling(
        target,
        "project",
        "rollback-stage",
        transaction_id,
    );
    let journal_path =
        super::super::pathing::journal_path(journal_directory, target, "project", transaction_id);
    let journal = TransactionJournal {
        version,
        tag: "project".to_owned(),
        transaction_id: transaction_id.to_owned(),
        documents: vec![JournalIntent {
            target: target.to_path_buf(),
            staging,
            backup,
            rollback_staging,
            retirements: Vec::new(),
        }],
        transitions: Vec::new(),
    };
    let frame =
        super::super::journal::encode_frame(toml::to_string_pretty(&journal).unwrap().as_bytes())
            .unwrap();
    fs::write(&journal_path, frame).unwrap();
    journal_path
}

#[test]
fn lexical_target_alias_in_journal_is_rejected_before_recovery_io() {
    let root = test_directory("journal-target-alias");
    let asset_root = root.join("assets");
    let journal_directory_path = root.join("journal");
    let journal_path = journal_directory_path.join("generation.zrjournal");
    fs::create_dir_all(asset_root.join("nested")).unwrap();
    fs::create_dir_all(&journal_directory_path).unwrap();
    let target = fs::canonicalize(&asset_root)
        .unwrap()
        .join("generation.zmeta");
    #[cfg(windows)]
    let alias_root = {
        let asset_root_text = asset_root.to_string_lossy();
        PathBuf::from(
            asset_root_text
                .strip_prefix(r"\\?\")
                .unwrap_or(asset_root_text.as_ref()),
        )
    };
    #[cfg(not(windows))]
    let alias_root = asset_root.clone();
    let mut aliased_target = alias_root.as_os_str().to_os_string();
    let separator = std::path::MAIN_SEPARATOR;
    aliased_target.push(format!(
        "{separator}nested{separator}..{separator}generation.zmeta"
    ));
    let aliased_target = PathBuf::from(aliased_target);
    let document = JournalDocument {
        state: JournalState::Intent,
        target_existed: Some(false),
        original_digest: None,
        new_digest: None,
        retired_digests: Vec::new(),
        target: aliased_target,
        staging: root.join("generation.stage"),
        backup: root.join("generation.backup"),
        rollback_staging: root.join("generation.rollback"),
        retirements: Vec::new(),
    };
    let mut identities = BTreeSet::new();
    let owner_lock = super::super::owner_lock::owner_lock_path(journal_path.parent().unwrap())
        .and_then(|path| super::super::pathing::PathIdentity::resolve(&path))
        .unwrap();
    let journal_directory =
        super::super::pathing::PathIdentity::resolve(journal_path.parent().unwrap()).unwrap();
    let transaction_id =
        super::super::pathing::transaction_id_for_test(journal_path.parent().unwrap(), 1);
    let aliased_identity = super::super::pathing::PathIdentity::resolve(&document.target).unwrap();
    assert_eq!(aliased_identity.operation_path(), target);
    assert!(
        !aliased_identity.has_exact_operation_path_encoding(&document.target),
        "resolved operation path {:?} must retain a different wire encoding from alias {:?}",
        aliased_identity.operation_path(),
        document.target
    );

    let error = validate_document_paths(
        &journal_path,
        "project",
        &transaction_id,
        &document,
        &mut identities,
        &owner_lock,
        &journal_directory,
    )
    .expect_err("recovery must not execute an aliased immutable intent");

    assert!(
        matches!(
            &error,
            DurableTransactionError::InvalidJournal { reason, .. }
                if reason.contains("not a normalized physical path")
        ),
        "unexpected lexical alias validation error: {error:?}"
    );
    assert!(identities.is_empty());
    assert!(!target.exists());
    assert!(!journal_path.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_rejects_cross_journal_ancestor_overlap_before_evidence_io() {
    struct AcceptAllWithoutEvidence;

    impl RecoveryPolicy for AcceptAllWithoutEvidence {
        fn validate_document(
            &self,
            _journal_path: &Path,
            _document: &JournalDocument,
        ) -> Result<(), String> {
            Ok(())
        }

        fn digest_file(&mut self, _path: &Path) -> io::Result<String> {
            panic!("namespace validation must precede evidence I/O")
        }
    }

    let root = test_directory("cross-journal-ancestor-overlap");
    let journal_directory = root.join("journal");
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::create_dir_all(&journal_directory).unwrap();
    let ancestor_target = fs::canonicalize(root.join("assets"))
        .unwrap()
        .join("generation.zmeta");
    let descendant_target = ancestor_target.join("child.zmeta");
    let first_transaction_id =
        super::super::pathing::transaction_id_for_test(&journal_directory, 1);
    let second_transaction_id =
        super::super::pathing::transaction_id_for_test(&journal_directory, 2);
    let journals = [
        journal_fixture(&journal_directory, ancestor_target, &first_transaction_id),
        journal_fixture(
            &journal_directory,
            descendant_target,
            &second_transaction_id,
        ),
    ];
    let mut policy = AcceptAllWithoutEvidence;

    let error = validate_journals(&journals, "project", &mut policy)
        .expect_err("recovery paths must form one antichain across all journals");

    assert!(
        matches!(
            &error,
            DurableTransactionError::InvalidJournal { reason, .. }
                if reason.contains("ancestor or descendant")
        ),
        "unexpected cross-journal namespace error: {error:?}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_rejects_empty_transaction_journal_without_indexing_documents() {
    let root = test_directory("empty-transaction-journal");
    let journal_directory = root.join("journal");
    fs::create_dir_all(&journal_directory).unwrap();
    let journal_path = journal_directory.join("generation.zrjournal");
    let transaction_id = super::super::pathing::transaction_id_for_test(&journal_directory, 1);
    let journal = FoldedTransactionJournal {
        tag: "project".to_owned(),
        transaction_id,
        phase: JournalPhase::Intent,
        documents: Vec::new(),
    };
    let mut policy = AcceptNothing;

    let error = validate_journals(&[(journal_path, journal, 0)], "project", &mut policy)
        .expect_err("an empty folded journal must be rejected as invalid input");

    assert!(matches!(
        error,
        DurableTransactionError::InvalidJournal { reason, .. }
            if reason == "empty transaction journal"
    ));
    fs::remove_dir_all(root).unwrap();
}

fn journal_fixture(
    journal_directory: &Path,
    target: PathBuf,
    transaction_id: &str,
) -> (PathBuf, FoldedTransactionJournal, usize) {
    let staging =
        super::super::pathing::transaction_sibling(&target, "project", "stage", transaction_id);
    let backup =
        super::super::pathing::transaction_sibling(&target, "project", "backup", transaction_id);
    let rollback_staging = super::super::pathing::transaction_sibling(
        &target,
        "project",
        "rollback-stage",
        transaction_id,
    );
    let journal_path =
        super::super::pathing::journal_path(journal_directory, &target, "project", transaction_id);
    (
        journal_path,
        FoldedTransactionJournal {
            tag: "project".to_owned(),
            transaction_id: transaction_id.to_owned(),
            phase: JournalPhase::Intent,
            documents: vec![JournalDocument {
                state: JournalState::Prepared,
                target_existed: Some(false),
                original_digest: None,
                new_digest: Some(blake3::hash(b"new-generation").to_hex().to_string()),
                retired_digests: Vec::new(),
                target,
                staging,
                backup,
                rollback_staging,
                retirements: Vec::new(),
            }],
        },
        0,
    )
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
    let transaction_id =
        super::super::pathing::transaction_id_for_test(journal_path.parent().unwrap(), 1);
    let journal = FoldedTransactionJournal {
        tag: "project".to_owned(),
        transaction_id,
        phase: JournalPhase::Active,
        documents: vec![JournalDocument {
            state: JournalState::Committed,
            target_existed: Some(true),
            original_digest: Some(blake3::hash(b"old-generation").to_hex().to_string()),
            new_digest: Some(blake3::hash(b"new-generation").to_hex().to_string()),
            retired_digests: Vec::new(),
            target: target.clone(),
            staging: staging.clone(),
            backup: backup.clone(),
            rollback_staging: rollback_staging.clone(),
            retirements: Vec::new(),
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
        super::super::commit::restore_document,
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

#[test]
fn active_missing_existing_target_recovery_rejects_corrupted_backup_evidence() {
    let root = test_directory("active-missing-target-corrupt-backup");
    let journal_path = root.join("generation.zrjournal");
    let backup = root.join("generation.backup");
    fs::create_dir_all(&root).unwrap();
    fs::write(&backup, b"corrupted-old-generation").unwrap();
    let document = JournalDocument {
        state: JournalState::Committing,
        target_existed: Some(true),
        original_digest: Some(blake3::hash(b"old-generation").to_hex().to_string()),
        new_digest: Some(blake3::hash(b"new-generation").to_hex().to_string()),
        retired_digests: Vec::new(),
        target: root.join("generation.zmeta"),
        staging: root.join("generation.stage"),
        backup,
        rollback_staging: root.join("generation.rollback"),
        retirements: Vec::new(),
    };
    let mut policy = AcceptNothing;
    let mut evidence = super::evidence::EvidenceCache::default();

    let error = super::evidence::validate_document_evidence(
        &journal_path,
        JournalPhase::Active,
        &document,
        &mut policy,
        &mut evidence,
    )
    .expect_err("a missing target is recoverable only from a valid backup");

    assert!(matches!(
        error,
        DurableTransactionError::InvalidJournal { reason, .. }
            if reason.contains("backup artifact digest")
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn intent_recovery_uses_journal_first_cleanup_when_transition_append_fails() {
    let root = test_directory("intent-append-failure-journal-first");
    let journal_path = root.join("generation.zrjournal");
    let staging = root.join("generation.stage");
    fs::create_dir_all(&root).unwrap();
    fs::write(&journal_path, b"intent journal").unwrap();
    fs::write(&staging, b"staged-generation").unwrap();
    let documents = vec![JournalDocument {
        state: JournalState::Intent,
        target_existed: None,
        original_digest: None,
        new_digest: None,
        retired_digests: Vec::new(),
        target: root.join("generation.zmeta"),
        staging: staging.clone(),
        backup: root.join("generation.backup"),
        rollback_staging: root.join("generation.rollback"),
        retirements: Vec::new(),
    }];

    super::replay::recover_intent_journal_with(&journal_path, &documents, |_| {
        Err(DurableTransactionError::operation(
            TransactionPhase::Recovery,
            &journal_path,
            io::Error::other("injected bounded append failure"),
        ))
    })
    .expect("an Intent journal can be retired without appending another bounded frame");

    assert!(!journal_path.exists());
    assert!(!staging.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn intent_recovery_does_not_cleanup_after_transition_append_failure() {
    let root = test_directory("intent-cleanup-transition-failure");
    let journal_path = root.join("journal-as-directory");
    let staging = root.join("generation.stage");
    fs::create_dir_all(&journal_path).unwrap();
    fs::write(&staging, b"staged-generation").unwrap();
    let transaction_id = super::super::pathing::transaction_id_for_test(&root, 1);
    let journal = FoldedTransactionJournal {
        tag: "project".to_owned(),
        transaction_id,
        phase: JournalPhase::Intent,
        documents: vec![JournalDocument {
            state: JournalState::Intent,
            target_existed: None,
            original_digest: None,
            new_digest: None,
            retired_digests: Vec::new(),
            target: root.join("generation.zmeta"),
            staging: staging.clone(),
            backup: root.join("generation.backup"),
            rollback_staging: root.join("generation.rollback"),
            retirements: Vec::new(),
        }],
    };

    super::replay::recover_journal(&journal_path, &journal)
        .expect_err("intent cleanup must not start behind an uncertain journal tail");

    assert!(journal_path.is_dir());
    assert_eq!(fs::read(&staging).unwrap(), b"staged-generation");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_attempts_every_document_after_a_restore_failure() {
    let journal_path = PathBuf::from("generation.zrjournal");
    let journal = FoldedTransactionJournal {
        tag: "project".to_owned(),
        transaction_id: TEST_TRANSACTION_ID.to_owned(),
        phase: JournalPhase::Active,
        documents: (0..3)
            .map(|index| JournalDocument {
                state: JournalState::Committed,
                target_existed: Some(false),
                original_digest: None,
                new_digest: None,
                retired_digests: Vec::new(),
                target: PathBuf::from(format!("{index}.zmeta")),
                staging: PathBuf::from(format!("{index}.stage")),
                backup: PathBuf::from(format!("{index}.backup")),
                rollback_staging: PathBuf::from(format!("{index}.rollback")),
                retirements: Vec::new(),
            })
            .collect(),
    };
    let rolling_back = std::cell::RefCell::new(Vec::new());
    let restore_attempts = std::cell::RefCell::new(Vec::new());
    let phase_attempts = std::cell::Cell::new(0);

    let error = recover_active_journal_with(
        &journal_path,
        &journal,
        |index| {
            rolling_back.borrow_mut().push(index);
            Ok(())
        },
        |_| {
            phase_attempts.set(phase_attempts.get() + 1);
            Ok(())
        },
        |document| {
            let index = document
                .target
                .file_stem()
                .and_then(|value| value.to_str())
                .and_then(|value| value.parse::<usize>().ok())
                .expect("test target carries its document index");
            restore_attempts.borrow_mut().push(index);
            if index == 2 {
                Err(io::Error::other("injected first restore failure"))
            } else {
                Ok(())
            }
        },
    )
    .expect_err("one failed restore must retain active recovery evidence");

    assert!(matches!(
        error,
        DurableTransactionError::Operation {
            phase: TransactionPhase::Recovery,
            path,
            ..
        } if path == PathBuf::from("2.zmeta")
    ));
    assert_eq!(&*rolling_back.borrow(), &[2, 1, 0]);
    assert_eq!(&*restore_attempts.borrow(), &[2, 1, 0]);
    assert_eq!(phase_attempts.get(), 0);
}

fn test_directory(name: &str) -> PathBuf {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("resolve current workspace for durable transaction test output")
                .join("target")
        });
    output_root.join("zircon-test-output").join(format!(
        "zircon-durable-transaction-{name}-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ))
}
