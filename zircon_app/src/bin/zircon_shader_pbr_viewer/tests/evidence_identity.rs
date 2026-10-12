use super::{is_safe_run_id, is_sha256, load_ready_frame_evidence_identity, parse_unique_json};
use serde_json::Value;

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

#[test]
fn provenance_json_rejects_duplicate_fields_and_trailing_records() {
    let duplicate = parse_unique_json::<Value>(
        r#"{"schema_version":3,"schema_version":3}"#,
        "duplicate provenance",
    );
    assert!(
        duplicate.is_err(),
        "duplicate provenance fields must fail closed"
    );

    let trailing = parse_unique_json::<Value>(r#"{"schema_version":3} {}"#, "trailing provenance");
    assert!(
        trailing.is_err(),
        "trailing provenance records must fail closed"
    );
}

#[test]
fn current_loader_bridge_requires_root_provided_real_ps_receipt() {
    let identity = std::env::var_os("ZR_SHADER_BRIDGE_IDENTITY");
    let hdri = std::env::var_os("ZR_SHADER_BRIDGE_HDRI");
    if identity.is_none() || hdri.is_none() {
        return;
    }
    let identity = std::path::PathBuf::from(identity.expect("identity env"));
    let hdri = std::path::PathBuf::from(hdri.expect("HDRI env"));
    let loaded = load_ready_frame_evidence_identity(&identity, &hdri)
        .expect("real PS identity transport should satisfy the private loader");
    assert_eq!(
        loaded.validation_policy,
        super::READY_FRAME_EVIDENCE_VALIDATION_POLICY
    );
    assert!(!loaded.source_manifest_sha256.is_empty());
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
