use super::*;

#[test]
fn producer_rejects_valid_lock_rows_that_exceed_the_helper_wire_budget() {
    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: "00000000-0000-4000-8000-000000000001".parse().unwrap(),
            manifest_digest: zircon_runtime_interface::project::ProjectManifestDigest::from_bytes(
                b"manifest",
            ),
        },
        target: ProjectPackageLockTarget {
            runtime_mode: ProjectPackageLockRuntimeMode::EditorHost,
            platform: ProjectPackageLockPlatform::Windows,
        },
        authority: ProjectPackageLockAuthority {
            index_sha256: "a".repeat(64),
            policy_sha256: "b".repeat(64),
            build_set_id: "c".repeat(64),
            capability_digest: "d".repeat(64),
            provider_revision_digest: "e".repeat(64),
        },
        entries: (0..512)
            .map(|index| ProjectPackageLockEntry {
                plugin_id: format!("studio.p{index}"),
                package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
                version: "4294967295.4294967295.4294967295".into(),
                release_revision: "1".into(),
                artifact_digest: "a".repeat(64),
                capabilities: vec!["r".repeat(128)],
            })
            .collect(),
    };
    assert!(lock.validate().is_ok());
    assert_eq!(
        require_bounded_lock(&lock),
        Err(ProjectPackageLockProviderError::InvalidLock)
    );
}

#[test]
fn server_targets_are_rejected_before_policy_or_store_access() {
    let target = NativePluginArtifactTarget::new(
        RuntimeTargetMode::ServerRuntime,
        crate::core::framework::project::ExportTargetPlatform::Windows,
    );
    assert_eq!(
        contract_target(&target),
        Err(ProjectPackageLockProviderError::UnsupportedTarget)
    );
}

#[test]
fn target_contract_is_explicit_for_client_and_editor() {
    let client = NativePluginArtifactTarget::new(
        RuntimeTargetMode::ClientRuntime,
        crate::core::framework::project::ExportTargetPlatform::Windows,
    );
    assert_eq!(
        contract_target(&client).unwrap().runtime_mode,
        ProjectPackageLockRuntimeMode::ClientRuntime
    );
    let editor = NativePluginArtifactTarget::new(
        RuntimeTargetMode::EditorHost,
        crate::core::framework::project::ExportTargetPlatform::Windows,
    );
    assert_eq!(
        contract_target(&editor).unwrap().runtime_mode,
        ProjectPackageLockRuntimeMode::EditorHost
    );
}
