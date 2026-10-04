use super::*;
use crate::io::transaction::schema::JournalIntent;

#[test]
fn prepublication_conflict_preserves_external_target_and_skips_rollback() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-prepublication-conflict-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    fs::create_dir_all(&root).unwrap();
    let target = root.join("generation.zmeta");
    let staging = root.join("generation.stage");
    fs::write(&staging, b"transaction-generation").unwrap();

    let mut staged = StagedFile {
        intent: JournalIntent {
            target: target.clone(),
            staging,
            backup: root.join("generation.backup"),
            rollback_staging: root.join("generation.rollback"),
            retirements: Vec::new(),
        },
        target_existed: false,
        original_digest: None,
        new_digest: String::new(),
        retired_digests: Vec::new(),
        committed: false,
    };

    // This write represents a non-cooperating creator after preparation but before publish.
    fs::write(&target, b"external-generation").unwrap();
    let error = commit_file(&mut staged, TransactionFault::None, 0).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert!(!staged.committed);
    assert_eq!(fs::read(&target).unwrap(), b"external-generation");
    fs::remove_dir_all(root).unwrap();
}
