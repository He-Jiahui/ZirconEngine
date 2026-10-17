use super::support::*;
use crate::service::{cloud::CloudConfig, config::ServiceConfig};

#[test]
fn snapshot_paths_reject_escape_collisions_and_secret_or_generated_content() {
    let base = manifest(b"content");
    for path in [
        "../escape",
        "/root",
        "C:/root",
        "A\\B",
        ".env",
        "a/.env.local",
        "cache/file",
        "file.key",
        "a//b",
        "NUL.txt",
        "name.",
        "a?b",
        ".ssh/id_rsa",
        "Build/log",
        "Binaries/editor",
        "user.pfx",
    ] {
        let mut candidate = base.clone();
        candidate.files[0].path = path.into();
        assert!(candidate.validate().is_err(), "{path}");
    }
    for (left, right) in [
        ("Assets/Scene.zr", "assets/scene.zr"),
        ("Assets", "assets/scene.zr"),
    ] {
        let mut candidate = base.clone();
        candidate.files[0].path = left.into();
        let mut duplicate = candidate.files[0].clone();
        duplicate.path = right.into();
        candidate.files.push(duplicate);
        assert!(candidate.validate().is_err());
    }
    let mut candidate = base;
    candidate.ignore_policy = "unknown-policy".into();
    assert!(candidate.validate().is_err());
}

#[test]
fn canonical_manifest_digest_ignores_input_entry_order_and_rejects_links() {
    let mut left = manifest(b"content");
    let mut other = left.files[0].clone();
    other.path = "Assets/other.zr".into();
    left.files.push(other);
    let mut right = left.clone();
    right.files.reverse();
    assert_eq!(left.canonicalize().unwrap(), right.canonicalize().unwrap());
    let mut json = serde_json::to_value(left).unwrap();
    json["files"][0]["linkTarget"] = serde_json::json!("../secret");
    assert!(serde_json::from_value::<crate::service::cloud::Manifest>(json).is_err());
}

#[test]
fn cloud_configuration_requires_separate_absolute_root_and_key() {
    let files = Files::new();
    assert!(files.config.validate().is_ok());
    assert!(CloudConfig {
        root: "relative".into(),
        key_file: files.config.key_file.clone()
    }
    .validate()
    .is_err());
    assert!(CloudConfig {
        root: files.path.clone(),
        key_file: files.config.key_file.clone()
    }
    .validate()
    .is_err());
    let config = serde_json::json!({
        "bind":"127.0.0.1:7231", "database":files.path.join("test.db"),
        "issuer":"https://issuer.test", "audience":"hub", "introspection_client_id":"hub",
        "introspection_secret_file":files.path.join("oidc.secret"), "allow_loopback_http":false
    });
    assert!(serde_json::from_value::<ServiceConfig>(config).is_err());
}
