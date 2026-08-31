use std::fs;

use super::{
    commit_prepared_files, test_directory, DurableCommitDisposition, DurableCommitReport,
    PreparedFileWrite, TransactionFault,
};

#[test]
fn live_target_inside_journal_owner_is_rejected_before_materialization() {
    let root = test_directory("live-target-inside-journal-owner");
    let journal = root.join("journal");
    let target = journal.join("nested/generation.zmeta");
    fs::create_dir_all(&root).unwrap();

    let mut report = DurableCommitReport::default();
    let result = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::None,
        &mut report,
    );
    let target_was_published = target.exists();
    let journal_was_materialized = journal.exists();
    fs::remove_dir_all(&root).unwrap();

    let error = result.expect_err("the journal owner must not contain live transaction targets");
    assert!(error
        .to_string()
        .contains("transaction live path overlaps the journal owner namespace"));
    assert!(!target_was_published);
    assert!(!journal_was_materialized);
    assert_eq!(report, DurableCommitReport::default());
}

#[test]
fn retirement_inside_journal_owner_is_rejected_as_an_input_contract() {
    let root = test_directory("retirement-inside-journal-owner");
    let journal = root.join("journal");
    let retirement = journal.join("retired.asset");
    let target = root.join("published.asset");
    fs::create_dir_all(&journal).unwrap();
    fs::write(&retirement, b"retired-generation").unwrap();

    let mut report = DurableCommitReport::default();
    let result = commit_prepared_files(
        &journal,
        "project",
        vec![
            PreparedFileWrite::new(target.clone(), b"published-generation".to_vec())
                .retiring(retirement.clone()),
        ],
        TransactionFault::None,
        &mut report,
    );
    let retired_bytes = fs::read(&retirement).unwrap();
    let target_was_published = target.exists();
    fs::remove_dir_all(&root).unwrap();

    let error = result.expect_err("the journal owner must not contain retired live paths");
    assert!(error
        .to_string()
        .contains("transaction live path overlaps the journal owner namespace"));
    assert_eq!(retired_bytes, b"retired-generation");
    assert!(!target_was_published);
    assert_eq!(report, DurableCommitReport::default());
}

#[test]
fn journal_owner_inside_live_target_is_rejected_before_materialization() {
    let root = test_directory("journal-owner-inside-live-target");
    let target = root.join("generation.zmeta");
    let journal = target.join("journal");
    fs::create_dir_all(&root).unwrap();

    let mut report = DurableCommitReport::default();
    let result = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::None,
        &mut report,
    );
    let target_was_materialized = target.exists();
    fs::remove_dir_all(&root).unwrap();

    let error = result.expect_err("the journal owner must not reside below a live file path");
    assert!(error
        .to_string()
        .contains("transaction live path overlaps the journal owner namespace"));
    assert!(!target_was_materialized);
    assert_eq!(report, DurableCommitReport::default());
}

#[cfg(windows)]
#[test]
fn windows_case_alias_inside_journal_owner_is_rejected_before_materialization() {
    let root = test_directory("windows-case-alias-inside-journal-owner");
    let journal = root.join("Journal");
    let target = root.join("journal/nested/generation.zmeta");
    fs::create_dir_all(&root).unwrap();

    let mut report = DurableCommitReport::default();
    let result = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::None,
        &mut report,
    );
    let target_was_published = target.exists();
    let journal_was_materialized = journal.exists();
    fs::remove_dir_all(&root).unwrap();

    let error = result.expect_err("Windows path identity must reject case-aliased overlap");
    assert!(error
        .to_string()
        .contains("transaction live path overlaps the journal owner namespace"));
    assert!(!target_was_published);
    assert!(!journal_was_materialized);
    assert_eq!(report, DurableCommitReport::default());
}

#[cfg(windows)]
#[test]
fn windows_journal_name_prefix_sibling_commits_generation() {
    let root = test_directory("windows-journal-name-prefix-sibling");
    let journal = root.join("journal");
    let target = root.join("journal-old/generation.zmeta");
    fs::create_dir_all(&root).unwrap();

    let mut report = DurableCommitReport::default();
    let disposition = commit_prepared_files(
        &journal,
        "project",
        vec![PreparedFileWrite::new(
            target.clone(),
            b"new-generation".to_vec(),
        )],
        TransactionFault::None,
        &mut report,
    )
    .expect("a path string prefix without a separator is a sibling namespace");

    assert_eq!(disposition, DurableCommitDisposition::Durable);
    assert_eq!(fs::read(&target).unwrap(), b"new-generation");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 0);
    fs::remove_dir_all(&root).unwrap();
}
