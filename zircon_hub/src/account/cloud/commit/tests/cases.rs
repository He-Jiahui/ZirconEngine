use super::*;
use crate::account::cloud::manifest::{FileEntry, Manifest};
use serde_json::json;
use zircon_runtime_interface::project::{
    ProjectGuid, ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
    ProjectPackageLockState, ProjectPackageLockTarget, PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
};

const ORGANIZATION: &str = "00000000-0000-4000-8000-000000000001";
const PROJECT: &str = "00000000-0000-4000-8000-000000000002";

fn manifest() -> Manifest {
    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: ProjectGuid::new(),
            manifest_digest: ProjectManifestDigest::from_bytes(b"commit fixture"),
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
    Manifest {
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
    }
}

fn committed(digest: &str) -> Value {
    json!({
        "status": "committed",
        "snapshot": {
            "organizationId": ORGANIZATION,
            "projectId": PROJECT,
            "baseRevision": "0",
            "revision": "1",
            "manifestDigest": digest
        }
    })
}

fn conflict(digest: &str) -> Value {
    json!({
        "status": "conflict",
        "organizationId": ORGANIZATION,
        "projectId": PROJECT,
        "baseRevision": "0",
        "currentRevision": "1",
        "manifestDigest": digest
    })
}

#[test]
fn http_status_and_tenant_bound_receipt_distinguish_publish_from_conflict() {
    let digest = manifest().canonical_digest().unwrap();
    let published = decode_http(
        reqwest::StatusCode::OK,
        &serde_json::to_vec(&committed(&digest)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        project_receipt(&published, ORGANIZATION, PROJECT, "0", &digest).unwrap()["status"],
        "committed"
    );
    assert!(!is_conflict(&published));

    let rejected = decode_http(
        reqwest::StatusCode::CONFLICT,
        &serde_json::to_vec(&conflict(&digest)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        project_receipt(&rejected, ORGANIZATION, PROJECT, "0", &digest).unwrap()["status"],
        "conflict"
    );
    assert!(is_conflict(&rejected));
}

#[test]
fn malformed_status_pair_or_receipt_binding_remains_unknown() {
    let digest = manifest().canonical_digest().unwrap();
    for (status, body) in [
        (reqwest::StatusCode::OK, conflict(&digest)),
        (reqwest::StatusCode::CONFLICT, committed(&digest)),
        (
            reqwest::StatusCode::CONFLICT,
            json!({"error":"policy_conflict"}),
        ),
    ] {
        assert!(decode_http(status, &serde_json::to_vec(&body).unwrap()).is_err());
    }
    let mut cases = Vec::new();
    let mut other_organization = conflict(&digest);
    other_organization["organizationId"] = json!(PROJECT);
    cases.push(other_organization);
    let mut other_project = conflict(&digest);
    other_project["projectId"] = json!(ORGANIZATION);
    cases.push(other_project);
    let mut other_base = conflict(&digest);
    other_base["baseRevision"] = json!("2");
    cases.push(other_base);
    let mut other_digest = conflict(&digest);
    other_digest["manifestDigest"] = json!("c".repeat(64));
    cases.push(other_digest);
    let mut same_revision = conflict(&digest);
    same_revision["currentRevision"] = json!("0");
    cases.push(same_revision);
    let mut extra_field = conflict(&digest);
    extra_field["token"] = json!("secret");
    cases.push(extra_field);
    for value in cases {
        assert!(project_receipt(&value, ORGANIZATION, PROJECT, "0", &digest).is_none());
    }
}

#[test]
fn exact_operation_id_conflict_is_a_definite_rejection_not_a_cas_conflict() {
    let rejection = serde_json::to_vec(&json!({"error":"operation_id_conflict"})).unwrap();
    assert!(matches!(
        decode_http(reqwest::StatusCode::CONFLICT, &rejection),
        Err(AccountError::OperationConflict)
    ));
    assert!(matches!(
        decode_http(reqwest::StatusCode::OK, &rejection),
        Err(AccountError::OutcomeUnknown)
    ));
    for body in [
        json!({"error":"policy_conflict"}),
        json!({"error":"operation_id_conflict","result":"forged"}),
    ] {
        let bytes = serde_json::to_vec(&body).unwrap();
        assert!(matches!(
            decode_http(reqwest::StatusCode::CONFLICT, &bytes),
            Err(AccountError::OutcomeUnknown)
        ));
    }
}
