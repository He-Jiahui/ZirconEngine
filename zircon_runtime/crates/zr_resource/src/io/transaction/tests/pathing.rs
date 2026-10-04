use super::*;

const OPAQUE_TRANSACTION_ID: &str =
    "0000000000000000000000000000000000000000000000000000000000000000-1-1";

fn test_directory(label: &str) -> PathBuf {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    output_root.join("zircon-test-output").join(format!(
        "transaction-identity-{label}-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ))
}

#[test]
fn transaction_identity_is_partitioned_by_canonical_journal_owner() {
    let root = test_directory("owner-partition");
    let first_owner = root.join("first-owner");
    let second_owner = root.join("second-owner");
    fs::create_dir_all(&first_owner).unwrap();
    fs::create_dir_all(&second_owner).unwrap();
    let first_sequence = ArtifactSequence::starting_at(7);
    let second_sequence = ArtifactSequence::starting_at(7);
    let first_identity = PathIdentity::resolve(&first_owner).unwrap();
    let second_identity = PathIdentity::resolve(&second_owner).unwrap();
    let first = next_transaction_id_with_sequence(first_identity.operation_path(), &first_sequence)
        .unwrap();
    let second =
        next_transaction_id_with_sequence(second_identity.operation_path(), &second_sequence)
            .unwrap();

    assert_ne!(first, second);
    assert!(valid_transaction_id(&first, &first_identity));
    assert!(valid_transaction_id(&second, &second_identity));
    assert!(!valid_transaction_id(&first, &second_identity));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn transaction_identity_parser_rejects_legacy_and_malformed_wires() {
    let owner = test_directory("wire-validation");
    fs::create_dir_all(&owner).unwrap();
    let identity = PathIdentity::resolve(&owner).unwrap();
    let token = journal_owner_token(identity.operation_path());

    assert!(!valid_transaction_id("42-1", &identity));
    assert!(!valid_transaction_id(&format!("{token}-42-0"), &identity));
    assert!(!valid_transaction_id(
        &format!("{}-42-1", token.to_uppercase()),
        &identity
    ));
    assert!(!valid_transaction_id(
        &format!("{}-42-1", &token[..32]),
        &identity
    ));
    assert!(!valid_transaction_id(
        &format!("{token}-42-1-extra"),
        &identity
    ));
    assert!(valid_transaction_id(&format!("{token}-42-1"), &identity));
    fs::remove_dir_all(owner).unwrap();
}

#[test]
fn project_transaction_sibling_filter_requires_canonical_project_artifacts() {
    let source = Path::new("scene.zscene");
    for role in ["stage", "backup", "rollback-stage"] {
        let artifact = transaction_sibling(source, "project", role, OPAQUE_TRANSACTION_ID);
        assert!(
            is_project_transaction_sibling_path(&artifact),
            "{artifact:?}"
        );
    }

    let basename = basename_token(source);
    let transaction_owner = "0000000000000000000000000000000000000000000000000000000000000000";
    let canonical = format!(".{basename}.zr-project-stage-{OPAQUE_TRANSACTION_ID}");
    assert!(is_project_transaction_sibling_path(Path::new(&canonical)));

    let lookalikes = [
        format!(".{basename}.zr-editor-stage-{OPAQUE_TRANSACTION_ID}"),
        format!(".{basename}.zr-project-journal-{OPAQUE_TRANSACTION_ID}.zrjournal"),
        format!(".{basename}.zr-project-retired-backup-{OPAQUE_TRANSACTION_ID}"),
        format!(".scene.zscene.zr-project-stage-{OPAQUE_TRANSACTION_ID}"),
        format!(".{basename}.zr-project-stage-{transaction_owner}-01-1"),
        format!(".{basename}.zr-project-stage-{transaction_owner}-1-01"),
        format!(".{basename}.zr-project-stage-{transaction_owner}-1-0"),
        format!(".{basename}.zr-project-stage-{OPAQUE_TRANSACTION_ID}-copy"),
    ];
    for lookalike in lookalikes {
        assert!(
            !is_project_transaction_sibling_path(Path::new(&lookalike)),
            "unexpectedly filtered {lookalike}"
        );
    }
}

#[test]
fn transaction_identity_exhaustion_is_terminal() {
    let owner = Path::new("journal-owner");
    let sequence = ArtifactSequence::starting_at(u64::MAX);

    let final_identity = next_transaction_id_with_sequence(owner, &sequence).unwrap();

    assert!(final_identity.ends_with(&format!("-{}-{}", std::process::id(), u64::MAX)));
    assert_eq!(
        next_transaction_id_with_sequence(owner, &sequence),
        Err(ArtifactIdentityExhausted)
    );
}

#[cfg(unix)]
#[test]
fn basename_tokens_distinguish_non_unicode_basenames() {
    use std::os::unix::ffi::OsStringExt;

    let parent = Path::new("/tmp");
    let first = parent.join(OsString::from_vec(vec![b'a', 0x80]));
    let second = parent.join(OsString::from_vec(vec![b'a', 0x81]));
    let literal = parent.join("zircon.data");

    let first_artifact = transaction_sibling(&first, "project", "stage", OPAQUE_TRANSACTION_ID);
    let second_artifact = transaction_sibling(&second, "project", "stage", OPAQUE_TRANSACTION_ID);
    let literal_artifact = transaction_sibling(&literal, "project", "stage", OPAQUE_TRANSACTION_ID);

    assert_ne!(first_artifact, second_artifact);
    assert_ne!(first_artifact, literal_artifact);
    assert_ne!(second_artifact, literal_artifact);
}

#[cfg(windows)]
#[test]
fn basename_tokens_distinguish_non_unicode_basenames() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    let parent = Path::new(r"C:\zircon");
    let first = parent.join(OsString::from_wide(&[0xd800]));
    let second = parent.join(OsString::from_wide(&[0xd801]));
    let literal = parent.join("zircon.data");

    let first_artifact = transaction_sibling(&first, "project", "stage", OPAQUE_TRANSACTION_ID);
    let second_artifact = transaction_sibling(&second, "project", "stage", OPAQUE_TRANSACTION_ID);
    let literal_artifact = transaction_sibling(&literal, "project", "stage", OPAQUE_TRANSACTION_ID);

    assert_ne!(first_artifact, second_artifact);
    assert_ne!(first_artifact, literal_artifact);
    assert_ne!(second_artifact, literal_artifact);
}

#[test]
fn split_at_deepest_existing_ancestor_scans_from_leaf() {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "zircon-resource-pathing-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let existing = root.join("existing").join("branch");
    fs::create_dir_all(&existing).unwrap();
    let missing = existing.join("new").join("nested").join("asset.zmeta");

    let (ancestor, tail) = split_at_deepest_existing_ancestor(&missing).unwrap();

    assert_eq!(ancestor, existing);
    assert_eq!(
        tail,
        vec![
            OsString::from("new"),
            OsString::from("nested"),
            OsString::from("asset.zmeta"),
        ]
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn split_at_deepest_existing_ancestor_preserves_parent_components() {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "zircon-resource-parent-pathing-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id()
    ));
    let existing = root.join("existing");
    fs::create_dir_all(&existing).unwrap();
    let mut missing = existing.as_os_str().to_os_string();
    missing.push(std::path::MAIN_SEPARATOR_STR);
    missing.push("missing");
    missing.push(std::path::MAIN_SEPARATOR_STR);
    missing.push("..");
    missing.push(std::path::MAIN_SEPARATOR_STR);
    missing.push("asset.zmeta");
    let missing = PathBuf::from(missing);

    let (ancestor, tail) = split_at_deepest_existing_ancestor(&missing).unwrap();

    assert_eq!(ancestor, existing);
    assert_eq!(
        tail,
        vec![
            OsString::from("missing"),
            OsString::from(".."),
            OsString::from("asset.zmeta"),
        ]
    );
    fs::remove_dir_all(root).unwrap();
}
