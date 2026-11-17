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

#[test]
fn short_candidates_keep_exact_legacy_spelling() {
    for role in ["staging", "backup"] {
        let name = sibling_file_name(Path::new("asset.bin"), role, 42, NonZeroU64::MIN).unwrap();
        assert_eq!(name, format!(".asset.bin.zr-{role}-42-1"));
        assert!(is_atomic_write_transaction_path_legacy(Path::new(&name)));
        assert!(!is_compact_journal_staging_path(Path::new(&name)));
    }
}

#[cfg(any(windows, unix))]
#[test]
fn exact_component_budget_preserves_fit_and_compacts_overflow_at_full_wire() {
    for (role, overhead, compact_length) in [("staging", 44, 109), ("backup", 43, 108)] {
        let leaf = "a".repeat(255 - overhead);
        let fit = sibling_file_name(
            Path::new(&leaf),
            role,
            u32::MAX,
            NonZeroU64::new(u64::MAX).unwrap(),
        )
        .unwrap();
        assert_eq!(fit, format!(".{leaf}.zr-{role}-{}-{}", u32::MAX, u64::MAX));
        assert_eq!(fit.len(), 255);

        let overflow_leaf = format!("{leaf}b");
        let compact = sibling_file_name(
            Path::new(&overflow_leaf),
            role,
            u32::MAX,
            NonZeroU64::new(u64::MAX).unwrap(),
        )
        .unwrap();
        let owner = native_leaf_token(Path::new(&overflow_leaf)).unwrap();
        assert_eq!(
            compact,
            format!(".{owner}.zr2-{role}-{}-{}", u32::MAX, u64::MAX)
        );
        assert_eq!(compact.len(), compact_length);
        assert!(is_atomic_write_transaction_path(Path::new(&compact)));
        assert!(!is_atomic_write_transaction_path_legacy(Path::new(
            &compact
        )));
    }
}

#[cfg(any(windows, unix))]
#[test]
fn sequence_digit_growth_rebudgets_each_candidate() {
    for (role, overhead) in [("staging", 17), ("backup", 16)] {
        let leaf = "a".repeat(255 - overhead);
        let nine =
            sibling_file_name(Path::new(&leaf), role, 42, NonZeroU64::new(9).unwrap()).unwrap();
        let ten =
            sibling_file_name(Path::new(&leaf), role, 42, NonZeroU64::new(10).unwrap()).unwrap();
        assert_eq!(nine, format!(".{leaf}.zr-{role}-42-9"));
        assert_eq!(nine.len(), 255);
        assert_eq!(
            ten,
            format!(
                ".{}.zr2-{role}-42-10",
                native_leaf_token(Path::new(&leaf)).unwrap()
            )
        );
    }
}

#[cfg(any(windows, unix))]
#[test]
fn fit_collision_at_nine_advances_to_compact_ten_without_touching_old_candidate() {
    let directory = test_directory("nine-to-ten");
    fs::create_dir_all(&directory).unwrap();
    let pid = std::process::id();
    let leaf = "a".repeat(255 - 14 - pid.to_string().len() - 1);
    let target = directory.join(&leaf);
    let stale = directory.join(format!(".{leaf}.zr-staging-{pid}-9"));
    fs::write(&stale, b"stale legacy staging").unwrap();
    let sequence = ArtifactSequence::starting_at(9);

    let candidate =
        unique_sibling_path_with_sequence(&directory, &target, "staging", &sequence).unwrap();

    assert_eq!(
        candidate,
        directory.join(format!(
            ".{}.zr2-staging-{pid}-10",
            native_leaf_token(&target).unwrap()
        ))
    );
    assert_eq!(fs::read(&stale).unwrap(), b"stale legacy staging");
    assert!(!candidate.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[cfg(any(windows, unix))]
#[test]
fn compact_collision_advances_to_last_identity_and_preserves_evidence() {
    let directory = test_directory("compact-collision-advance");
    fs::create_dir_all(&directory).unwrap();
    let target = directory.join("a".repeat(255));
    let stale = directory.join(
        sibling_file_name(
            &target,
            "staging",
            std::process::id(),
            NonZeroU64::new(u64::MAX - 1).unwrap(),
        )
        .unwrap(),
    );
    fs::write(&stale, b"stale compact staging").unwrap();
    let sequence = ArtifactSequence::starting_at(u64::MAX - 1);

    let candidate =
        unique_sibling_path_with_sequence(&directory, &target, "staging", &sequence).unwrap();

    assert_eq!(
        candidate.file_name().unwrap(),
        format!(
            ".{}.zr2-staging-{}-{}",
            native_leaf_token(&target).unwrap(),
            std::process::id(),
            u64::MAX
        )
        .as_str()
    );
    assert!(sequence.next().is_err());
    assert_eq!(fs::read(&stale).unwrap(), b"stale compact staging");
    fs::remove_dir_all(directory).unwrap();
}

#[cfg(any(windows, unix))]
#[test]
fn compact_final_file_or_directory_collision_returns_typed_exhaustion() {
    for is_directory in [false, true] {
        let directory = test_directory("compact-terminal-collision");
        fs::create_dir_all(&directory).unwrap();
        let target = directory.join("a".repeat(255));
        let stale = directory.join(
            sibling_file_name(
                &target,
                "backup",
                std::process::id(),
                NonZeroU64::new(u64::MAX).unwrap(),
            )
            .unwrap(),
        );
        if is_directory {
            fs::create_dir(&stale).unwrap();
        } else {
            fs::write(&stale, b"terminal recovery evidence").unwrap();
        }
        let sequence = ArtifactSequence::starting_at(u64::MAX);

        let error = unique_sibling_path_with_sequence(&directory, &target, "backup", &sequence)
            .expect_err("compact collisions must not wrap or replace evidence");

        assert!(error
            .get_ref()
            .and_then(|source| source.downcast_ref::<ArtifactIdentityExhausted>())
            .is_some());
        if is_directory {
            assert!(stale.is_dir());
        } else {
            assert_eq!(fs::read(&stale).unwrap(), b"terminal recovery evidence");
        }
        fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(windows)]
#[test]
fn utf16_fitting_legacy_name_is_not_compacted_for_its_utf8_size() {
    let directory = test_directory("utf16-legacy-budget");
    fs::create_dir_all(&directory).unwrap();
    let leaf = "é".repeat(211);
    let legacy = sibling_file_name(
        Path::new(&leaf),
        "staging",
        u32::MAX,
        NonZeroU64::new(u64::MAX).unwrap(),
    )
    .unwrap();
    assert_eq!(
        legacy,
        format!(".{leaf}.zr-staging-{}-{}", u32::MAX, u64::MAX)
    );
    assert_eq!(legacy.encode_utf16().count(), 255);
    assert!(legacy.len() > 255);
    let path = directory.join(legacy);
    fs::write(&path, b"native unit budget").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"native unit budget");
    fs::remove_dir_all(directory).unwrap();
}

#[cfg(any(windows, unix))]
#[test]
fn full_native_leaf_hash_distinguishes_same_prefix_different_tails() {
    for prefix in ["a".repeat(254), "é".repeat(254)] {
        let first = PathBuf::from(format!("{prefix}x"));
        let second = PathBuf::from(format!("{prefix}y"));
        let first_token = native_leaf_token(&first).unwrap();
        assert_ne!(first_token, native_leaf_token(&second).unwrap());
        assert_eq!(first_token, native_hash_oracle(first.file_name().unwrap()));
        assert_eq!(
            first_token,
            native_leaf_token(&Path::new("other-parent").join(&first)).unwrap()
        );
        let compact = sibling_file_name(&first, "backup", u32::MAX, NonZeroU64::MIN).unwrap();
        assert_eq!(compact, format!(".{first_token}.zr2-backup-{}-1", u32::MAX));
    }
    assert!(native_leaf_token(Path::new("/")).is_none());
}

#[cfg(any(windows, unix))]
fn native_hash_oracle(name: &std::ffi::OsStr) -> String {
    #[cfg(windows)]
    let bytes: Vec<u8> = {
        use std::os::windows::ffi::OsStrExt;
        name.encode_wide().flat_map(u16::to_le_bytes).collect()
    };
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt;
        name.as_bytes().to_vec()
    };
    blake3::hash(&bytes).to_hex().to_string()
}

#[cfg(any(windows, unix))]
#[test]
fn raw_native_hash_never_uses_lossy_text_or_legacy_fallback() {
    #[cfg(windows)]
    let raw_names = {
        use std::os::windows::ffi::OsStringExt;
        [0xd800, 0xd801].map(|tail| {
            let mut units = vec![b'a' as u16; 254];
            units.push(tail);
            std::ffi::OsString::from_wide(&units)
        })
    };
    #[cfg(unix)]
    let raw_names = {
        use std::os::unix::ffi::OsStringExt;
        [0xfe, 0xff].map(|tail| {
            let mut bytes = vec![b'a'; 254];
            bytes.push(tail);
            std::ffi::OsString::from_vec(bytes)
        })
    };
    let targets = raw_names.map(PathBuf::from);
    let fallback = native_leaf_token(Path::new("zircon.data")).unwrap();
    let tokens = targets
        .each_ref()
        .map(|target| native_leaf_token(target).unwrap());
    assert_ne!(tokens[0], tokens[1]);
    for (target, token) in targets.iter().zip(tokens) {
        assert_ne!(token, fallback);
        assert_eq!(token, native_hash_oracle(target.file_name().unwrap()));
        assert_eq!(target_file_name(target), "zircon.data");
        assert_eq!(
            sibling_file_name(target, "staging", 42, NonZeroU64::MIN).unwrap(),
            ".zircon.data.zr-staging-42-1"
        );
    }
}

#[cfg(any(windows, unix))]
#[test]
fn compact_journal_class_requires_lossless_exact_suffix_and_staging_role() {
    for suffix in [".zrjournal", ".ZRJOURNAL", ".zrjournal.data"] {
        let leaf = format!("{}{suffix}", "a".repeat(255 - suffix.len()));
        for role in ["staging", "backup"] {
            let name = sibling_file_name(Path::new(&leaf), role, 42, NonZeroU64::MIN).unwrap();
            let expected_class = suffix == ".zrjournal" && role == "staging";
            assert_eq!(
                is_compact_journal_staging_path(Path::new(&name)),
                expected_class
            );
            assert_eq!(name.contains(".zrjournal.zr2-"), expected_class);
            assert!(is_atomic_write_transaction_path(Path::new(&name)));
        }
    }
}

#[test]
fn compact_recognizer_requires_full_digest_exact_role_and_canonical_nonce() {
    let owner = "abcdef0123456789".repeat(4);
    for role in ["staging", "backup"] {
        for (pid, sequence) in [
            ("0".to_owned(), "1".to_owned()),
            (u32::MAX.to_string(), u64::MAX.to_string()),
        ] {
            let name = format!(".{owner}.zr2-{role}-{pid}-{sequence}");
            assert!(is_atomic_write_transaction_path(Path::new(&name)));
            assert!(!is_atomic_write_transaction_path_legacy(Path::new(&name)));
            assert!(!is_compact_journal_staging_path(Path::new(&name)));
        }
    }
    let journal = format!(".{owner}.zrjournal.zr2-staging-42-1");
    assert!(is_compact_journal_staging_path(Path::new(&journal)));
    let mut invalid = vec![
        format!("{owner}.zr2-staging-42-1"),
        format!(".{}.zr2-staging-42-1", &owner[..63]),
        format!(".{owner}0.zr2-staging-42-1"),
        format!(".{}.zr2-staging-42-1", owner.to_uppercase()),
        format!(".{}.zr2-staging-42-1", "g".repeat(64)),
        format!(".{owner}.zr2-rollback-42-1"),
        format!(".{owner}.zrjournal.zr2-backup-42-1"),
        format!(".{owner}.zrjournal.extra.zr2-staging-42-1"),
        format!(".{owner}.zr2-staging-42-1.extra"),
    ];
    for pid in ["", "01", "+1", "-1", "4294967296"] {
        invalid.push(format!(".{owner}.zr2-backup-{pid}-1"));
    }
    for sequence in ["", "0", "01", "+1", "-1", "18446744073709551616", "1-2"] {
        invalid.push(format!(".{owner}.zr2-staging-42-{sequence}"));
    }
    for name in invalid {
        assert!(
            !is_atomic_write_transaction_path(Path::new(&name)),
            "{name}"
        );
        assert!(!is_compact_journal_staging_path(Path::new(&name)), "{name}");
    }
    // These were accepted by v1; strict v2 validation must not retroactively change v1.
    assert!(is_atomic_write_transaction_path_legacy(Path::new(
        ".asset.bin.zr-backup-999999999999999999999-01"
    )));
}
