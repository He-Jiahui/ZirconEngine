use std::fs;
use std::path::PathBuf;

use super::*;

#[test]
fn ancestor_and_descendant_targets_are_rejected_without_filesystem_side_effects() {
    let root = test_directory("live-ancestor-descendant");
    fs::create_dir_all(&root).unwrap();
    let parent = root.join("assets");
    let child = parent.join("child.zmeta");
    let journal = root.join("journal");
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(child, b"child".to_vec()),
            PreparedFileWrite::new(parent.clone(), b"parent".to_vec()),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("live target namespaces must be an antichain");

    assert!(error.to_string().contains("ancestor or descendant"));
    assert!(!journal.exists());
    assert!(!parent.exists());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_case_insensitive_ancestor_and_descendant_targets_are_rejected() {
    let root = test_directory("live-windows-case-ancestor-descendant");
    fs::create_dir_all(&root).unwrap();
    let parent = root.join("Assets");
    let child = root.join("assets").join("child.zmeta");
    let journal = root.join("journal");
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(child, b"child".to_vec()),
            PreparedFileWrite::new(parent.clone(), b"parent".to_vec()),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("Windows live target namespaces must be case-insensitive antichains");

    assert!(error.to_string().contains("ancestor or descendant"));
    assert!(!journal.exists());
    assert!(!parent.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn prefix_sibling_targets_remain_independent() {
    let root = test_directory("live-prefix-siblings");
    fs::create_dir_all(&root).unwrap();
    let journal = root.join("journal");
    let first = root.join("assets");
    let second = root.join("assets2");
    let mut report = DurableCommitReport::default();

    let disposition = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(first.clone(), b"first".to_vec()),
            PreparedFileWrite::new(second.clone(), b"second".to_vec()),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect("prefix siblings are distinct live paths");

    assert_eq!(disposition, DurableCommitDisposition::Durable);
    assert_eq!(fs::read(first).unwrap(), b"first");
    assert_eq!(fs::read(second).unwrap(), b"second");
    assert_eq!(fs::read_dir(journal).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn component_ordering_catches_interleaved_ancestor_targets() {
    let root = test_directory("live-component-ordering");
    fs::create_dir_all(&root).unwrap();
    let journal = root.join("journal");
    let parent = root.join("assets");
    let child = parent.join("child.zmeta");
    let interleaving_sibling = root.join("assets-child");
    let mut report = DurableCommitReport::default();

    let error = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(interleaving_sibling, b"sibling".to_vec()),
            PreparedFileWrite::new(child, b"child".to_vec()),
            PreparedFileWrite::new(parent, b"parent".to_vec()),
        ],
        TransactionFault::None,
        &mut report,
    )
    .expect_err("component ordering must retain ancestor rejection");

    assert!(error.to_string().contains("ancestor or descendant"));
    assert!(!journal.exists());
    fs::remove_dir_all(root).unwrap();
}

fn test_directory(name: &str) -> PathBuf {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    output_root.join("zircon-test-output").join(format!(
        "zircon-durable-engine-{name}-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ))
}
