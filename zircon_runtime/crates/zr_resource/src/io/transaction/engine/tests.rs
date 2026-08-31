use std::fs;

use super::*;

mod ambiguous_replace;
mod artifact_namespace;
mod durable_io_profile;
mod journal_namespace;
mod live_namespace;
mod namespace_profile;
mod pre_active_abort;

#[test]
fn one_generation_can_durably_retire_a_source_and_its_sidecar() {
    let output_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "durable-multi-retirement-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let registry = root.join("asset-registry.json");
    let source = root.join("panel.zui");
    let meta = root.join("panel.zui.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&registry, b"old-registry").unwrap();
    fs::write(&source, b"source").unwrap();
    fs::write(&meta, b"meta").unwrap();

    let mut report = DurableCommitReport::default();
    let outcome = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(registry.clone(), b"new-registry".to_vec())
                .retiring_with_expected_digest(
                    source.clone(),
                    blake3::hash(b"source").to_hex().to_string(),
                )
                .retiring_with_expected_digest(
                    meta.clone(),
                    blake3::hash(b"meta").to_hex().to_string(),
                ),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect("one transaction should publish the registry and retire both authored files");

    assert_eq!(outcome, DurableCommitDisposition::Durable);
    assert_eq!(fs::read(&registry).unwrap(), b"new-registry");
    assert!(!source.exists());
    assert!(!meta.exists());
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_restores_every_retirement_after_a_partial_multi_file_delete() {
    struct AcceptAll;

    impl super::super::recovery::RecoveryPolicy for AcceptAll {
        fn validate_document(
            &self,
            _journal_path: &std::path::Path,
            _document: &super::super::schema::JournalDocument,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    let output_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "durable-partial-multi-retirement-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let registry = root.join("asset-registry.json");
    let source = root.join("panel.zui");
    let meta = root.join("panel.zui.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&registry, b"old-registry").unwrap();
    fs::write(&source, b"source").unwrap();
    fs::write(&meta, b"meta").unwrap();

    let mut report = DurableCommitReport::default();
    commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(registry.clone(), b"new-registry".to_vec())
                .retiring_with_expected_digest(
                    source.clone(),
                    blake3::hash(b"source").to_hex().to_string(),
                )
                .retiring_with_expected_digest(
                    meta.clone(),
                    blake3::hash(b"meta").to_hex().to_string(),
                ),
        ],
        TransactionFault::CrashAfterRetiredDelete(0),
        &mut report,
    )
    .expect_err("the interruption must preserve an active multi-retirement journal");

    assert_eq!(fs::read(&registry).unwrap(), b"new-registry");
    assert!(!source.exists());
    assert_eq!(fs::read(&meta).unwrap(), b"meta");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);

    let recovered =
        super::super::recovery::recover_pending_transactions(&journal, "project", &mut AcceptAll)
            .expect(
                "recovery should restore the target and every retirement from durable evidence",
            );

    assert_eq!(recovered.rollback_count(), 1);
    assert_eq!(recovered.cleanup_count(), 1);
    assert_eq!(fs::read(&registry).unwrap(), b"old-registry");
    assert_eq!(fs::read(&source).unwrap(), b"source");
    assert_eq!(fs::read(&meta).unwrap(), b"meta");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn expected_retired_digest_rejects_a_file_changed_after_preparation() {
    let output_root = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "durable-retired-digest-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let source = root.join("source.asset");
    let target = root.join("moved.asset");
    fs::create_dir_all(&root).unwrap();
    fs::write(&source, b"prepared-source").unwrap();
    let expected_digest = blake3::hash(b"prepared-source").to_hex().to_string();
    fs::write(&source, b"changed-after-preparation").unwrap();

    let mut report = DurableCommitReport::default();
    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(target.clone(), b"prepared-source".to_vec())
                .retiring_with_expected_digest(source.clone(), expected_digest),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("a stale relocation source must not publish");

    assert!(error.to_string().contains("changed since preparation"));
    assert!(!target.exists());
    assert_eq!(fs::read(&source).unwrap(), b"changed-after-preparation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    assert_eq!(report, DurableCommitReport::default());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn lexical_target_aliases_are_rejected_before_journal_materialization() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-target-alias-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let asset_root = root.join("assets");
    let target = asset_root.join("generation.zmeta");
    let alias = asset_root.join("nested/../generation.zmeta");
    fs::create_dir_all(&asset_root).unwrap();
    fs::write(&target, b"old-generation").unwrap();

    let mut report = DurableCommitReport::default();
    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(target.clone(), b"first-generation".to_vec()),
            PreparedFileWrite::new(alias, b"second-generation".to_vec()),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("one physical target must not enter two transaction intents");

    assert!(error.to_string().contains("duplicate transaction target"));
    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert!(!journal.exists());
    assert_eq!(report, DurableCommitReport::default());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_unicode_case_aliases_for_missing_targets_are_rejected() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-windows-case-alias-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let asset_root = root.join("assets");
    fs::create_dir_all(&asset_root).unwrap();

    let mut report = DurableCommitReport::default();
    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(
                asset_root.join("generation-\u{00c4}.zmeta"),
                b"first-generation".to_vec(),
            ),
            PreparedFileWrite::new(
                asset_root.join("generation-\u{00e4}.zmeta"),
                b"second-generation".to_vec(),
            ),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("Windows path identity must fold Unicode casing for missing targets");

    assert!(error.to_string().contains("duplicate transaction target"));
    assert!(!journal.exists());
    assert_eq!(report, DurableCommitReport::default());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_verbatim_missing_journal_owner_commits_generation() {
    let logical_root = std::env::temp_dir().join(format!(
        "zircon-durable-windows-verbatim-journal-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    fs::create_dir_all(logical_root.join(".zircon")).unwrap();
    fs::create_dir_all(logical_root.join("assets")).unwrap();
    let root = fs::canonicalize(&logical_root).unwrap();
    assert!(root.to_string_lossy().starts_with(r"\\?\"));
    let journal = root.join(".zircon/project-generation");
    let target = root.join("assets/generation.zmeta");

    let mut report = DurableCommitReport::default();
    let disposition = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"first-generation".to_vec(),
        )],
        TransactionFault::None,
        &mut report,
    )
    .unwrap();

    assert_eq!(disposition, DurableCommitDisposition::Durable);
    assert_eq!(fs::read(target).unwrap(), b"first-generation");
    assert_eq!(report, DurableCommitReport::default());
    fs::remove_dir_all(logical_root).unwrap();
}

#[cfg(unix)]
#[test]
fn directory_symlink_target_aliases_are_rejected() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-directory-alias-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let physical = root.join("physical-assets");
    let alias = root.join("asset-alias");
    fs::create_dir_all(&physical).unwrap();
    std::os::unix::fs::symlink(&physical, &alias).unwrap();

    let mut report = DurableCommitReport::default();
    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(
                physical.join("generation.zmeta"),
                b"first-generation".to_vec(),
            ),
            PreparedFileWrite::new(
                alias.join("generation.zmeta"),
                b"second-generation".to_vec(),
            ),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("directory aliases must resolve to one physical target identity");

    assert!(error.to_string().contains("duplicate transaction target"));
    assert!(!journal.exists());
    assert_eq!(report, DurableCommitReport::default());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn post_replace_failure_rolls_back_the_published_target() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-post-replace-{}-{}",
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
        TransactionFault::FailAfterTargetReplace(0),
        &mut report,
    )
    .expect_err("post-replace durability failure must fail the transaction");

    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    assert_eq!(report.rollback_restore_attempt_count(), 1);
    assert_eq!(report.rollback_restore_success_count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_pending_generation_blocks_a_second_transaction_for_the_same_owner() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-pending-owner-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&target, b"old-generation").unwrap();
    let mut interrupted_report = DurableCommitReport::default();
    commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"interrupted-generation".to_vec(),
        )],
        TransactionFault::CrashAfterStaging(0),
        &mut interrupted_report,
    )
    .expect_err("the first transaction must retain pending recovery");

    let mut blocked_report = DurableCommitReport::default();
    let error = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"forbidden-generation".to_vec(),
        )],
        TransactionFault::None,
        &mut blocked_report,
    )
    .expect_err("pending recovery must block a second transaction");

    assert!(error.to_string().contains("pending recovery"));
    assert_eq!(interrupted_report, DurableCommitReport::default());
    assert_eq!(blocked_report, DurableCommitReport::default());
    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn staging_directory_sync_failure_aborts_before_publication() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-staging-directory-sync-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&target, b"old-generation").unwrap();

    let mut report = DurableCommitReport::default();
    let error = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::FailStagingDirectorySync(0),
        &mut report,
    )
    .expect_err("prepared evidence cannot precede the staging directory barrier");

    assert!(error.to_string().contains("staging directory sync"));
    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    assert!(fs::read_dir(&root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("zr-project-stage")
    }));
    fs::remove_dir_all(root).unwrap();
}

fn test_directory(name: &str) -> std::path::PathBuf {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("resolve current workspace for durable transaction test output")
                .join("target")
        });
    output_root.join("zircon-test-output").join(format!(
        "zircon-durable-engine-{name}-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ))
}

#[test]
fn intent_recovery_cleanup_preserves_evidence_when_the_journal_cannot_be_removed() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-intent-cleanup-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal-as-directory");
    let staging = root.join("generation.stage");
    fs::create_dir_all(&journal).unwrap();
    fs::write(&staging, b"staged-generation").unwrap();
    let documents = vec![super::super::schema::JournalDocument {
        state: JournalState::Prepared,
        target_existed: Some(false),
        original_digest: None,
        new_digest: Some("staged-digest".to_owned()),
        retired_digests: Vec::new(),
        target: root.join("generation.zmeta"),
        staging: staging.clone(),
        backup: root.join("generation.backup"),
        rollback_staging: root.join("generation.rollback"),
        retirements: Vec::new(),
    }];

    super::super::commit::cleanup_documents_journal_first(&journal, &documents)
        .expect_err("journal removal failure must stop unpublished artifact cleanup");

    assert!(staging.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_restore_reports_attempt_without_claiming_success() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-restore-report-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let first = root.join("first.zmeta");
    let second = root.join("second.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&first, b"old-first").unwrap();
    fs::write(&second, b"old-second").unwrap();

    let mut report = DurableCommitReport::default();
    commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(first, b"new-first".to_vec()),
            PreparedFileWrite::new(second, b"new-second".to_vec()),
        ],
        TransactionFault::RestoreFailure {
            commit_index: 1,
            restore_index: 0,
        },
        &mut report,
    )
    .expect_err("injected restore failure must leave restart recovery evidence");

    assert_eq!(report.rollback_restore_attempt_count(), 1);
    assert_eq!(report.rollback_restore_success_count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cleanup_transition_failure_preserves_the_committed_generation() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-cleanup-transition-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&target, b"old-generation").unwrap();

    let mut report = DurableCommitReport::default();
    let disposition = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::FailCleanupTransition,
        &mut report,
    )
    .expect("cleanup transition failure is after the durable commit point");

    assert_eq!(disposition, DurableCommitDisposition::CleanupDeferred);
    assert_eq!(fs::read(&target).unwrap(), b"new-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);
    assert_eq!(report.deferred_cleanup_count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn committed_artifact_cleanup_failure_preserves_the_committed_generation() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-committed-cleanup-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&target, b"old-generation").unwrap();

    let mut report = DurableCommitReport::default();
    let disposition = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::FailCommittedCleanup,
        &mut report,
    )
    .expect("artifact cleanup failure is after the durable commit point");

    assert_eq!(disposition, DurableCommitDisposition::CleanupDeferred);
    assert_eq!(fs::read(&target).unwrap(), b"new-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);
    assert_eq!(report.deferred_cleanup_count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rollback_transition_failure_falls_back_to_cleanup_after_live_restoration() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-rollback-transition-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let first = root.join("first.zmeta");
    let second = root.join("second.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&first, b"old-first").unwrap();
    fs::write(&second, b"old-second").unwrap();

    let mut report = DurableCommitReport::default();
    commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(first.clone(), b"new-first".to_vec()),
            PreparedFileWrite::new(second.clone(), b"new-second".to_vec()),
        ],
        TransactionFault::FailRollbackTransition {
            commit_index: 1,
            restore_index: 0,
        },
        &mut report,
    )
    .expect_err("rollback transition failure must retain restart recovery evidence");

    assert_eq!(fs::read(&first).unwrap(), b"old-first");
    assert_eq!(fs::read(&second).unwrap(), b"old-second");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    assert_eq!(report.rollback_restore_attempt_count(), 1);
    assert_eq!(report.rollback_restore_success_count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_accepts_consumed_staging_after_rollback_completed() {
    struct AcceptAll;

    impl super::super::recovery::RecoveryPolicy for AcceptAll {
        fn validate_document(
            &self,
            _journal_path: &std::path::Path,
            _document: &super::super::schema::JournalDocument,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    let root = std::env::temp_dir().join(format!(
        "zircon-durable-rollback-completed-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let first = root.join("first.zmeta");
    let second = root.join("second.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&first, b"old-first").unwrap();
    fs::write(&second, b"old-second").unwrap();

    let mut report = DurableCommitReport::default();
    commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(first.clone(), b"new-first".to_vec()),
            PreparedFileWrite::new(second.clone(), b"new-second".to_vec()),
        ],
        TransactionFault::CrashAfterRollbackCompleted { commit_index: 1 },
        &mut report,
    )
    .expect_err("the interruption must retain rollback-completed cleanup evidence");

    assert_eq!(fs::read(&first).unwrap(), b"old-first");
    assert_eq!(fs::read(&second).unwrap(), b"old-second");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);

    let recovered =
        super::super::recovery::recover_pending_transactions(&journal, "project", &mut AcceptAll)
            .expect("rollback-completed recovery permits consumed staging artifacts");

    assert_eq!(recovered.rollback_count(), 0);
    assert_eq!(recovered.cleanup_count(), 1);
    assert_eq!(fs::read(&first).unwrap(), b"old-first");
    assert_eq!(fs::read(&second).unwrap(), b"old-second");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn recovery_accepts_an_active_committed_document_already_restored_to_old_bytes() {
    struct AcceptAll;

    impl super::super::recovery::RecoveryPolicy for AcceptAll {
        fn validate_document(
            &self,
            _journal_path: &std::path::Path,
            _document: &super::super::schema::JournalDocument,
        ) -> Result<(), String> {
            Ok(())
        }
    }

    let root = std::env::temp_dir().join(format!(
        "zircon-durable-active-committed-old-{}-{}",
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
        TransactionFault::CrashAfterCommit(0),
        &mut report,
    )
    .expect_err("the interruption must retain an active committed journal");
    fs::write(&target, b"old-generation").unwrap();

    let recovered =
        super::super::recovery::recover_pending_transactions(&journal, "project", &mut AcceptAll)
            .expect("active rollback evidence accepts either visible generation");

    assert_eq!(recovered.rollback_count(), 1);
    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn incomplete_commit_point_write_rolls_back_the_generation() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-commit-point-write-{}-{}",
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
        TransactionFault::FailCommitPointWrite,
        &mut report,
    )
    .expect_err("an incomplete commit marker must leave the transaction uncommitted");

    assert_eq!(fs::read(&target).unwrap(), b"old-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    assert_eq!(report.rollback_restore_attempt_count(), 1);
    assert_eq!(report.rollback_restore_success_count(), 1);
    assert_eq!(report.deferred_commit_recovery_count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unsynced_complete_commit_point_keeps_the_live_generation_and_recovery_evidence() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-commit-point-sync-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    let target = root.join("generation.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&target, b"old-generation").unwrap();

    let mut report = DurableCommitReport::default();
    let disposition = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::FailCommitPointSync,
        &mut report,
    )
    .expect("a complete commit marker keeps the visible generation until restart arbitration");

    assert_eq!(
        disposition,
        DurableCommitDisposition::CommitRecoveryDeferred
    );
    assert_eq!(fs::read(&target).unwrap(), b"new-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);
    assert_eq!(report.deferred_commit_recovery_count(), 1);
    assert_eq!(report.deferred_cleanup_count(), 0);
    fs::remove_dir_all(root).unwrap();
}
