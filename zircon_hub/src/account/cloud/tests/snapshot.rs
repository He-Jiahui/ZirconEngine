use std::{fs, path::PathBuf, time::SystemTime};

use zircon_runtime_interface::project::{
    ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
    ProjectPackageLockTarget, PROJECT_MANIFEST_FORMAT_VERSION,
    PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
};

use super::*;

struct EmptyProvider;

impl ProjectPackageLockProvider for EmptyProvider {
    fn capture(
        &self,
        _project_root: &std::path::Path,
        expected: &ProjectPackageLockContext,
    ) -> Result<ProjectPackageLock, PackageLockProviderError> {
        Ok(ProjectPackageLock {
            schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
            project: ProjectPackageLockProject {
                project_guid: expected.project_guid,
                manifest_digest: expected.manifest_digest,
            },
            target: expected.target,
            authority: ProjectPackageLockAuthority {
                index_sha256: "a".repeat(64),
                policy_sha256: "b".repeat(64),
                build_set_id: "c".repeat(64),
                capability_digest: "d".repeat(64),
                provider_revision_digest: "e".repeat(64),
            },
            entries: Vec::new(),
        })
    }
}

fn lock_context(root: &Path, guid: ProjectGuid) -> ProjectPackageLockContext {
    ProjectPackageLockContext {
        project_guid: guid,
        manifest_digest: ProjectManifestDigest::from_bytes(
            fs::read(root.join("zircon-project.toml")).unwrap(),
        ),
        target: ProjectPackageLockTarget {
            runtime_mode: ProjectPackageLockRuntimeMode::ClientRuntime,
            platform: ProjectPackageLockPlatform::Windows,
        },
    }
}

fn fixture() -> (PathBuf, ProjectGuid) {
    let target = PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").expect("managed target"));
    let root = target.join(format!(
        "cloud-snapshot-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let guid = ProjectGuid::new();
    fs::create_dir_all(root.join("Content")).unwrap();
    fs::write(root.join("Content/scene.zui"), b"scene").unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::write(root.join("target/generated.bin"), b"generated").unwrap();
    fs::create_dir_all(root.join(".zircon")).unwrap();
    fs::write(root.join(".zircon/local-state"), b"private").unwrap();
    fs::write(root.join(".env"), b"secret").unwrap();
    fs::write(
        root.join("zircon-project.toml"),
        format!(
            "name = 'Local'\ndefault_scene = 'res://scenes/main.scene.toml'\nformat_version = {}\nlibrary_version = 1\nproject_guid = '{guid}'\nasset_roots = ['res']\n",
            PROJECT_MANIFEST_FORMAT_VERSION
        ),
    )
    .unwrap();
    (root, guid)
}

#[test]
fn snapshot_is_canonical_bounded_and_omits_private_generated_and_secret_paths() {
    let (root, guid) = fixture();
    let context = lock_context(&root, guid);
    let snapshot = capture_with_package_lock(&root, guid, &context, &EmptyProvider).unwrap();
    let paths = snapshot
        .manifest
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(paths, ["Content/scene.zui"]);
    assert_eq!(snapshot.manifest.ignore_policy, "zircon-project-v1");
    assert_ne!(snapshot.manifest.package_lock_digest, sha256(&[]));
    assert_eq!(
        read_blob(&root, &snapshot.manifest.files[0]).unwrap(),
        b"scene"
    );
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn changed_project_identity_is_rejected_before_helper_capture() {
    let (root, guid) = fixture();
    let other_guid = ProjectGuid::new();
    assert_ne!(guid, other_guid);
    assert_eq!(
        capture(&root, other_guid).await,
        Err("hub_cloud_sync_project_changed")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ignored_and_empty_directories_consume_the_bounded_scan_budget() {
    let (root, _) = fixture();
    let scan_root = root.join("scan-budget");
    fs::create_dir_all(scan_root.join("node_modules")).unwrap();
    fs::create_dir_all(scan_root.join("empty-a")).unwrap();
    fs::create_dir_all(scan_root.join("empty-b")).unwrap();

    let mut entries = Vec::new();
    let mut total_bytes = 0;
    let mut entry_limited = ScanBudget::new(2, 10);
    assert_eq!(
        scan_directory(
            &scan_root,
            &scan_root,
            &mut entries,
            &mut total_bytes,
            &mut entry_limited,
        ),
        Err("hub_cloud_sync_snapshot_invalid")
    );

    let mut directory_limited = ScanBudget::new(10, 2);
    assert_eq!(
        scan_directory(
            &scan_root,
            &scan_root,
            &mut entries,
            &mut total_bytes,
            &mut directory_limited,
        ),
        Err("hub_cloud_sync_snapshot_invalid")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_project_bytes_fail_before_the_captured_manifest_can_be_uploaded() {
    let (root, guid) = fixture();
    let context = lock_context(&root, guid);
    let snapshot = capture_with_package_lock(&root, guid, &context, &EmptyProvider).unwrap();
    fs::write(root.join("Content/scene.zui"), b"changed").unwrap();
    assert_eq!(
        read_blob(&root, &snapshot.manifest.files[0]),
        Err("hub_cloud_sync_project_changed")
    );
    fs::remove_dir_all(root).unwrap();
}
