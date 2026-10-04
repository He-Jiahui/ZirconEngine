use super::*;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use zircon_runtime_interface::project::{
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
};

struct Fixture(ProjectPackageLock);

impl ProjectPackageLockProvider for Fixture {
    fn capture(
        &self,
        _project_root: &Path,
        _expected: &ProjectPackageLockContext,
    ) -> Result<ProjectPackageLock, PackageLockProviderError> {
        Ok(self.0.clone())
    }
}

fn context() -> ProjectPackageLockContext {
    ProjectPackageLockContext {
        project_guid: ProjectGuid::from_str("00000000-0000-4000-8000-000000000001").unwrap(),
        manifest_digest: ProjectManifestDigest::from_bytes(b"manifest"),
        target: ProjectPackageLockTarget {
            runtime_mode: ProjectPackageLockRuntimeMode::ClientRuntime,
            platform: ProjectPackageLockPlatform::Windows,
        },
    }
}

fn fixture(context: &ProjectPackageLockContext) -> ProjectPackageLock {
    ProjectPackageLock {
        schema_version: zircon_runtime_interface::project::PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: context.project_guid,
            manifest_digest: context.manifest_digest,
        },
        target: context.target,
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
fn authoritative_empty_lock_has_a_real_digest() {
    let context = context();
    let provider = Fixture(fixture(&context));
    let digest = package_lock_digest(
        &provider,
        Path::new("project-root-is-routing-only"),
        &context,
    )
    .unwrap();
    assert_ne!(digest, format!("{:x}", Sha256::digest([])));
}

#[test]
fn target_and_revision_mismatch_are_rejected_before_cloud_manifest_write() {
    let context = context();
    let mut wrong_target = fixture(&context);
    wrong_target.target.runtime_mode = ProjectPackageLockRuntimeMode::EditorHost;
    assert_eq!(
        package_lock_digest(
            &Fixture(wrong_target),
            Path::new("project-root-is-routing-only"),
            &context,
        ),
        Err(PackageLockProviderError::TargetChanged)
    );

    let mut wrong_revision = fixture(&context);
    wrong_revision.project.manifest_digest = ProjectManifestDigest::from_bytes(b"other generation");
    assert_eq!(
        package_lock_digest(
            &Fixture(wrong_revision),
            Path::new("project-root-is-routing-only"),
            &context,
        ),
        Err(PackageLockProviderError::ProjectChanged)
    );
}

#[test]
fn noncanonical_runtime_wire_is_rejected() {
    let context = context();
    let mut lock = fixture(&context);
    lock.entries = vec![zircon_runtime_interface::project::ProjectPackageLockEntry {
        plugin_id: "studio.a".into(),
        package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
        version: "1.0.0".into(),
        release_revision: "1".into(),
        artifact_digest: "a".repeat(64),
        capabilities: vec!["z".into(), "a".into()],
    }];
    assert_eq!(
        package_lock_digest(
            &Fixture(lock),
            Path::new("project-root-is-routing-only"),
            &context,
        ),
        Err(PackageLockProviderError::Invalid)
    );
}
