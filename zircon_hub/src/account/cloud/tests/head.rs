use super::*;
use crate::account::cloud::manifest::{FileEntry, MAX_BLOB_BYTES};
use serde_json::json;
use sha2::{Digest, Sha256};
use zircon_runtime_interface::project::{
    ProjectGuid, ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
    ProjectPackageLockState, ProjectPackageLockTarget, PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
};

const ORGANIZATION: &str = "00000000-0000-4000-8000-000000000001";
const PROJECT: &str = "00000000-0000-4000-8000-000000000002";

fn head() -> Value {
    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: ProjectGuid::new(),
            manifest_digest: ProjectManifestDigest::from_bytes(b"head fixture"),
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
        entries: Vec::new(),
    };
    let package_lock_digest = lock.digest().unwrap();
    let manifest = Manifest {
        schema_version: 1,
        engine: "Zircon".into(),
        package_lock_digest,
        package_lock: Some(ProjectPackageLockState::present(lock).unwrap()),
        ignore_policy: "zircon-project-v1".into(),
        source_revision: None,
        files: vec![FileEntry {
            path: "Content/scene.zui".into(),
            digest: "b".repeat(64),
            bytes: 5,
        }],
    };
    let manifest_digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&manifest).unwrap())
    );
    json!({
        "organizationId": ORGANIZATION, "projectId": PROJECT,
        "baseRevision": "0", "revision": "1",
        "manifestDigest": manifest_digest, "createdAt": 1,
        "createdBy": "c".repeat(64), "manifest": manifest
    })
}

#[test]
fn empty_and_valid_heads_are_admitted_without_using_remote_paths() {
    assert_eq!(
        admit(Value::Null, ORGANIZATION, PROJECT).unwrap(),
        Value::Null
    );
    let value = head();
    assert_eq!(admit(value.clone(), ORGANIZATION, PROJECT).unwrap(), value);
}

#[test]
fn rejects_cross_project_revision_digest_and_unsafe_paths() {
    let mut cases = Vec::new();
    for (key, value) in [
        ("organizationId", json!(PROJECT)),
        ("projectId", json!(ORGANIZATION)),
        ("revision", json!("0")),
        ("manifestDigest", json!("0".repeat(64))),
    ] {
        let mut candidate = head();
        candidate[key] = value;
        cases.push(candidate);
    }
    let mut path = head();
    path["manifest"]["files"][0]["path"] = json!("../secret");
    cases.push(path);
    let mut size = head();
    size["manifest"]["files"][0]["bytes"] = json!(MAX_BLOB_BYTES + 1);
    cases.push(size);
    for candidate in cases {
        assert!(admit(candidate, ORGANIZATION, PROJECT).is_err());
    }
}

#[test]
fn head_requires_server_canonical_file_order_even_with_a_matching_digest() {
    let mut candidate = head();
    candidate["manifest"]["files"] = json!([
        { "path": "Content/a.zui", "digest": "a".repeat(64), "bytes": 3 },
        { "path": "Content/z.zui", "digest": "b".repeat(64), "bytes": 5 }
    ]);
    let manifest: Manifest = serde_json::from_value(candidate["manifest"].clone()).unwrap();
    candidate["manifestDigest"] = json!(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&manifest).unwrap())
    ));
    assert!(admit(candidate.clone(), ORGANIZATION, PROJECT).is_ok());

    candidate["manifest"]["files"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let manifest: Manifest = serde_json::from_value(candidate["manifest"].clone()).unwrap();
    candidate["manifestDigest"] = json!(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&manifest).unwrap())
    ));
    assert!(matches!(
        admit(candidate, ORGANIZATION, PROJECT),
        Err(AccountError::ServiceFailure)
    ));
}
