use super::{is_safe_run_id, is_sha256};

#[test]
fn evidence_identity_ids_and_hashes_have_stable_machine_readable_forms() {
    assert!(is_safe_run_id(
        "shader-pbr-20260825-a1b2c3d4-warm-measured-01"
    ));
    assert!(!is_safe_run_id("a"));
    assert!(!is_safe_run_id("1abc"));
    assert!(!is_safe_run_id("shader-pbr/unsafe"));
    assert!(is_sha256(&"a".repeat(64)));
    assert!(!is_sha256(&"A".repeat(64)));
}

#[cfg(windows)]
#[test]
fn evidence_transport_paths_do_not_leak_windows_verbatim_prefixes() {
    assert_eq!(
        super::evidence_transport_path(std::path::Path::new(r"\\?\E:\profile\ready.png")),
        std::path::PathBuf::from(r"E:\profile\ready.png")
    );
    assert_eq!(
        super::evidence_transport_path(std::path::Path::new(r"\\?\UNC\host\share\ready.png")),
        std::path::PathBuf::from(r"\\host\share\ready.png")
    );
}
