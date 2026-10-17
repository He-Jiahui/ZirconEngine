#![cfg(windows)]

use super::*;
use crate::io::source_revision::tests::fixture;
use crate::io::transaction::{recover_pending_transactions, JournalDocument, RecoveryPolicy};

fn publish(
    root: &Path,
    expected: SourceRevision,
    bytes: &[u8],
    fault: TransactionFault,
) -> Result<DurableCommitReceipt, DurableTransactionError> {
    commit_prepared_files_with_revisions(
        &root.join("journal"),
        "editor182",
        vec![
            PreparedFileWrite::new(root.join("scene"), bytes.to_vec()).expecting_source(expected),
            PreparedFileWrite::new(root.join("workspace"), b"workspace-new".to_vec()),
        ],
        fault,
        &mut DurableCommitReport::default(),
    )
}

#[test]
fn stale_expected_revision_preserves_all_live_targets() {
    let fixture = fixture("stale");
    let source = fixture.0.join("scene");
    fs::write(&source, b"AAAA").unwrap();
    fs::write(fixture.0.join("workspace"), b"workspace-old").unwrap();
    let expected = SourceRevision::observe(&source).unwrap();
    let modified = fs::metadata(&source).unwrap().modified().unwrap();
    fs::write(&source, b"BBBB").unwrap();
    let file = fs::File::options().write(true).open(&source).unwrap();
    file.set_modified(modified).unwrap();
    drop(file);
    let error = publish(&fixture.0, expected, b"local", TransactionFault::None).unwrap_err();
    assert!(matches!(
        error,
        DurableTransactionError::SourceConflict { .. }
    ));
    assert_eq!(fs::read(&source).unwrap(), b"BBBB");
    assert_eq!(
        fs::read(fixture.0.join("workspace")).unwrap(),
        b"workspace-old"
    );
}

#[test]
fn competing_expected_writers_commit_only_one_revision() {
    let fixture = fixture("writers");
    let path = fixture.0.join("scene");
    fs::write(&path, b"base").unwrap();
    let expected = SourceRevision::observe(&path).unwrap();
    // Both writers prepared against the same base before either publication.
    let first = publish(
        &fixture.0,
        expected.clone(),
        b"first",
        TransactionFault::None,
    )
    .unwrap();
    let second = publish(
        &fixture.0,
        expected.clone(),
        b"second",
        TransactionFault::None,
    )
    .unwrap_err();
    assert!(matches!(
        second,
        DurableTransactionError::SourceConflict { .. }
    ));
    assert_eq!(fs::read(&path).unwrap(), b"first");
    let (_, revisions) = first.into_parts();
    assert!(revisions[0].1.matches_path(&path).unwrap());
    let retry = publish(
        &fixture.0,
        revisions[0].1.clone(),
        b"retry",
        TransactionFault::None,
    )
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"retry");
    drop(retry);
}

#[test]
fn expected_missing_never_replaces_a_late_target() {
    let fixture = fixture("missing");
    let write = PreparedFileWrite::new(fixture.0.join("scene"), b"local".to_vec())
        .expecting_source(SourceRevision::Missing);
    fs::write(fixture.0.join("scene"), b"late creator").unwrap();
    let result = commit_prepared_files_with_revisions(
        &fixture.0.join("journal"),
        "editor182",
        vec![write],
        TransactionFault::None,
        &mut DurableCommitReport::default(),
    );
    assert!(matches!(
        result,
        Err(DurableTransactionError::SourceConflict { .. })
    ));
    assert_eq!(fs::read(fixture.0.join("scene")).unwrap(), b"late creator");
}

#[test]
fn receipt_identifies_the_staged_object() {
    let fixture = fixture("receipt");
    let path = fixture.0.join("scene");
    fs::write(&path, b"same").unwrap();
    let before = SourceRevision::observe(&path).unwrap();
    let receipt = publish(&fixture.0, before.clone(), b"same", TransactionFault::None).unwrap();
    let (_, revisions) = receipt.into_parts();
    let published = &revisions[0].1;
    assert_ne!(
        published, &before,
        "equal bytes do not erase the replacement file identity"
    );
    assert!(published.matches_path(&path).unwrap());
    crate::io::atomic_write(&path, b"same").unwrap();
    assert!(!published.matches_path(&path).unwrap());
}

#[test]
fn metadata_witness_does_not_block_windows_replace() {
    let fixture = fixture("witness");
    let path = fixture.0.join("scene");
    fs::write(&path, b"old").unwrap();
    let expected = SourceRevision::observe(&path).unwrap();
    // Keep both the loaded object's witness and the publisher's staged witness alive.
    let retained = expected.clone();
    let receipt = publish(&fixture.0, expected, b"new", TransactionFault::None)
        .expect("zero-access FileIdInfo and ReplaceFileW compatibility must be native-validated");
    let (_, revisions) = receipt.into_parts();
    assert!(revisions[0].1.matches_path(&path).unwrap());
    assert!(!retained.matches_path(&path).unwrap());
}

#[test]
fn prepublication_fault_preserves_source_and_companion() {
    for fault in [
        TransactionFault::FailStageWrite(0),
        TransactionFault::BeforeCommit(0),
    ] {
        let fixture = fixture("pre-fault");
        fs::write(fixture.0.join("scene"), b"old").unwrap();
        fs::write(fixture.0.join("workspace"), b"workspace-old").unwrap();
        let expected = SourceRevision::observe(&fixture.0.join("scene")).unwrap();
        assert!(publish(&fixture.0, expected, b"new", fault).is_err());
        assert_eq!(fs::read(fixture.0.join("scene")).unwrap(), b"old");
        assert_eq!(
            fs::read(fixture.0.join("workspace")).unwrap(),
            b"workspace-old"
        );
    }
}

struct AllowFixture;
impl RecoveryPolicy for AllowFixture {
    fn validate_document(&self, _: &Path, _: &JournalDocument) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn postpublication_fault_keeps_recovery_semantics() {
    for fault in [
        TransactionFault::CrashAfterTargetReplace(0),
        TransactionFault::FailCommitPointSync,
    ] {
        let fixture = fixture("post-fault");
        let path = fixture.0.join("scene");
        fs::write(&path, b"old").unwrap();
        fs::write(fixture.0.join("workspace"), b"workspace-old").unwrap();
        let expected = SourceRevision::observe(&path).unwrap();
        let result = publish(&fixture.0, expected, b"new", fault);
        match fault {
            TransactionFault::FailCommitPointSync => {
                let receipt = result.expect("visible publication keeps its typed deferred receipt");
                assert_eq!(
                    receipt.disposition,
                    DurableCommitDisposition::CommitRecoveryDeferred
                );
                assert!(receipt.revisions[0].1.matches_path(&path).unwrap());
                assert_eq!(fs::read(&path).unwrap(), b"new");
                assert_eq!(
                    fs::read(fixture.0.join("workspace")).unwrap(),
                    b"workspace-new"
                );
            }
            _ => assert!(result.is_err()),
        }
        recover_pending_transactions(&fixture.0.join("journal"), "editor182", &mut AllowFixture)
            .unwrap();
        let pair = (
            fs::read(&path).unwrap(),
            fs::read(fixture.0.join("workspace")).unwrap(),
        );
        assert!(
            pair == (b"old".to_vec(), b"workspace-old".to_vec())
                || pair == (b"new".to_vec(), b"workspace-new".to_vec())
        );
    }
}
