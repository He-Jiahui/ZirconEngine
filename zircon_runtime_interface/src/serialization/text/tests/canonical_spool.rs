use std::io::Write;
use std::rc::Rc;

use super::{
    canonical_spool_resource_limit, initialize_attempt_directory, SpoolAttempt, TempSpool,
    ATTEMPT_JOURNAL_FILE_NAME, ATTEMPT_JOURNAL_MAGIC, MEMORY_SPOOL_BYTES,
};

#[test]
fn small_canonical_values_stay_in_memory_without_creating_a_temp_file() {
    let root = test_root("memory");
    let attempt = Rc::new(SpoolAttempt::new_with_root(root.clone()));
    let mut spool = TempSpool::new(Rc::clone(&attempt));

    spool
        .write_all(&vec![b'x'; MEMORY_SPOOL_BYTES])
        .expect("the in-memory threshold should accept an exact-size value");
    spool
        .finish_write()
        .expect("finishing an in-memory spool should be infallible");

    assert!(spool.is_memory_backed());
    assert!(!spool.has_open_file());
    assert!(!root.exists());
}

#[test]
fn large_canonical_values_close_the_spill_file_after_serialization() {
    let root = test_root("spill");
    let attempt = Rc::new(SpoolAttempt::new_with_root(root.clone()));
    let mut spool = TempSpool::new(Rc::clone(&attempt));

    spool
        .write_all(&vec![b'x'; MEMORY_SPOOL_BYTES + 1])
        .expect("a large value should spill into the attempt directory");
    assert!(!spool.is_memory_backed());
    assert!(spool.has_open_file());

    spool
        .finish_write()
        .expect("finishing a spill must close its retained writer");
    assert!(!spool.has_open_file());

    drop(spool);
    drop(attempt);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn spilled_attempt_records_versioned_recovery_evidence() {
    let root = test_root("journal");
    let attempt = SpoolAttempt::new_with_root(root.clone());

    let (value_path, value_file) = attempt
        .allocate_file()
        .expect("a spill attempt should create its recovery journal");
    drop(value_file);
    let directory = value_path.parent().expect("attempt directory");
    let journal = std::fs::read_to_string(directory.join(ATTEMPT_JOURNAL_FILE_NAME))
        .expect("attempt journal should be readable before any recovery decision");

    assert!(journal.starts_with(ATTEMPT_JOURNAL_MAGIC));
    assert!(journal.contains("\nversion=1\n"));
    assert!(journal.contains(&format!("owner_pid={}\n", std::process::id())));
    let attempt_id = directory
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.rsplit('-').next())
        .expect("attempt id in directory name");
    assert!(journal.contains(&format!("attempt_id={attempt_id}\n")));

    drop(attempt);
    assert!(!directory.exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn failed_attempt_journal_initialization_rolls_back_the_directory() {
    let root = test_root("journal-rollback");
    let directory = root.join("attempt");
    std::fs::create_dir_all(directory.join(ATTEMPT_JOURNAL_FILE_NAME))
        .expect("fixture should block journal file creation");

    initialize_attempt_directory(&directory, 1)
        .expect_err("a directory at the journal path must reject initialization");

    assert!(!directory.exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn spill_file_count_is_bounded_per_attempt_before_directory_allocation() {
    let root = test_root("file-limit");
    let attempt = SpoolAttempt::new_with_root_and_file_limit(root.clone(), 0);

    let error = attempt
        .allocate_file()
        .expect_err("the attempt must reject a spill above its file budget");

    assert_eq!(
        canonical_spool_resource_limit(&error),
        Some(("canonical spool files", 0, 1))
    );
    assert!(!root.exists());
}

fn test_root(label: &str) -> std::path::PathBuf {
    std::env::current_dir()
        .expect("test working directory")
        .join(format!(
            "target/canonical-spool-tests/{}-{label}",
            std::process::id()
        ))
}
