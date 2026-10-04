use std::fs;

use super::*;

#[test]
fn owner_lock_rejects_a_second_live_holder() {
    let root = std::env::temp_dir().join(format!(
        "zircon-durable-owner-lock-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let journal = root.join("journal");
    fs::create_dir_all(&journal).unwrap();
    let first = TransactionOwnerLock::acquire(&journal, TransactionPhase::Stage).unwrap();

    let error = TransactionOwnerLock::acquire(&journal, TransactionPhase::Stage).unwrap_err();

    assert!(error.to_string().contains("another process owns"));
    drop(first);
    TransactionOwnerLock::acquire(&journal, TransactionPhase::Stage).unwrap();
    fs::remove_dir_all(root).unwrap();
}
