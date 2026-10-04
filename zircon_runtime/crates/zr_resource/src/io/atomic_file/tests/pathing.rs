use std::fs;

use super::*;
use crate::io::ArtifactIdentityExhausted;

fn test_directory(label: &str) -> PathBuf {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    output_root.join("zircon-test-output").join(format!(
        "atomic-identity-{label}-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ))
}

#[test]
fn stale_candidate_advances_to_the_next_checked_identity() {
    let directory = test_directory("collision-advance");
    fs::create_dir_all(&directory).unwrap();
    let target = directory.join("asset.bin");
    let stale = directory.join(format!(
        ".asset.bin.zr-staging-{}-{}",
        std::process::id(),
        u64::MAX - 1
    ));
    fs::write(&stale, b"stale").unwrap();
    let sequence = ArtifactSequence::starting_at(u64::MAX - 1);

    let candidate =
        unique_sibling_path_with_sequence(&directory, &target, "staging", &sequence).unwrap();

    assert!(candidate.ends_with(format!(
        ".asset.bin.zr-staging-{}-{}",
        std::process::id(),
        u64::MAX
    )));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn final_candidate_collision_returns_typed_exhaustion() {
    let directory = test_directory("terminal-collision");
    fs::create_dir_all(&directory).unwrap();
    let target = directory.join("asset.bin");
    let stale = directory.join(format!(
        ".asset.bin.zr-backup-{}-{}",
        std::process::id(),
        u64::MAX
    ));
    fs::write(&stale, b"stale").unwrap();
    let sequence = ArtifactSequence::starting_at(u64::MAX);

    let error = unique_sibling_path_with_sequence(&directory, &target, "backup", &sequence)
        .expect_err("the allocator must not wrap after the final collision");

    assert!(error
        .get_ref()
        .and_then(|source| source.downcast_ref::<ArtifactIdentityExhausted>())
        .is_some());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn atomic_transaction_recognizer_rejects_zero_sequence() {
    assert!(!is_atomic_write_transaction_path(Path::new(
        ".asset.bin.zr-staging-42-0"
    )));
    assert!(is_atomic_write_transaction_path(Path::new(
        ".asset.bin.zr-staging-42-1"
    )));
}
