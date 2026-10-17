use super::*;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

const LIMITS: TreeRemovalLimits = TreeRemovalLimits {
    entries: 64,
    directories: 32,
};

fn fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hub-anchored-tree-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

#[cfg(unix)]
#[test]
fn removal_stays_on_the_opened_tree_when_an_ancestor_is_replaced() {
    use std::os::unix::fs::symlink;

    let root = fixture("unix-swap");
    let external = fixture("unix-external");
    let private_root = root.join(".zircon/cloud/uploads");
    let operation = "00000000-0000-4000-8000-000000000001";
    fs::create_dir_all(private_root.join(operation)).unwrap();
    fs::write(private_root.join(operation).join("payload"), b"private").unwrap();
    let external_uploads = external.join("cloud/uploads");
    fs::create_dir_all(external_uploads.join(operation)).unwrap();
    let external_file = external_uploads.join(operation).join("payload");
    fs::write(&external_file, b"external must survive").unwrap();

    let anchor = AnchoredDirectory::open(&private_root).unwrap();
    let old_private = root.join(".zircon-old");
    fs::rename(root.join(".zircon"), &old_private).unwrap();
    symlink(&external, root.join(".zircon")).unwrap();

    let usage = anchor.remove_tree(OsStr::new(operation), LIMITS).unwrap();
    assert!(usage.entries >= 1);
    assert!(!old_private.join("cloud/uploads").join(operation).exists());
    assert_eq!(fs::read(external_file).unwrap(), b"external must survive");

    fs::remove_file(root.join(".zircon")).unwrap();
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}

#[cfg(windows)]
#[test]
fn held_ancestor_handles_block_path_replacement_during_removal() {
    let root = fixture("windows-swap");
    let external = fixture("windows-external");
    let private_root = root.join(".zircon/cloud/uploads");
    let operation = "00000000-0000-4000-8000-000000000001";
    fs::create_dir_all(private_root.join(operation)).unwrap();
    fs::write(private_root.join(operation).join("payload"), b"private").unwrap();
    let external_uploads = external.join("cloud/uploads");
    fs::create_dir_all(external_uploads.join(operation)).unwrap();
    let external_file = external_uploads.join(operation).join("payload");
    fs::write(&external_file, b"external must survive").unwrap();

    let anchor = AnchoredDirectory::open(&private_root).unwrap();
    assert!(fs::rename(root.join(".zircon"), root.join(".zircon-old")).is_err());
    anchor.remove_tree(OsStr::new(operation), LIMITS).unwrap();
    assert_eq!(fs::read(external_file).unwrap(), b"external must survive");
    drop(anchor);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}

#[test]
fn bounded_preflight_preserves_tree_when_limits_are_exceeded() {
    let root = fixture("budget");
    let uploads = root.join("uploads");
    let operation = "operation";
    let nested = uploads.join(operation).join("nested/deeper");
    fs::create_dir_all(&nested).unwrap();
    let payload = nested.join("payload");
    fs::write(&payload, b"keep").unwrap();
    let anchor = AnchoredDirectory::open(&uploads).unwrap();
    assert!(anchor
        .remove_tree(
            OsStr::new(operation),
            TreeRemovalLimits {
                entries: 1,
                directories: 1,
            },
        )
        .is_err());
    assert_eq!(fs::read(payload).unwrap(), b"keep");
    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn marker_identity_is_rechecked_through_the_anchored_operation_handle() {
    let root = fixture("marker-identity");
    let uploads = root.join("uploads");
    let operation = "operation";
    let directory = uploads.join(operation);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("marker.json"), b"bound operation").unwrap();
    fs::write(directory.join("payload"), b"preserve on mismatch").unwrap();
    let anchor = AnchoredDirectory::open(&uploads).unwrap();

    assert!(anchor
        .remove_tree_preserving_marker(
            OsStr::new(operation),
            OsStr::new("marker.json"),
            Some(b"different operation"),
            LIMITS,
        )
        .is_err());
    assert_eq!(
        fs::read(directory.join("payload")).unwrap(),
        b"preserve on mismatch"
    );
    assert!(directory.join("marker.json").exists());

    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_directory_probe_keeps_a_directory_with_multiple_entries() {
    let root = fixture("nonempty-probe");
    let uploads = root.join("uploads");
    let operation = "operation";
    let directory = uploads.join(operation);
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("first"), b"keep first").unwrap();
    fs::write(directory.join("second"), b"keep second").unwrap();
    let anchor = AnchoredDirectory::open(&uploads).unwrap();

    assert!(!anchor
        .remove_empty_directory_if_empty(OsStr::new(operation))
        .unwrap());
    assert_eq!(fs::read(directory.join("first")).unwrap(), b"keep first");
    assert_eq!(fs::read(directory.join("second")).unwrap(), b"keep second");

    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn null_readdir_classification_distinguishes_eof_interruption_and_error() {
    let interrupted_errno = i32::MAX;
    let other_errno = interrupted_errno - 1;
    assert_eq!(classify_readdir_null(0, interrupted_errno).unwrap(), true);
    assert_eq!(
        classify_readdir_null(interrupted_errno, interrupted_errno).unwrap(),
        false
    );
    assert_eq!(
        classify_readdir_null(other_errno, interrupted_errno)
            .unwrap_err()
            .raw_os_error(),
        Some(other_errno)
    );
}

#[test]
fn identity_bound_cleanup_finishes_only_an_empty_markerless_operation() {
    let root = fixture("markerless-retry");
    let uploads = root.join("uploads");
    let operation = "operation";
    let directory = uploads.join(operation);
    fs::create_dir_all(&directory).unwrap();
    let anchor = AnchoredDirectory::open(&uploads).unwrap();

    assert_eq!(
        anchor
            .remove_tree_preserving_marker(
                OsStr::new(operation),
                OsStr::new("marker.json"),
                Some(b"validated identity"),
                LIMITS,
            )
            .unwrap(),
        TreeRemovalUsage {
            entries: 0,
            directories: 1,
        }
    );
    assert!(!directory.exists());

    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn identity_bound_cleanup_does_not_accept_a_missing_marker_with_contents() {
    let root = fixture("markerless-nonempty");
    let uploads = root.join("uploads");
    let operation = "operation";
    let directory = uploads.join(operation);
    fs::create_dir_all(&directory).unwrap();
    let payload = directory.join("payload");
    fs::write(&payload, b"preserve").unwrap();
    let anchor = AnchoredDirectory::open(&uploads).unwrap();

    assert!(anchor
        .remove_tree_preserving_marker(
            OsStr::new(operation),
            OsStr::new("marker.json"),
            Some(b"validated identity"),
            LIMITS,
        )
        .is_err());
    assert_eq!(fs::read(payload).unwrap(), b"preserve");

    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn anchored_publication_creates_directories_and_no_clobber_links() {
    let root = fixture("anchored-publication");
    let anchor = AnchoredDirectory::open_for_directory_writes(&root).unwrap();
    let child = anchor
        .ensure_child_directory(OsStr::new("private"))
        .unwrap();
    let nested = child.ensure_child_directory(OsStr::new("nested")).unwrap();
    let mut source = nested.create_new_file(OsStr::new("temporary")).unwrap();
    std::io::Write::write_all(&mut source, b"complete payload").unwrap();
    source.sync_all().unwrap();
    drop(source);

    nested
        .hard_link(OsStr::new("temporary"), OsStr::new("published"))
        .unwrap();
    assert_eq!(
        fs::read(root.join("private/nested/published")).unwrap(),
        b"complete payload"
    );
    assert_eq!(
        nested
            .hard_link(OsStr::new("temporary"), OsStr::new("published"))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::AlreadyExists
    );
    nested.remove_file(OsStr::new("temporary")).unwrap();
    nested.remove_file(OsStr::new("published")).unwrap();

    drop(nested);
    drop(child);
    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn markerless_retry_does_not_delete_entries_created_after_empty_preflight() {
    let root = fixture("markerless-race");
    let uploads = root.join("uploads");
    let operation = "operation";
    let directory = uploads.join(operation);
    fs::create_dir_all(&directory).unwrap();
    let anchor = AnchoredDirectory::open(&uploads).unwrap();
    let late_entry = directory.join("late-entry");

    let result = anchor.remove_tree_inner_with_hook(
        OsStr::new(operation),
        Some((OsStr::new("marker.json"), Some(b"bound marker"))),
        LIMITS,
        || fs::write(&late_entry, b"preserve concurrent data").unwrap(),
    );

    assert!(result.is_err());
    assert_eq!(fs::read(&late_entry).unwrap(), b"preserve concurrent data");
    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn anchored_private_creation_uses_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let root = fixture("private-permissions");
    let anchor = AnchoredDirectory::open_for_directory_writes(&root).unwrap();
    let private = anchor
        .ensure_child_directory(OsStr::new("private"))
        .unwrap();
    let file = private.create_new_file(OsStr::new("blob")).unwrap();

    assert_eq!(
        fs::metadata(private.path()).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(file.metadata().unwrap().permissions().mode() & 0o777, 0o600);

    drop(file);
    drop(private);
    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn opening_an_existing_private_file_removes_group_and_other_access() {
    use std::os::unix::fs::PermissionsExt;

    let root = fixture("existing-private-file");
    let path = root.join("legacy-marker");
    fs::write(&path, b"private legacy bytes").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let anchor = AnchoredDirectory::open(&root).unwrap();

    let mut file = anchor
        .open_existing_private_file(OsStr::new("legacy-marker"))
        .unwrap();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes).unwrap();

    assert_eq!(bytes, b"private legacy bytes");
    assert_eq!(file.metadata().unwrap().permissions().mode() & 0o777, 0o600);
    drop(file);
    drop(anchor);
    fs::remove_dir_all(root).unwrap();
}
