use super::*;
use serde_json::json;

fn target(mode: &str) -> NativePluginArtifactTarget {
    serde_json::from_value(json!({
        "runtime_mode": mode,
        "platform": "windows"
    }))
    .unwrap()
}

#[test]
fn package_service_protocol_accepts_only_v2_targeted_queries() {
    let inventory = br#"{"action":"inventory","schemaVersion":2,"target":{"runtime_mode":"editor_host","platform":"windows"},"identityDigest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#;
    let request: Request = serde_json::from_slice(inventory).unwrap();
    assert_eq!(
        line_request_target(&request).unwrap(),
        &target("editor_host")
    );

    let receipt = br#"{"action":"receipt","schemaVersion":2,"target":{"runtime_mode":"client_runtime","platform":"windows"},"identityDigest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","operationId":"3298de17-8f1a-4e04-a28e-55c1ff43f2d8"}"#;
    let request: Request = serde_json::from_slice(receipt).unwrap();
    assert_eq!(
        line_request_target(&request).unwrap(),
        &target("client_runtime")
    );

    for bytes in [
        br#"{"action":"inventory","identityDigest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#.as_slice(),
        br#"{"action":"inventory","schemaVersion":1,"target":{"runtime_mode":"editor_host","platform":"windows"},"identityDigest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        br#"{"action":"inventory","schemaVersion":2,"target":{"runtime_mode":"server_runtime","platform":"windows"},"identityDigest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        br#"{"action":"inventory","schemaVersion":2,"target":{"runtime_mode":"editor_host","platform":"windows"},"identityDigest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","token":"forbidden"}"#,
        br#"{"action":"activate","schemaVersion":2,"target":{"runtime_mode":"editor_host","platform":"windows"}}"#,
    ] {
        if let Ok(request) = serde_json::from_slice::<Request>(bytes) {
            assert!(line_request_target(&request).is_err());
        } else {
            assert!(serde_json::from_slice::<Request>(bytes).is_err());
        }
    }
}

#[test]
fn project_lock_requests_are_bounded_and_targeted() {
    assert!(validate_project_lock_request(
        PROJECT_LOCK_REQUEST_SCHEMA_VERSION_V1,
        &target("client_runtime"),
        Path::new("C:/Projects/Example"),
    )
    .is_ok());
    assert!(validate_project_lock_request(
        PROJECT_LOCK_REQUEST_SCHEMA_VERSION_V1,
        &target("server_runtime"),
        Path::new("C:/Projects/Example"),
    )
    .is_err());
    assert!(validate_project_lock_request(
        PROJECT_LOCK_REQUEST_SCHEMA_VERSION_V1,
        &target("client_runtime"),
        Path::new("C:/Projects/../Example"),
    )
    .is_err());
}

#[test]
fn helper_digest_validation_is_canonical_lowercase_sha256() {
    assert!(valid_digest(&"a".repeat(64)));
    assert!(!valid_digest(&"A".repeat(64)));
    assert!(!valid_digest(&"a".repeat(63)));
}

#[test]
fn incomplete_control_lines_remain_rejected() {
    assert!(line(&mut &b"commit"[..]).is_err());
    assert!(line(&mut vec![b'x'; 65537].as_slice()).is_err());
    assert_eq!(line(&mut &b"commit\n"[..]).unwrap(), b"commit\n");
}
