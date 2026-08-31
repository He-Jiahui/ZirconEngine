use std::io;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use super::{path_entry, PathEntry};
use crate::io::artifact_identity::ArtifactSequence;

static ATOMIC_FILE_SEQUENCE: ArtifactSequence = ArtifactSequence::new();

pub(super) fn unique_sibling_path(
    directory: &Path,
    target: &Path,
    role: &str,
) -> io::Result<PathBuf> {
    unique_sibling_path_with_sequence(directory, target, role, &ATOMIC_FILE_SEQUENCE)
}

fn unique_sibling_path_with_sequence(
    directory: &Path,
    target: &Path,
    role: &str,
    sequence: &ArtifactSequence,
) -> io::Result<PathBuf> {
    let file_name = target_file_name(target);
    loop {
        let id = sequence.next().map_err(io::Error::other)?.get();
        let candidate = directory.join(format!(
            ".{file_name}.zr-{role}-{}-{id}",
            std::process::id()
        ));
        if path_entry(&candidate)? == PathEntry::Missing {
            return Ok(candidate);
        }
    }
}

pub(super) fn target_file_name(target: &Path) -> &str {
    target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data")
}

pub fn is_atomic_write_transaction_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !file_name.starts_with('.') {
        return false;
    }
    [".zr-staging-", ".zr-backup-"].into_iter().any(|marker| {
        let Some((_, suffix)) = file_name.rsplit_once(marker) else {
            return false;
        };
        let Some((process_id, sequence)) = suffix.split_once('-') else {
            return false;
        };
        !process_id.is_empty()
            && !sequence.is_empty()
            && process_id.bytes().all(|byte| byte.is_ascii_digit())
            && sequence.bytes().all(|byte| byte.is_ascii_digit())
            && sequence.parse::<NonZeroU64>().is_ok()
    })
}

#[cfg(test)]
mod tests {
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
}
