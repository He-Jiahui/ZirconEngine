use super::*;

const OP: &str = "00000000-0000-4000-8000-000000000001";
const ORG: &str = "00000000-0000-4000-8000-000000000002";
const PKG: &str = "00000000-0000-4000-8000-000000000003";

fn request() -> InstallRequest {
    InstallRequest {
        operation_id: OP.into(),
        identity_digest: "a".repeat(64),
        package_id: PKG.into(),
        version: "1.0.0".into(),
        release_revision: "1".into(),
        artifact_digest: "b".repeat(64),
        artifact_size: 3,
        expected_inventory_revision: "0".into(),
        schema_version: PACKAGE_INSTALL_SCHEMA_V2,
        target: Some(
            PackageInstallTarget::for_runtime_mode(PackageRuntimeMode::EditorHost).unwrap(),
        ),
    }
}

#[tokio::test]
async fn targetless_legacy_retry_and_reconcile_do_not_load_or_query_the_package_helper() {
    use super::super::operations::{OperationIdentity, OperationJournal, OperationStatus};
    use std::sync::Arc;

    let directory = std::env::temp_dir().join(format!(
        "hub-package-target-migration-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let journal_path = directory.join("operations.dat");
    let mut broker = super::super::tests::broker("http://127.0.0.1:1".into());
    Arc::get_mut(&mut broker)
        .unwrap()
        .config
        .operation_journal_path = Some(journal_path.clone());
    super::super::tests::set_account(&broker, "alice", 7).await;

    let identity = OperationIdentity::new(&broker.config, "alice");
    let mut legacy_request = request();
    legacy_request.identity_digest = identity_digest(&identity, ORG).unwrap();
    legacy_request.schema_version = LEGACY_PACKAGE_INSTALL_SCHEMA_V1;
    legacy_request.target = None;
    let payload = OperationPayload::InstallPackage {
        organization: ORG.into(),
        owner: "d".repeat(64),
        request: legacy_request,
    };
    let journal = OperationJournal::new(journal_path.clone());
    assert!(journal.admit(&identity, OP, "7", payload).unwrap());
    let retained_operation_status = || {
        let records = journal.list(&identity).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].operation_id, OP);
        records[0].status
    };
    assert_eq!(retained_operation_status(), OperationStatus::Unknown);

    reset_package_dispatch_counts();
    assert!(matches!(
        broker.retry_operation("7", OP).await,
        Err(AccountError::PackageTargetRequired)
    ));
    assert_eq!(package_dispatch_counts(), (0, 0));
    assert_eq!(retained_operation_status(), OperationStatus::Unknown);

    assert!(matches!(
        broker.reconcile_operation("7", OP).await,
        Err(AccountError::PackageTargetRequired)
    ));
    assert_eq!(package_dispatch_counts(), (0, 0));
    assert_eq!(retained_operation_status(), OperationStatus::Unknown);

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn helper_target_echo_is_required_even_when_recovered_receipt_is_absent() {
    let target = PackageInstallTarget {
        runtime_mode: PackageRuntimeMode::EditorHost,
        platform: PackageTargetPlatform::Windows,
    };
    let response = serde_json::json!({"status":"ready","target":target});
    assert!(validate_target_echo(&response, target).is_ok());

    let wrong_mode = serde_json::json!({
        "status":"ready",
        "target":{"runtime_mode":"client_runtime","platform":"windows"}
    });
    assert!(validate_target_echo(&wrong_mode, target).is_err());
    assert!(validate_target_echo(&serde_json::json!({"status":"ready"}), target).is_err());
}

fn receipt(request: &InstallRequest) -> Value {
    serde_json::json!({"schemaVersion":2,"operationId":OP,"requestDigest":request.fingerprint().unwrap(),"inventoryRevision":"1","pluginId":"fixture.plugin","target":request.target,"package":{"operationId":OP,"packageId":PKG,"version":"1.0.0","releaseRevision":"1","artifactDigest":"b".repeat(64),"slot":"slots/private","files":{"plugin.toml":"c".repeat(64)}}})
}

#[test]
fn package_receipt_requires_original_operation_payload_and_projects_no_native_paths() {
    let request = request();
    let raw = receipt(&request);
    let view = project_receipt(&request, &raw).unwrap();
    assert_eq!(view["schemaVersion"], 2);
    assert_eq!(view["targetMode"], "editor_host");
    assert_eq!(view["operationId"], OP);
    assert_eq!(view["inventoryRevision"], "1");
    for field in ["requestDigest", "identityDigest", "slot", "files"] {
        assert!(view.get(field).is_none());
        assert!(view["package"].get(field).is_none());
    }
    for field in ["requestDigest", "operationId", "inventoryRevision"] {
        let mut bad = raw.clone();
        bad[field] = serde_json::json!("invalid");
        assert!(project_receipt(&request, &bad).is_none(), "{field}");
    }
    let mut other = request.clone();
    other.expected_inventory_revision = "1".into();
    assert!(project_receipt(&other, &raw).is_none());
    let mut wrong_target = raw.clone();
    wrong_target["target"]["runtime_mode"] = serde_json::json!("client_runtime");
    assert!(project_receipt(&request, &wrong_target).is_none());
}

#[test]
fn package_inventory_projects_only_verified_installations_and_rejects_duplicate_ids() {
    let raw = receipt(&request());
    let mut inventory = serde_json::json!({"schemaVersion":1,"revision":"18446744073709551615","packages":[raw["package"].clone()]});
    let view = project_inventory(&inventory, PackageRuntimeMode::ClientRuntime).unwrap();
    assert_eq!(view["schemaVersion"], 2);
    assert_eq!(view["targetMode"], "client_runtime");
    assert!(view["packages"][0].get("slot").is_none());
    inventory["packages"]
        .as_array_mut()
        .unwrap()
        .push(raw["package"].clone());
    assert!(project_inventory(&inventory, PackageRuntimeMode::ClientRuntime).is_err());
    assert!(project_inventory(
        &serde_json::json!({"schemaVersion":1,"revision":"01","packages":[]}),
        PackageRuntimeMode::ClientRuntime,
    )
    .is_err());
}

#[test]
fn policy_index_resolves_only_the_explicit_runtime_target() {
    let editor = PackageInstallTarget {
        runtime_mode: PackageRuntimeMode::EditorHost,
        platform: PackageTargetPlatform::Windows,
    };
    let client = PackageInstallTarget {
        runtime_mode: PackageRuntimeMode::ClientRuntime,
        platform: PackageTargetPlatform::Windows,
    };
    let index: NativePluginPolicyIndex = serde_json::from_value(serde_json::json!({
        "schemaVersion": 1,
        "kind": POLICY_INDEX_KIND,
        "entries": [
            {"target":{"runtime_mode":"editor_host","platform":"windows"},"policy_path":"C:/host/editor-policy.json","policy_sha256":"a".repeat(64)},
            {"target":{"runtime_mode":"client_runtime","platform":"windows"},"policy_path":"C:/host/client-policy.json","policy_sha256":"b".repeat(64)}
        ]
    }))
    .unwrap();

    assert_eq!(
        selected_policy_index_entry(&index, editor)
            .unwrap()
            .policy_path,
        PathBuf::from("C:/host/editor-policy.json")
    );
    assert_eq!(
        selected_policy_index_entry(&index, client)
            .unwrap()
            .policy_path,
        PathBuf::from("C:/host/client-policy.json")
    );
    assert!(matches!(
        selected_policy_index_entry(
            &index,
            PackageInstallTarget {
                runtime_mode: PackageRuntimeMode::EditorHost,
                platform: PackageTargetPlatform::Linux,
            },
        ),
        Err(AccountError::PackageTargetUnconfigured)
    ));

    let duplicate: NativePluginPolicyIndex = serde_json::from_value(serde_json::json!({
        "schemaVersion": 1,
        "kind": POLICY_INDEX_KIND,
        "entries": [
            {"target":{"runtime_mode":"editor_host","platform":"windows"},"policy_path":"C:/host/same.json","policy_sha256":"a".repeat(64)},
            {"target":{"runtime_mode":"editor_host","platform":"windows"},"policy_path":"C:/host/other.json","policy_sha256":"b".repeat(64)}
        ]
    }))
    .unwrap();
    assert!(matches!(
        selected_policy_index_entry(&duplicate, editor),
        Err(AccountError::PackageTrust)
    ));
}

#[test]
fn selected_host_policy_root_requires_matching_bytes_and_target() {
    let target = PackageInstallTarget {
        runtime_mode: PackageRuntimeMode::ClientRuntime,
        platform: PackageTargetPlatform::Windows,
    };
    let expected_root = std::env::temp_dir().join("zircon-client-policy-root");
    let policy_bytes = serde_json::to_vec(&serde_json::json!({
        "target": {"runtime_mode":"client_runtime","platform":"windows"},
        "root": expected_root,
    }))
    .unwrap();
    let entry = NativePluginPolicyIndexEntry {
        target,
        policy_path: std::env::temp_dir().join("client-policy.json"),
        policy_sha256: digest(&policy_bytes),
    };
    assert_eq!(
        selected_policy_root(&entry, target, &policy_bytes).unwrap(),
        expected_root
    );
    assert!(matches!(
        selected_policy_root(&entry, target, b"tampered"),
        Err(AccountError::PackageTrust)
    ));

    let wrong_target = serde_json::to_vec(&serde_json::json!({
        "target": {"runtime_mode":"editor_host","platform":"windows"},
        "root": std::env::temp_dir().join("editor-root"),
    }))
    .unwrap();
    let wrong_entry = NativePluginPolicyIndexEntry {
        policy_sha256: digest(&wrong_target),
        ..entry
    };
    assert!(matches!(
        selected_policy_root(&wrong_entry, target, &wrong_target),
        Err(AccountError::PackageTrust)
    ));
}

#[test]
fn package_client_configuration_requires_v2_and_does_not_select_a_second_policy_index() {
    let configuration = serde_json::json!({
        "schemaVersion": 2,
        "executable": std::env::temp_dir().join("zircon-package-service.exe"),
        "executableSha256": "a".repeat(64),
    });
    assert!(serde_json::from_value::<ClientConfig>(configuration.clone()).is_ok());
    let mut legacy = configuration.clone();
    legacy["schemaVersion"] = serde_json::json!(1);
    assert!(serde_json::from_value::<ClientConfig>(legacy).is_err());
    let mut divergent_policy = configuration;
    divergent_policy["policyIndexPath"] =
        serde_json::json!(std::env::temp_dir().join("other-policy-index.json"));
    assert!(serde_json::from_value::<ClientConfig>(divergent_policy).is_err());
}

#[test]
fn package_target_modes_and_action_schema_are_strict_and_legacy_journals_do_not_gain_a_target() {
    assert_eq!(
        serde_json::from_str::<PackageRuntimeMode>("\"editor_host\"").unwrap(),
        PackageRuntimeMode::EditorHost
    );
    assert_eq!(
        serde_json::from_str::<PackageRuntimeMode>("\"client_runtime\"").unwrap(),
        PackageRuntimeMode::ClientRuntime
    );
    assert!(serde_json::from_str::<PackageRuntimeMode>("\"server_runtime\"").is_err());
    assert!(serde_json::from_value::<PackageActionSchemaV2>(serde_json::json!(2)).is_ok());
    for version in [1, 3, 255] {
        assert!(
            serde_json::from_value::<PackageActionSchemaV2>(serde_json::json!(version)).is_err()
        );
    }

    let legacy = serde_json::json!({
        "operationId": OP,
        "identityDigest": "a".repeat(64),
        "packageId": PKG,
        "version": "1.0.0",
        "releaseRevision": "1",
        "artifactDigest": "b".repeat(64),
        "artifactSize": 3,
        "expectedInventoryRevision": "0"
    });
    let parsed: InstallRequest = serde_json::from_value(legacy.clone()).unwrap();
    assert_eq!(parsed.schema_version, LEGACY_PACKAGE_INSTALL_SCHEMA_V1);
    assert!(parsed.target.is_none());
    assert!(matches!(
        parsed.validate_for_new_install(),
        Err(AccountError::PackageTargetRequired)
    ));
    assert_eq!(serde_json::to_value(parsed).unwrap(), legacy);
}

#[test]
fn install_payload_cannot_be_dispatched_as_http_or_change_the_operation_id() {
    let mut payload = OperationPayload::InstallPackage {
        organization: ORG.into(),
        owner: "d".repeat(64),
        request: request(),
    };
    payload.validate(OP).unwrap();
    assert!(payload.request(OP).is_err());
    assert!(payload.validate(PKG).is_err());
    if let OperationPayload::InstallPackage { request, .. } = &mut payload {
        request.artifact_size = MAX_PACKAGE_BYTES as u64 + 1;
    }
    assert!(matches!(
        payload.validate(OP),
        Err(AccountError::PackageCapacity)
    ));
}

#[test]
fn package_owner_partition_includes_issuer_subject_client_service_and_organization() {
    let config = crate::account::AccountConfig {
        issuer: "https://identity.example/realm".into(),
        client_id: "hub".into(),
        service_url: "https://service.example".into(),
        callback_port: 8480,
        allow_loopback_http: false,
        operation_journal_path: None,
    };
    let identity = OperationIdentity::new(&config, "alice");
    let original = identity_digest(&identity, ORG).unwrap();
    for (issuer, client, service, subject, org) in [
        (
            "https://other.example/realm",
            "hub",
            "https://service.example",
            "alice",
            ORG,
        ),
        (
            "https://identity.example/realm",
            "other",
            "https://service.example",
            "alice",
            ORG,
        ),
        (
            "https://identity.example/realm",
            "hub",
            "https://other.example",
            "alice",
            ORG,
        ),
        (
            "https://identity.example/realm",
            "hub",
            "https://service.example",
            "bob",
            ORG,
        ),
        (
            "https://identity.example/realm",
            "hub",
            "https://service.example",
            "alice",
            PKG,
        ),
    ] {
        let config = crate::account::AccountConfig {
            issuer: issuer.into(),
            client_id: client.into(),
            service_url: service.into(),
            ..config.clone()
        };
        assert_ne!(
            original,
            identity_digest(&OperationIdentity::new(&config, subject), org).unwrap()
        );
    }
}

#[test]
fn catalog_license_request_and_receipt_bind_requested_release() {
    let payload = OperationPayload::CatalogLicense {
        organization: ORG.into(),
        expected_policy_revision: "5".into(),
        package_id: PKG.into(),
        revision: "2".into(),
        license_id: "license-v2".into(),
    };
    let (path, body) = payload.request(OP).unwrap();
    assert_eq!(path, format!("/v1/organizations/{ORG}/licenses"));
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["operationId"], OP);
    assert_eq!(body["expectedPolicyRevision"], "5");
    assert!(super::super::service::project_mutation_result(
        &payload,
        &serde_json::json!({"packageId":PKG,"revision":"2","token":"not projected"})
    )
    .unwrap()
    .get("token")
    .is_none());
    assert!(super::super::service::project_mutation_result(
        &payload,
        &serde_json::json!({"packageId":PKG,"revision":"3"})
    )
    .is_none());
}
