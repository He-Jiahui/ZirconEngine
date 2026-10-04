//! Genuine framed-journal regression coverage for cleanup-only restart recovery.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::super::super::commit::document_artifacts;
use super::super::super::journal::{decode_journal_with_valid_len, record_phase, record_state};
use super::super::super::schema::{FoldedTransactionJournal, JournalPhase, JournalState};
use super::super::super::stage::remove_reserved_if_exists;
use super::super::super::{
    commit_prepared_files, DurableCommitReport, DurableTransactionError, JournalDocument,
    PreparedFileWrite, TransactionFault, TransactionPhase,
};
use super::super::replay::recover_cleanup_journal_with;
use super::super::{
    detect_pending_transactions, recover_pending_transactions, RecoveryMode, RecoveryPolicy,
};
use super::test_directory;

struct ExactTargets {
    paths: BTreeSet<PathBuf>,
    mode: RecoveryMode,
    samples: Cell<usize>,
}

impl ExactTargets {
    fn new(paths: &[PathBuf], mode: RecoveryMode) -> Self {
        Self {
            paths: paths.iter().cloned().collect(),
            mode,
            samples: Cell::new(0),
        }
    }
}

fn validate_exact(paths: &BTreeSet<PathBuf>, document: &JournalDocument) -> Result<(), String> {
    if !paths.contains(document.target())
        || document.retired_paths().any(|path| !paths.contains(path))
    {
        return Err("journal references a path outside the fixture's authored inputs".to_owned());
    }
    Ok(())
}

impl RecoveryPolicy for ExactTargets {
    fn recovery_mode(&self) -> RecoveryMode {
        self.samples.set(self.samples.get() + 1);
        self.mode
    }

    fn validate_document(
        &self,
        _journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        validate_exact(&self.paths, document)
    }
}

struct DefaultRestore {
    paths: BTreeSet<PathBuf>,
}

impl DefaultRestore {
    fn new(paths: &[PathBuf]) -> Self {
        Self {
            paths: paths.iter().cloned().collect(),
        }
    }
}

impl RecoveryPolicy for DefaultRestore {
    fn validate_document(
        &self,
        _journal_path: &Path,
        document: &JournalDocument,
    ) -> Result<(), String> {
        validate_exact(&self.paths, document)
    }
}

struct Fixture {
    root: PathBuf,
    directory: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = test_directory(name);
        fs::create_dir_all(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let directory = root.join("journal");
        Self { root, directory }
    }

    fn existing_pair(&self) -> (Vec<PathBuf>, Vec<PreparedFileWrite>) {
        let first = self.root.join("first.zmeta");
        let second = self.root.join("second.zmeta");
        fs::write(&first, b"old-first").unwrap();
        fs::write(&second, b"old-second").unwrap();
        let writes = vec![
            PreparedFileWrite::new(first.clone(), b"new-first".to_vec()),
            PreparedFileWrite::new(second.clone(), b"new-second".to_vec()),
        ];
        (vec![first, second], writes)
    }

    fn interrupt(
        &self,
        writes: Vec<PreparedFileWrite>,
        fault: TransactionFault,
    ) -> (PathBuf, FoldedTransactionJournal) {
        commit_prepared_files(
            &self.directory,
            "project",
            writes,
            fault,
            &mut DurableCommitReport::default(),
        )
        .expect_err("a real producer fault must retain a current framed journal");
        let journals = fs::read_dir(&self.directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("zrjournal"))
            .collect::<Vec<_>>();
        assert_eq!(journals.len(), 1);
        let path = journals.into_iter().next().unwrap();
        let journal = read_journal(&path);
        (path, journal)
    }

    fn finish(self) {
        fs::remove_dir_all(self.root).unwrap();
    }
}

fn read_journal(path: &Path) -> FoldedTransactionJournal {
    let bytes = fs::read(path).unwrap();
    let (parsed, valid_len) = decode_journal_with_valid_len(path, &bytes).unwrap();
    assert_eq!(valid_len, bytes.len());
    parsed.fold().unwrap()
}

fn snapshot(paths: impl IntoIterator<Item = PathBuf>) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    paths
        .into_iter()
        .map(|path| {
            let bytes = match fs::read(&path) {
                Ok(bytes) => Some(bytes),
                Err(error) if error.kind() == io::ErrorKind::NotFound => None,
                Err(error) => panic!("cannot snapshot {}: {error}", path.display()),
            };
            (path, bytes)
        })
        .collect()
}

fn transaction_snapshot(
    path: &Path,
    journal: &FoldedTransactionJournal,
) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut paths = vec![path.to_path_buf()];
    for document in &journal.documents {
        paths.push(document.target().to_path_buf());
        paths.extend(document.retired_paths().map(Path::to_path_buf));
        paths.extend(document_artifacts(document).map(Path::to_path_buf));
    }
    snapshot(paths)
}

fn assert_cleaned(path: &Path, journal: &FoldedTransactionJournal) {
    assert!(!path.exists());
    for document in &journal.documents {
        for artifact in document_artifacts(document) {
            assert!(
                !artifact.exists(),
                "reserved artifact remains: {}",
                artifact.display()
            );
        }
    }
}

#[test]
fn cleanup_preserves_mixed_live_generations_and_samples_mode_once_per_call() {
    let fixture = Fixture::new("cleanup-mixed-live");
    let (targets, writes) = fixture.existing_pair();
    let (path, journal) = fixture.interrupt(writes, TransactionFault::CrashAfterCommit(0));
    assert_eq!(journal.phase, JournalPhase::Active);
    assert_eq!(journal.documents[0].state, JournalState::Committed);
    assert_eq!(journal.documents[1].state, JournalState::Prepared);
    let live = snapshot(targets.clone());
    let evidence = transaction_snapshot(&path, &journal);
    let mut policy = ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts);

    assert_eq!(
        detect_pending_transactions(&fixture.directory, "project", &mut policy).unwrap(),
        vec![path.clone()]
    );
    assert_eq!(policy.samples.get(), 1);
    assert_eq!(transaction_snapshot(&path, &journal), evidence);
    let recovered =
        recover_pending_transactions(&fixture.directory, "project", &mut policy).unwrap();

    assert_eq!(policy.samples.get(), 2);
    assert_eq!(recovered.rollback_count(), 0);
    assert_eq!(recovered.cleanup_count(), 1);
    assert_eq!(snapshot(targets), live);
    assert_cleaned(&path, &journal);
    fixture.finish();
}

#[test]
fn cleanup_preserves_new_or_replaced_target_and_partial_retirements() {
    for existed in [false, true] {
        let fixture = Fixture::new("cleanup-retirement-window");
        let target = fixture.root.join("replacement.zmeta");
        let first = fixture.root.join("first.meta.toml");
        let second = fixture.root.join("second.meta.toml");
        if existed {
            fs::write(&target, b"old-target").unwrap();
        }
        fs::write(&first, b"old-first-retired").unwrap();
        fs::write(&second, b"old-second-retired").unwrap();
        let write = PreparedFileWrite::new(target.clone(), b"new-target".to_vec())
            .retiring_with_expected_digest(
                first.clone(),
                blake3::hash(b"old-first-retired").to_hex().to_string(),
            )
            .retiring_with_expected_digest(
                second.clone(),
                blake3::hash(b"old-second-retired").to_hex().to_string(),
            );
        let (path, journal) =
            fixture.interrupt(vec![write], TransactionFault::CrashAfterRetiredDelete(0));
        assert_eq!(fs::read(&target).unwrap(), b"new-target");
        assert!(!first.exists());
        assert_eq!(fs::read(&second).unwrap(), b"old-second-retired");
        let targets = vec![target, first, second];
        let live = snapshot(targets.clone());
        let recovered = recover_pending_transactions(
            &fixture.directory,
            "project",
            &mut ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts),
        )
        .unwrap();

        assert_eq!(recovered.rollback_count(), 0);
        assert_eq!(recovered.cleanup_count(), 1);
        assert_eq!(snapshot(targets), live);
        assert_cleaned(&path, &journal);
        fixture.finish();
    }
}

#[test]
fn cleanup_rejects_missing_existing_anchor_while_default_recovery_restores_it() {
    let fixture = Fixture::new("cleanup-missing-existing-anchor");
    let (targets, writes) = fixture.existing_pair();
    let (path, journal) = fixture.interrupt(writes, TransactionFault::CrashAfterCommit(0));
    fs::remove_file(&targets[0]).unwrap();
    let before = transaction_snapshot(&path, &journal);

    let error = recover_pending_transactions(
        &fixture.directory,
        "project",
        &mut ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts),
    )
    .expect_err("cleanup must not discard the only evidence for an existing live anchor");
    assert!(matches!(
        error,
        DurableTransactionError::InvalidJournal { reason, .. }
            if reason.contains("missing existing live target")
    ));
    assert_eq!(transaction_snapshot(&path, &journal), before);
    let recovered = recover_pending_transactions(
        &fixture.directory,
        "project",
        &mut DefaultRestore::new(&targets),
    )
    .unwrap();

    assert_eq!(recovered.rollback_count(), 1);
    assert_eq!(recovered.cleanup_count(), 1);
    assert_eq!(fs::read(&targets[0]).unwrap(), b"old-first");
    assert_eq!(fs::read(&targets[1]).unwrap(), b"old-second");
    assert_cleaned(&path, &journal);
    fixture.finish();
}

#[test]
fn missing_new_target_requires_every_retired_live_anchor() {
    for retired_deleted in [false, true] {
        let fixture = Fixture::new("cleanup-missing-new-anchor");
        let target = fixture.root.join("new.zmeta");
        let retired = fixture.root.join("old.meta.toml");
        fs::write(&retired, b"original-retired").unwrap();
        let write = PreparedFileWrite::new(target.clone(), b"new-target".to_vec())
            .retiring_with_expected_digest(
                retired.clone(),
                blake3::hash(b"original-retired").to_hex().to_string(),
            );
        let fault = if retired_deleted {
            TransactionFault::CrashAfterRetiredDelete(0)
        } else {
            TransactionFault::CrashAfterTargetReplace(0)
        };
        let (path, journal) = fixture.interrupt(vec![write], fault);
        fs::remove_file(&target).unwrap();
        let targets = vec![target.clone(), retired.clone()];
        let before = transaction_snapshot(&path, &journal);
        let result = recover_pending_transactions(
            &fixture.directory,
            "project",
            &mut ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts),
        );

        if retired_deleted {
            assert!(matches!(
                result,
                Err(DurableTransactionError::InvalidJournal { .. })
            ));
            assert_eq!(transaction_snapshot(&path, &journal), before);
        } else {
            assert_eq!(result.unwrap().rollback_count(), 0);
            assert!(!target.exists());
            assert_eq!(fs::read(&retired).unwrap(), b"original-retired");
            assert_cleaned(&path, &journal);
        }
        fixture.finish();
    }
}

#[test]
fn cleanup_active_retry_keeps_states_and_rejects_default_restore_consumers() {
    let fixture = Fixture::new("cleanup-active-retry");
    let (targets, writes) = fixture.existing_pair();
    let (path, journal) = fixture.interrupt(writes, TransactionFault::CrashAfterCommit(0));
    record_phase(&path, JournalPhase::CleanupActive).unwrap();
    remove_reserved_if_exists(&journal.documents[0].backup).unwrap();
    remove_reserved_if_exists(&journal.documents[1].staging).unwrap();
    let retry = read_journal(&path);
    assert_eq!(retry.phase, JournalPhase::CleanupActive);
    assert_eq!(retry.documents[0].state, JournalState::Committed);
    assert_eq!(retry.documents[1].state, JournalState::Prepared);
    let before = transaction_snapshot(&path, &retry);
    let mut default = DefaultRestore::new(&targets);

    for result in [
        detect_pending_transactions(&fixture.directory, "project", &mut default).map(|_| ()),
        recover_pending_transactions(&fixture.directory, "project", &mut default).map(|_| ()),
    ] {
        assert!(matches!(
            result,
            Err(DurableTransactionError::InvalidJournal { .. })
        ));
        assert_eq!(transaction_snapshot(&path, &retry), before);
    }
    let live = snapshot(targets.clone());
    let recovered = recover_pending_transactions(
        &fixture.directory,
        "project",
        &mut ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts),
    )
    .unwrap();

    assert_eq!(recovered.rollback_count(), 0);
    assert_eq!(recovered.cleanup_count(), 1);
    assert_eq!(snapshot(targets), live);
    assert_cleaned(&path, &retry);
    fixture.finish();
}

#[test]
fn cleanup_active_retry_still_validates_live_anchors_and_remaining_artifacts() {
    for breach in [
        "missing-anchor",
        "foreign-live",
        "corrupt-stage",
        "corrupt-backup",
    ] {
        let fixture = Fixture::new("cleanup-active-invalid-retry");
        let (targets, writes) = fixture.existing_pair();
        let (path, journal) = fixture.interrupt(writes, TransactionFault::CrashAfterCommit(0));
        record_phase(&path, JournalPhase::CleanupActive).unwrap();
        remove_reserved_if_exists(&journal.documents[0].backup).unwrap();
        match breach {
            "missing-anchor" => fs::remove_file(&targets[0]).unwrap(),
            "foreign-live" => fs::write(&targets[0], b"unrelated-generation").unwrap(),
            "corrupt-stage" => fs::write(&journal.documents[1].staging, b"corrupt").unwrap(),
            "corrupt-backup" => fs::write(&journal.documents[1].backup, b"corrupt").unwrap(),
            _ => unreachable!(),
        }
        let retry = read_journal(&path);
        let before = transaction_snapshot(&path, &retry);
        let error = recover_pending_transactions(
            &fixture.directory,
            "project",
            &mut ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts),
        )
        .expect_err("cleanup permits missing artifacts, never corrupted evidence");

        assert!(matches!(
            error,
            DurableTransactionError::InvalidJournal { .. }
        ));
        assert_eq!(transaction_snapshot(&path, &retry), before);
        fixture.finish();
    }
}

#[derive(Clone, Copy)]
enum AppendFailure {
    BeforeWrite,
    TornFrame,
    CompleteFrame,
}

#[test]
fn cleanup_append_failure_retains_real_intent_active_and_terminal_evidence() {
    let cases = [
        (
            TransactionFault::CrashAfterStaging(0),
            JournalPhase::Intent,
            JournalPhase::CleanupIntent,
        ),
        (
            TransactionFault::CrashAfterCommit(0),
            JournalPhase::Active,
            JournalPhase::CleanupActive,
        ),
        (
            TransactionFault::CrashAfterRollbackCompleted { commit_index: 1 },
            JournalPhase::RollbackCompleted,
            JournalPhase::CleanupRollback,
        ),
        (
            TransactionFault::CrashAfterAllCommitted,
            JournalPhase::AllCommitted,
            JournalPhase::Cleanup,
        ),
    ];
    for (fault, expected, cleanup_phase) in cases {
        for append_failure in [
            AppendFailure::BeforeWrite,
            AppendFailure::TornFrame,
            AppendFailure::CompleteFrame,
        ] {
            let fixture = Fixture::new("cleanup-append-failure");
            let (targets, writes) = fixture.existing_pair();
            let (path, journal) = fixture.interrupt(writes, fault);
            assert_eq!(journal.phase, expected);
            let mut policy = ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts);
            assert_eq!(
                detect_pending_transactions(&fixture.directory, "project", &mut policy).unwrap(),
                vec![path.clone()]
            );
            let before = transaction_snapshot(&path, &journal);
            let attempts = Cell::new(0);

            let error = recover_cleanup_journal_with(
                &path,
                &journal.documents,
                cleanup_phase,
                RecoveryMode::CleanupArtifacts,
                |phase| {
                    assert_eq!(phase, cleanup_phase);
                    attempts.set(attempts.get() + 1);
                    if !matches!(append_failure, AppendFailure::BeforeWrite) {
                        record_phase(&path, phase)?;
                        if matches!(append_failure, AppendFailure::TornFrame) {
                            let file = fs::OpenOptions::new().write(true).open(&path).unwrap();
                            let prefix_len = before[0].1.as_ref().unwrap().len() as u64;
                            let appended_len = file.metadata().unwrap().len() - prefix_len;
                            assert!(appended_len > 1);
                            file.set_len(prefix_len + appended_len / 2).unwrap();
                            file.sync_all().unwrap();
                        }
                    }
                    Err(DurableTransactionError::operation(
                        TransactionPhase::Recovery,
                        &path,
                        io::Error::other("injected phase append acknowledgement failure"),
                    ))
                },
            )
            .expect_err("cleanup append failure must not fall back to journal-first deletion");

            assert!(matches!(
                error,
                DurableTransactionError::Operation {
                    phase: TransactionPhase::Recovery,
                    ..
                }
            ));
            assert_eq!(attempts.get(), 1);
            let after = transaction_snapshot(&path, &journal);
            assert_eq!(&after[1..], &before[1..]);
            match append_failure {
                AppendFailure::BeforeWrite => assert_eq!(after, before),
                AppendFailure::CompleteFrame => {
                    assert_ne!(after[0], before[0]);
                    assert_eq!(read_journal(&path).phase, cleanup_phase);
                }
                AppendFailure::TornFrame => {
                    let bytes = after[0].1.as_ref().unwrap();
                    let prefix = before[0].1.as_ref().unwrap();
                    let (parsed, valid_len) = decode_journal_with_valid_len(&path, bytes).unwrap();
                    assert_eq!(valid_len, prefix.len());
                    assert!(bytes.len() > valid_len);
                    assert_eq!(&bytes[..valid_len], prefix);
                    assert_eq!(parsed.fold().unwrap().phase, expected);
                }
            }
            let live = snapshot(targets.clone());
            let recovered =
                recover_pending_transactions(&fixture.directory, "project", &mut policy).unwrap();
            assert_eq!(recovered.rollback_count(), 0);
            assert_eq!(recovered.cleanup_count(), 1);
            assert_eq!(snapshot(targets), live);
            assert_cleaned(&path, &journal);
            fixture.finish();
        }
    }
}

#[test]
fn cleanup_active_rejects_later_activation_and_document_transitions() {
    for reactivate in [false, true] {
        let fixture = Fixture::new("cleanup-active-sealed-phase");
        let (targets, writes) = fixture.existing_pair();
        let (path, journal) = fixture.interrupt(writes, TransactionFault::CrashAfterCommit(0));
        record_phase(&path, JournalPhase::CleanupActive).unwrap();
        if reactivate {
            record_phase(&path, JournalPhase::Active).unwrap();
        } else {
            record_state(&path, 0, JournalState::RollingBack).unwrap();
        }
        let before = transaction_snapshot(&path, &journal);
        let result = recover_pending_transactions(
            &fixture.directory,
            "project",
            &mut ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts),
        );

        assert!(matches!(
            result,
            Err(DurableTransactionError::InvalidJournal { .. })
        ));
        assert_eq!(transaction_snapshot(&path, &journal), before);
        fixture.finish();
    }
}

#[test]
fn invalid_journal_in_batch_prevents_valid_cleanup_and_orphan_removal() {
    let fixture = Fixture::new("cleanup-invalid-batch");
    let (targets, writes) = fixture.existing_pair();
    let (path, journal) = fixture.interrupt(writes, TransactionFault::CrashAfterCommit(0));
    let invalid = fixture.directory.join("zz-invalid.zrjournal");
    let orphan = fixture
        .directory
        .join("..registry.zr-project-journal-10-2.zrjournal.zr-staging-123-4");
    fs::write(&invalid, b"not a current framed journal").unwrap();
    fs::write(&orphan, b"unpublished intent frame").unwrap();
    let before = transaction_snapshot(&path, &journal);
    let extra = snapshot([invalid.clone(), orphan.clone()]);
    let mut policy = ExactTargets::new(&targets, RecoveryMode::CleanupArtifacts);

    assert!(detect_pending_transactions(&fixture.directory, "project", &mut policy).is_err());
    assert!(recover_pending_transactions(&fixture.directory, "project", &mut policy).is_err());
    assert_eq!(transaction_snapshot(&path, &journal), before);
    assert_eq!(snapshot([invalid.clone(), orphan.clone()]), extra);
    fs::remove_file(invalid).unwrap();
    let live = snapshot(targets.clone());
    let recovered =
        recover_pending_transactions(&fixture.directory, "project", &mut policy).unwrap();

    assert_eq!(recovered.rollback_count(), 0);
    assert_eq!(recovered.cleanup_count(), 1);
    assert_eq!(recovered.intent_orphan_cleanup_count(), 1);
    assert!(!orphan.exists());
    assert_eq!(snapshot(targets), live);
    assert_cleaned(&path, &journal);
    fixture.finish();
}
