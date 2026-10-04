use serde_json::json;

use super::*;

fn target(runtime_mode: RuntimeTargetMode) -> NativePluginArtifactTarget {
    NativePluginArtifactTarget {
        runtime_mode,
        platform: ExportTargetPlatform::Windows,
    }
}

fn index(entries: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schemaVersion": 1,
        "kind": "zircon_native_plugin_policy_index",
        "entries": entries,
    }))
    .unwrap()
}

fn entry(runtime_mode: &str, root: &str, digest: &str) -> serde_json::Value {
    let path = std::env::temp_dir()
        .join(root)
        .join("policy.json")
        .to_string_lossy()
        .into_owned();
    json!({
        "target": {"runtime_mode": runtime_mode, "platform": "windows"},
        "policy_path": path,
        "policy_sha256": digest,
    })
}

fn install_selection(
    runtime_mode: &str,
    plugin_id: &str,
    identity_digest: &str,
    package_id: &str,
    release_revision: &str,
    artifact_digest: &str,
) -> serde_json::Value {
    json!({
        "target": {"runtime_mode": runtime_mode, "platform": "windows"},
        "pluginId": plugin_id,
        "identityDigest": identity_digest,
        "packageId": package_id,
        "releaseRevision": release_revision,
        "artifactDigest": artifact_digest,
    })
}

fn install_selections(entries: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "schemaVersion": 1,
        "kind": "zircon_native_plugin_installed_selections",
        "entries": entries,
    }))
    .unwrap()
}

#[test]
fn policy_index_selects_distinct_editor_and_client_context_roots() {
    let index = parse_policy_index(&index(json!([
        entry("editor_host", "editor", &"a".repeat(64)),
        entry("client_runtime", "client", &"b".repeat(64)),
    ])))
    .expect("versioned host policy index should parse");

    let editor = index
        .entry_for(&target(RuntimeTargetMode::EditorHost))
        .expect("EditorHost context should be configured");
    let client = index
        .entry_for(&target(RuntimeTargetMode::ClientRuntime))
        .expect("ClientRuntime context should be configured");

    assert_ne!(editor.policy_path, client.policy_path);
    assert_ne!(editor.policy_sha256, client.policy_sha256);
}

#[test]
fn policy_index_rejects_duplicate_contexts_and_duplicate_policy_paths() {
    let digest = "a".repeat(64);
    for entries in [
        json!([
            entry("editor_host", "editor", &digest),
            entry("editor_host", "client", &digest),
        ]),
        json!([
            entry("editor_host", "shared", &digest),
            entry("client_runtime", "shared", &digest),
        ]),
    ] {
        assert!(parse_policy_index(&index(entries)).is_err());
    }
}

#[test]
fn policy_index_rejects_unknown_schema_and_noncanonical_policy_digest() {
    let mut unsupported = serde_json::from_slice::<serde_json::Value>(&index(json!([entry(
        "editor_host",
        "editor",
        &"a".repeat(64)
    ),])))
    .unwrap();
    unsupported["schemaVersion"] = json!(2);
    assert!(parse_policy_index(&serde_json::to_vec(&unsupported).unwrap()).is_err());

    let malformed_digest = index(json!([entry("editor_host", "editor", &"A".repeat(64))]));
    assert!(parse_policy_index(&malformed_digest).is_err());
}

#[test]
fn installed_selection_index_maps_each_context_and_plugin_to_one_exact_package_identity() {
    let editor_identity = "a".repeat(64);
    let client_identity = "b".repeat(64);
    let index = parse_installed_selection_index(&install_selections(json!([
        install_selection(
            "editor_host",
            "studio.physics",
            &editor_identity,
            "3298de17-8f1a-4e04-a28e-55c1ff43f2d8",
            "7",
            &"c".repeat(64),
        ),
        install_selection(
            "client_runtime",
            "studio.physics",
            &client_identity,
            "3298de17-8f1a-4e04-a28e-55c1ff43f2d8",
            "8",
            &"d".repeat(64),
        ),
    ])))
    .expect("versioned host selection index should parse");

    let editor = index
        .entry_for(&target(RuntimeTargetMode::EditorHost), "studio.physics")
        .expect("EditorHost selection should be explicit");
    let client = index
        .entry_for(&target(RuntimeTargetMode::ClientRuntime), "studio.physics")
        .expect("ClientRuntime selection should be explicit");

    assert_eq!(editor.identity_digest, editor_identity);
    assert_eq!(editor.release_revision, "7");
    assert_eq!(editor.artifact_digest, "c".repeat(64));
    assert_eq!(client.identity_digest, client_identity);
    assert_eq!(client.release_revision, "8");
    assert_eq!(client.artifact_digest, "d".repeat(64));
    assert!(index
        .entry_for(&target(RuntimeTargetMode::EditorHost), "studio.audio")
        .is_none());
}

#[test]
fn installed_selection_index_rejects_duplicate_plugin_targets_and_untrusted_fields() {
    let valid = install_selection(
        "editor_host",
        "studio.physics",
        &"a".repeat(64),
        "3298de17-8f1a-4e04-a28e-55c1ff43f2d8",
        "7",
        &"b".repeat(64),
    );
    let duplicate = install_selections(json!([valid.clone(), valid.clone()]));
    assert!(parse_installed_selection_index(&duplicate).is_err());

    for malformed in [
        install_selection(
            "server_runtime",
            "studio.physics",
            &"a".repeat(64),
            "3298de17-8f1a-4e04-a28e-55c1ff43f2d8",
            "7",
            &"b".repeat(64),
        ),
        install_selection(
            "editor_host",
            "studio.physics",
            &"A".repeat(64),
            "3298de17-8f1a-4e04-a28e-55c1ff43f2d8",
            "7",
            &"b".repeat(64),
        ),
        install_selection(
            "editor_host",
            "studio.physics",
            &"a".repeat(64),
            "not-a-package-id",
            "7",
            &"b".repeat(64),
        ),
        install_selection(
            "editor_host",
            "studio.physics",
            &"a".repeat(64),
            "3298de17-8f1a-4e04-a28e-55c1ff43f2d8",
            "07",
            &"b".repeat(64),
        ),
    ] {
        assert!(parse_installed_selection_index(&install_selections(json!([malformed]))).is_err());
    }
}

#[test]
fn installed_selection_upsert_replaces_only_the_same_target_and_plugin() {
    let selected_target = target(RuntimeTargetMode::EditorHost);
    let other_target = target(RuntimeTargetMode::ClientRuntime);
    let mut entries = vec![
        NativePluginInstalledSelection {
            target: selected_target.clone(),
            plugin_id: "studio.physics".into(),
            identity_digest: "a".repeat(64),
            package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
            release_revision: "1".into(),
            artifact_digest: "b".repeat(64),
        },
        NativePluginInstalledSelection {
            target: other_target.clone(),
            plugin_id: "studio.physics".into(),
            identity_digest: "c".repeat(64),
            package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
            release_revision: "1".into(),
            artifact_digest: "d".repeat(64),
        },
    ];
    upsert_installed_selection(
        &mut entries,
        NativePluginInstalledSelection {
            target: selected_target,
            plugin_id: "studio.physics".into(),
            identity_digest: "e".repeat(64),
            package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
            release_revision: "2".into(),
            artifact_digest: "f".repeat(64),
        },
    );
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].identity_digest, "e".repeat(64));
    assert_eq!(entries[1].identity_digest, "c".repeat(64));
}

#[test]
fn policy_digest_check_rejects_modified_host_policy_bytes() {
    let original = b"host-owned policy";
    let expected = digest(original);
    assert!(verify_policy_digest(&expected, original).is_ok());
    assert!(verify_policy_digest(&expected, b"changed policy").is_err());
}

#[test]
fn context_roots_must_be_disjoint_trees() {
    let base = std::env::temp_dir().join("zircon-native-plugin-policy");
    let editor = base.join("editor");
    let client = base.join("client");
    assert!(paths_overlap(&base, &editor));
    assert!(!paths_overlap(&editor, &client));
}

#[test]
fn index_entry_target_must_match_the_policy_target() {
    let index = parse_policy_index(&index(json!([entry(
        "editor_host",
        "editor",
        &"a".repeat(64)
    ),])))
    .expect("host index fixture should parse");
    let entry = index.entries.first().unwrap();
    let mut policy = test_policy(RuntimeTargetMode::EditorHost);
    assert!(validate_policy_entry(entry, &policy));
    policy.target.runtime_mode = RuntimeTargetMode::ClientRuntime;
    assert!(!validate_policy_entry(entry, &policy));
}

fn test_policy(runtime_mode: RuntimeTargetMode) -> PackageHostPolicy {
    use super::super::super::NativePackageKeyPolicy;
    use chrono::{Duration, Utc};

    let now = Utc::now();
    PackageHostPolicy {
        root: std::env::temp_dir().join("zircon-native-plugin-policy-store"),
        trust_registry: json!({
            "schema_version": 1,
            "trust_registry_kind": "zircon_product_receipt_trust_registry",
            "issuers": [{
                "signer_id": "product-signer",
                "algorithm": "ed25519-v1",
                "public_key_hex": "00".repeat(32),
                "disabled": false,
            }],
        }),
        key_policies: vec![NativePackageKeyPolicy {
            signer_id: "product-signer".into(),
            not_before: now - Duration::hours(1),
            not_after: now + Duration::hours(2),
            revoked: false,
            allowed_package_ids: vec!["3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into()],
        }],
        trust_valid_until: now + Duration::hours(1),
        target: target(runtime_mode),
        target_triple: "x86_64-pc-windows-msvc".into(),
        sdk_api_version: "0.1.0".into(),
        build_set_id: "a".repeat(64),
        allowed_capabilities: vec![],
        max_receipt_age_seconds: 3600,
    }
}
