use super::*;
use zircon_runtime_interface::project::PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1;

#[test]
fn helper_lock_response_cannot_substitute_project_generation_or_target() {
    let project = ProjectPackageLockProject {
        project_guid: ProjectGuid::new(),
        manifest_digest: ProjectManifestDigest::from_bytes(b"current project"),
    };
    let target = PackageInstallTarget {
        runtime_mode: PackageRuntimeMode::EditorHost,
        platform: PackageTargetPlatform::Windows,
    };
    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: project.clone(),
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
    assert!(validate_lock_response(lock.clone(), project.clone(), target).is_ok());
    let mut stale = lock.clone();
    stale.project.manifest_digest = ProjectManifestDigest::from_bytes(b"stale project");
    assert!(validate_lock_response(stale, project.clone(), target).is_err());
    let mut foreign = lock.clone();
    foreign.project.project_guid = ProjectGuid::new();
    assert!(validate_lock_response(foreign, project.clone(), target).is_err());
    let mut wrong_target = lock;
    wrong_target.target.runtime_mode = ProjectPackageLockRuntimeMode::ClientRuntime;
    assert!(validate_lock_response(wrong_target, project, target).is_err());
}
