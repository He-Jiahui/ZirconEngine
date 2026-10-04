use super::*;

fn fixture() -> ProjectPackageLock {
    ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: ProjectGuid::from_str("00000000-0000-4000-8000-000000000001").unwrap(),
            manifest_digest: ProjectManifestDigest::from_bytes(b"manifest"),
        },
        target: ProjectPackageLockTarget {
            runtime_mode: ProjectPackageLockRuntimeMode::ClientRuntime,
            platform: ProjectPackageLockPlatform::Windows,
        },
        authority: ProjectPackageLockAuthority {
            index_sha256: "a".repeat(64),
            policy_sha256: "b".repeat(64),
            build_set_id: "c".repeat(64),
            capability_digest: "d".repeat(64),
            provider_revision_digest: "e".repeat(64),
        },
        entries: Vec::new(),
    }
}

#[test]
fn empty_lock_is_canonical_and_not_unavailable_sentinel() {
    let lock = fixture();
    let digest = lock.digest().unwrap();
    assert!(lock.is_empty_selection());
    assert_ne!(digest, format!("{:x}", Sha256::digest([])));
    assert_eq!(
        ProjectPackageLock::decode_canonical(&lock.canonical_bytes().unwrap()).unwrap(),
        lock
    );
}

#[test]
fn canonical_wire_sorts_capabilities_and_entries() {
    let mut lock = fixture();
    lock.entries = vec![
        ProjectPackageLockEntry {
            plugin_id: "studio.z".into(),
            package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
            version: "1.0.0".into(),
            release_revision: "2".into(),
            artifact_digest: "b".repeat(64),
            capabilities: vec!["render".into(), "audio".into()],
        },
        ProjectPackageLockEntry {
            plugin_id: "studio.a".into(),
            package_id: "4298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
            version: "1.0.0".into(),
            release_revision: "1".into(),
            artifact_digest: "a".repeat(64),
            capabilities: Vec::new(),
        },
    ];
    let bytes = lock.canonical_bytes().unwrap();
    assert!(ProjectPackageLock::decode_canonical(&bytes).is_ok());
    assert!(lock.require_canonical_wire().is_err());
}

#[test]
fn shared_lock_digest_excludes_local_checkout_identity_but_keeps_target() {
    let first = fixture();
    let mut second = first.clone();
    second.project.project_guid =
        ProjectGuid::from_str("00000000-0000-4000-8000-000000000002").unwrap();
    second.project.manifest_digest = ProjectManifestDigest::from_bytes(b"other checkout");
    assert_eq!(first.digest().unwrap(), second.digest().unwrap());
    assert_ne!(
        first.canonical_bytes().unwrap(),
        second.canonical_bytes().unwrap()
    );
    second.target.runtime_mode = ProjectPackageLockRuntimeMode::EditorHost;
    assert_ne!(first.digest().unwrap(), second.digest().unwrap());
}

#[test]
fn authority_generation_is_in_the_shared_digest_but_checkout_identity_is_not() {
    let first = fixture();
    let mut rotated = first.clone();
    rotated.authority.build_set_id = "e".repeat(64);
    assert_ne!(first.digest().unwrap(), rotated.digest().unwrap());
    assert!(valid_package_version("0.0.0"));
    assert!(valid_package_version("4294967295.1.0"));
    assert!(!valid_package_version("01.2.3"));
    assert!(!valid_package_version("1.2"));
    assert!(!valid_package_version("1.2.3-alpha"));
}

#[test]
fn typed_lock_state_rejects_unavailable_for_present_consumers() {
    let state = ProjectPackageLockState::present(fixture()).unwrap();
    assert!(state.validate().is_ok());
    assert!(state.is_present());
    let unavailable = ProjectPackageLockState::Unavailable {
        schema_version: PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1,
        reason: ProjectPackageLockUnavailableReason::ProviderMissing,
    };
    assert!(unavailable.validate().is_ok());
    assert!(!unavailable.is_present());
}

#[test]
fn invalid_identity_and_duplicate_plugins_fail_closed() {
    let mut lock = fixture();
    lock.entries = vec![
        ProjectPackageLockEntry {
            plugin_id: "studio.a".into(),
            package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
            version: "1.0.0".into(),
            release_revision: "1".into(),
            artifact_digest: "a".repeat(64),
            capabilities: Vec::new(),
        },
        ProjectPackageLockEntry {
            plugin_id: "studio.a".into(),
            package_id: "4298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
            version: "1.0.0".into(),
            release_revision: "2".into(),
            artifact_digest: "b".repeat(64),
            capabilities: Vec::new(),
        },
    ];
    assert_eq!(
        lock.validate(),
        Err(ProjectPackageLockError::DuplicatePlugin)
    );
}
