use std::{fs, path::PathBuf, time::SystemTime};

use zircon_runtime_interface::project::{
    ProjectGuid, ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
    ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
    ProjectPackageLockState, ProjectPackageLockTarget, PROJECT_MANIFEST_FORMAT_VERSION,
    PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
};

use super::*;

const ORGANIZATION: &str = "00000000-0000-4000-8000-000000000001";
const PROJECT: &str = "00000000-0000-4000-8000-000000000002";
pub(super) const STAGE: &str = "00000000-0000-4000-8000-000000000003";
const NEXT_STAGE: &str = "00000000-0000-4000-8000-000000000004";
const OTHER_STAGE: &str = "00000000-0000-4000-8000-000000000005";
const STALE_STAGE: &str = "00000000-0000-4000-8000-000000000004";
const CONFLICT_STAGE: &str = "00000000-0000-4000-8000-000000000005";
const OTHER_SCOPE_STAGE: &str = "00000000-0000-4000-8000-000000000006";
const CURRENT_STAGE: &str = "00000000-0000-4000-8000-000000000007";

pub(super) fn fixture() -> (PathBuf, ProjectGuid) {
    let target = PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").expect("managed target"));
    let root = target.join(format!(
        "cloud-stage-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let guid = ProjectGuid::new();
    fs::create_dir_all(&root).unwrap();
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

#[cfg(unix)]
#[test]
fn existing_permissive_cloud_directories_are_tightened_before_artifact_reads() {
    use std::os::unix::fs::PermissionsExt;

    let (root, _) = fixture();
    let components = [".zircon", "cloud", "uploads", STAGE];
    let operation = components
        .iter()
        .fold(root.clone(), |path, component| path.join(component));
    fs::create_dir_all(&operation).unwrap();
    let marker = operation.join(UPLOAD_MARKER);
    fs::write(&marker, b"legacy artifact").unwrap();
    for directory in [
        root.join(".zircon"),
        root.join(".zircon/cloud"),
        root.join(".zircon/cloud/uploads"),
        operation.clone(),
    ] {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::set_permissions(&marker, fs::Permissions::from_mode(0o644)).unwrap();

    let private = existing_private_directory(&root, &components)
        .unwrap()
        .unwrap();

    for directory in [
        root.join(".zircon"),
        root.join(".zircon/cloud"),
        root.join(".zircon/cloud/uploads"),
        operation,
    ] {
        assert_eq!(
            fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    let marker_path = private.join(UPLOAD_MARKER);
    assert_eq!(
        read_private_bounded(&root, &marker_path, 1024).unwrap(),
        b"legacy artifact"
    );
    assert_eq!(
        fs::metadata(marker_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn publication_recovery_removes_only_receipted_project_temps_and_private_orphans() {
    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let parent = root.join("Content");
    fs::create_dir_all(&parent).unwrap();
    let expected_temporary =
        next_publish_temporary_path(&parent, STAGE, &sha256(b"remote")).unwrap();
    let temporary_name = expected_temporary
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let artifact = PublicationArtifact {
        schema_version: PUBLICATION_SCHEMA_VERSION,
        stage_id: STAGE.into(),
        relative_path: "Content/remote.txt".into(),
        temporary_name,
        digest: sha256(b"remote"),
        bytes: 6,
    };
    let record = persist_publication_artifact(&root, &artifact).unwrap();
    fs::write(&expected_temporary, b"rem").unwrap();
    let user_file = parent.join(format!(
        "{PUBLISH_TEMP_PREFIX}{}-{}-1-2-3.tmp",
        sha256(b"unrelated-stage"),
        sha256(b"user-content")
    ));
    fs::write(&user_file, b"keep").unwrap();
    let private_orphan = staged.directory.join(format!(
        "{PUBLISH_TEMP_PREFIX}{}-{}-1-2-3.tmp",
        sha256(STAGE.as_bytes()),
        sha256(b"private-content")
    ));
    fs::write(&private_orphan, b"partial").unwrap();

    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(
        recover_orphaned_publish_temporaries(&root, &project_lease),
        Ok(2)
    );

    assert_eq!(
        fs::read(stage_file_path(&staged.directory, "Content/remote.txt").unwrap()).unwrap(),
        b"remote"
    );
    assert!(!expected_temporary.exists());
    assert!(!private_orphan.exists());
    assert!(!record.exists());
    assert_eq!(fs::read(&user_file).unwrap(), b"keep");
    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_publication_receipt_cleanup_is_reported_and_restart_recovery_can_finish() {
    use std::io::Write;

    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let target_parent = root.join("Content");
    fs::create_dir_all(&target_parent).unwrap();
    let temporary_path =
        next_publish_temporary_path(&target_parent, STAGE, &sha256(b"remote")).unwrap();
    let temporary_name = temporary_path.file_name().unwrap().to_os_string();
    let artifact = PublicationArtifact {
        schema_version: PUBLICATION_SCHEMA_VERSION,
        stage_id: STAGE.into(),
        relative_path: "Content/remote.txt".into(),
        temporary_name: temporary_name.to_string_lossy().into_owned(),
        digest: sha256(b"remote"),
        bytes: 6,
    };
    let record = persist_publication_artifact(&root, &artifact).unwrap();
    let parent = AnchoredDirectory::open_for_file_writes(&target_parent).unwrap();
    let mut source = parent.create_new_project_file(&temporary_name).unwrap();
    source.write_all(b"remote").unwrap();
    source.sync_all().unwrap();
    drop(source);
    parent
        .hard_link(&temporary_name, std::ffi::OsStr::new("remote.txt"))
        .unwrap();
    let mut publication = PublishTemporary {
        parent: parent.try_clone().unwrap(),
        name: temporary_name,
        file: None,
        receipt: Some(record.clone()),
    };

    let cleanup = publication.finish_after_publication_with(|_| {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "injected receipt cleanup failure",
        ))
    });
    assert_eq!(
        cleanup.unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    assert_eq!(
        fs::read(target_parent.join("remote.txt")).unwrap(),
        b"remote"
    );
    assert!(!temporary_path.exists());
    assert!(record.exists());

    let publication_path = operation_directory(&root, PUBLICATION_ROOT, STAGE).unwrap();
    let project_anchor = AnchoredDirectory::open(&root).unwrap();
    let publication_directory = AnchoredDirectory::open(&publication_path).unwrap();
    let mut budget = ArtifactScanBudget::default();
    let mut removed = 0;
    reap_stage_publication_records(
        &project_anchor,
        STAGE,
        &publication_directory,
        &mut budget,
        &mut removed,
    )
    .unwrap();
    assert_eq!(removed, 1);
    assert!(!record.exists());
    assert_eq!(
        fs::read(stage_file_path(&staged.directory, "Content/remote.txt").unwrap()).unwrap(),
        b"remote"
    );

    drop(publication_directory);
    drop(project_anchor);
    drop(parent);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn private_publication_does_not_follow_a_parent_swap_after_anchor_open() {
    use std::os::unix::fs::symlink;

    let (root, _) = fixture();
    let private = operation_directory(&root, PUBLICATION_ROOT, STAGE).unwrap();
    let target = private.join("record.json");
    let moved = private.with_file_name("stage-held");
    let external = root.with_file_name(format!(
        "{}-outside",
        root.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(&external).unwrap();

    let result = publish_new_file_with_hook(&private, &target, STAGE, b"private bytes", || {
        fs::rename(&private, &moved).unwrap();
        symlink(&external, &private).unwrap();
    });

    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert!(!external.join("record.json").exists());
    assert!(!moved.join("record.json").exists());
    fs::remove_file(&private).unwrap();
    fs::remove_dir_all(&external).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn private_publication_removes_link_if_parent_moves_after_identity_check() {
    let (root, _) = fixture();
    let private = operation_directory(&root, PUBLICATION_ROOT, STAGE).unwrap();
    let target = private.join("record.json");
    let external = root.with_file_name(format!(
        "{}-moved-private-parent",
        root.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(&external).unwrap();
    let moved = external.join("publications");

    let result = publish_new_file_with_hooks(
        &private,
        &target,
        STAGE,
        b"must stay private",
        || {},
        || fs::rename(&private, &moved).unwrap(),
    );

    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert!(!moved.join("record.json").exists());
    assert!(fs::read_dir(&moved).unwrap().next().is_none());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), 0);

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}

#[cfg(unix)]
#[test]
fn project_publication_fails_closed_when_parent_becomes_external_symlink() {
    use std::os::unix::fs::symlink;

    let (root, _) = fixture();
    let parent = root.join("Content/nested");
    fs::create_dir_all(&parent).unwrap();
    let moved = parent.with_file_name("nested-held");
    let external = root.with_file_name(format!(
        "{}-outside",
        root.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(&external).unwrap();
    let target = parent.join("new-file.txt");

    let result = publish_project_file_with_hook(
        &root,
        &target,
        STAGE,
        "Content/nested/new-file.txt",
        b"project bytes",
        || {
            fs::rename(&parent, &moved).unwrap();
            symlink(&external, &parent).unwrap();
        },
    );

    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert!(!external.join("new-file.txt").exists());
    assert!(!moved.join("new-file.txt").exists());
    fs::remove_file(&parent).unwrap();
    fs::remove_dir_all(&external).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn project_publication_removes_link_if_parent_moves_after_identity_check() {
    let (root, _) = fixture();
    let parent = root.join("Content/nested");
    fs::create_dir_all(&parent).unwrap();
    let external = root.with_file_name(format!(
        "{}-moved-parent",
        root.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(&external).unwrap();
    let moved = external.join("nested");
    let target = parent.join("new-file.txt");

    let result = publish_project_file_with_hooks(
        &root,
        &target,
        STAGE,
        "Content/nested/new-file.txt",
        b"must stay in project",
        || {},
        || fs::rename(&parent, &moved).unwrap(),
        || {},
    );

    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert!(!moved.join("new-file.txt").exists());
    assert!(fs::read_dir(&moved).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(PUBLISH_TEMP_PREFIX)
    }));
    assert_eq!(cloud_artifact_usage(&root).unwrap(), 0);

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}

#[cfg(unix)]
#[test]
fn project_publication_retracts_link_when_parent_moves_after_hard_link() {
    let (root, _) = fixture();
    let parent = root.join("Content/nested");
    fs::create_dir_all(&parent).unwrap();
    let external = root.with_file_name(format!(
        "{}-post-link-move",
        root.file_name().unwrap().to_string_lossy()
    ));
    fs::create_dir_all(&external).unwrap();
    let moved = external.join("nested");
    let target = parent.join("new-file.txt");

    let result = publish_project_file_with_hooks(
        &root,
        &target,
        STAGE,
        "Content/nested/new-file.txt",
        b"must stay in project",
        || {},
        || {},
        || fs::rename(&parent, &moved).unwrap(),
    );

    assert_eq!(result, Err("hub_cloud_sync_stage_invalid"));
    assert!(!moved.join("new-file.txt").exists());
    assert!(fs::read_dir(&moved).unwrap().next().is_none());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), 0);

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}

#[cfg(windows)]
#[test]
fn project_publication_holds_windows_ancestors_during_swap_attempt() {
    let (root, _) = fixture();
    let parent = root.join("Content/nested");
    fs::create_dir_all(&parent).unwrap();
    let moved = parent.with_file_name("nested-held");
    let target = parent.join("new-file.txt");

    let result = publish_project_file_with_hook(
        &root,
        &target,
        STAGE,
        "Content/nested/new-file.txt",
        b"project bytes",
        || assert!(fs::rename(&parent, &moved).is_err()),
    );

    assert_eq!(result, Ok(()));
    assert_eq!(fs::read(target).unwrap(), b"project bytes");
    fs::remove_dir_all(root).unwrap();
}

fn manifest(files: &[(&str, &[u8])]) -> (Manifest, String, Vec<(FileEntry, Vec<u8>)>) {
    let mut blobs = files
        .iter()
        .map(|(path, bytes)| {
            (
                FileEntry {
                    path: (*path).into(),
                    digest: sha256(bytes),
                    bytes: bytes.len() as u64,
                },
                bytes.to_vec(),
            )
        })
        .collect::<Vec<_>>();
    blobs.sort_by(|left, right| left.0.path.cmp(&right.0.path));
    let entries = blobs
        .iter()
        .map(|(entry, _)| entry.clone())
        .collect::<Vec<_>>();
    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: ProjectGuid::new(),
            manifest_digest: ProjectManifestDigest::from_bytes(b"staging fixture"),
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
        files: entries.clone(),
    }
    .normalized()
    .unwrap();
    let digest = manifest.canonical_digest().unwrap();
    (manifest, digest, blobs)
}

pub(super) fn stage(root: &Path, guid: ProjectGuid, files: &[(&str, &[u8])]) -> StagedSnapshot {
    let (manifest, digest, blobs) = manifest(files);
    let snapshot = stage_snapshot(
        root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        "1",
        &digest,
        manifest,
        blobs,
    )
    .unwrap();
    assert_eq!(verify_project_guid(root, guid), Ok(()));
    snapshot
}

fn scoped_stage(
    root: &Path,
    stage_id: &str,
    organization_id: &str,
    project_id: &str,
    revision: &str,
    scope_fingerprint: &str,
    files: &[(&str, &[u8])],
) -> StagedSnapshot {
    let (manifest, digest, blobs) = manifest(files);
    let snapshot = prepare_scoped_download_stage(
        root,
        stage_id,
        organization_id,
        project_id,
        revision,
        &digest,
        scope_fingerprint,
        manifest,
    )
    .unwrap();
    for (entry, bytes) in blobs {
        store_download_blob(&snapshot, &entry, &bytes).unwrap();
    }
    snapshot
}

fn apply_with_project_lease(
    root: &Path,
    guid: ProjectGuid,
    snapshot: &StagedSnapshot,
    revision: &str,
) -> Result<ApplyOutcome, &'static str> {
    let project_lease = ProjectCloudSyncLease::acquire(root)?;
    apply_additions(root, guid, snapshot, revision, &project_lease)
}

#[test]
fn download_stage_identity_is_stable_and_scoped_to_local_project_and_revision() {
    let local_guid = ProjectGuid::new();
    let scope = CloudAccountScope {
        environment: crate::projects::CloudBindingEnvironment {
            issuer: "https://issuer.example".into(),
            client_id: "hub".into(),
            service_url: "https://service.example".into(),
        },
        subject: "user-1".into(),
    };
    let current = stable_download_stage_id(local_guid, &scope, ORGANIZATION, PROJECT, "8").unwrap();
    assert_eq!(
        current,
        stable_download_stage_id(local_guid, &scope, ORGANIZATION, PROJECT, "8").unwrap()
    );
    assert_ne!(
        current,
        stable_download_stage_id(local_guid, &scope, ORGANIZATION, PROJECT, "9").unwrap()
    );
    assert_ne!(
        current,
        stable_download_stage_id(ProjectGuid::new(), &scope, ORGANIZATION, PROJECT, "8").unwrap()
    );
    assert_ne!(
        current,
        stable_download_stage_id(local_guid, &scope, PROJECT, ORGANIZATION, "8").unwrap()
    );
    let mut other_account = scope.clone();
    other_account.subject = "user-2".into();
    assert_ne!(
        current,
        stable_download_stage_id(local_guid, &other_account, ORGANIZATION, PROJECT, "8").unwrap()
    );
    assert_ne!(
        download_scope_fingerprint(local_guid, &scope, ORGANIZATION, PROJECT),
        download_scope_fingerprint(local_guid, &other_account, ORGANIZATION, PROJECT)
    );
}

#[test]
fn stale_download_pruning_keeps_same_revision_other_accounts_and_conflict_recovery() {
    let (root, guid) = fixture();
    let scope = "a".repeat(64);
    let other_scope = "b".repeat(64);
    scoped_stage(
        &root,
        STALE_STAGE,
        ORGANIZATION,
        PROJECT,
        "2",
        &scope,
        &[("Content/old.txt", b"old")],
    );
    let conflict = scoped_stage(
        &root,
        CONFLICT_STAGE,
        ORGANIZATION,
        PROJECT,
        "3",
        &scope,
        &[("Content/conflict.txt", b"remote")],
    );
    scoped_stage(
        &root,
        OTHER_SCOPE_STAGE,
        ORGANIZATION,
        PROJECT,
        "1",
        &other_scope,
        &[("Content/other.txt", b"other")],
    );
    scoped_stage(
        &root,
        CURRENT_STAGE,
        ORGANIZATION,
        PROJECT,
        "5",
        &scope,
        &[("Content/current.txt", b"current")],
    );
    fs::create_dir_all(root.join("Content")).unwrap();
    fs::write(root.join("Content/conflict.txt"), b"local edit").unwrap();
    assert!(matches!(
        apply_with_project_lease(&root, guid, &conflict, "3").unwrap(),
        ApplyOutcome::Conflict { .. }
    ));

    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(
        prune_stale_download_stages(&root, ORGANIZATION, PROJECT, &scope, "5", &project_lease,),
        Ok(1)
    );
    assert!(!root
        .join(format!(".zircon/cloud/staging/{STALE_STAGE}"))
        .exists());
    assert!(root
        .join(format!(".zircon/cloud/staging/{CONFLICT_STAGE}"))
        .exists());
    assert!(root
        .join(format!(".zircon/cloud/staging/{OTHER_SCOPE_STAGE}"))
        .exists());
    assert!(root
        .join(format!(".zircon/cloud/staging/{CURRENT_STAGE}"))
        .exists());
    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_download_discard_checks_scope_and_revision_and_preserves_local_files() {
    let (root, guid) = fixture();
    let scope = "c".repeat(64);
    let staged = scoped_stage(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        "7",
        &scope,
        &[("Content/conflict.txt", b"remote")],
    );
    fs::create_dir_all(root.join("Content")).unwrap();
    fs::write(root.join("Content/conflict.txt"), b"local edit").unwrap();
    assert!(matches!(
        apply_with_project_lease(&root, guid, &staged, "7").unwrap(),
        ApplyOutcome::Conflict { .. }
    ));

    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert!(discard_bound_download_stage(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        "7",
        &"d".repeat(64),
        &project_lease,
    )
    .is_err());
    assert!(staged.directory.exists());
    discard_bound_download_stage(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        "7",
        &scope,
        &project_lease,
    )
    .unwrap();
    assert!(!staged.directory.exists());
    assert_eq!(
        fs::read(root.join("Content/conflict.txt")).unwrap(),
        b"local edit"
    );
    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn private_recovery_and_pruning_reject_a_symlinked_zircon_parent() {
    use std::os::unix::fs::symlink;

    let (project_root, _) = fixture();
    let (external_root, _) = fixture();
    let scope = "e".repeat(64);
    let external_stage = scoped_stage(
        &external_root,
        STALE_STAGE,
        ORGANIZATION,
        PROJECT,
        "1",
        &scope,
        &[("Content/remote.txt", b"remote")],
    );
    symlink(external_root.join(".zircon"), project_root.join(".zircon")).unwrap();
    let project_lease = ProjectCloudSyncLease::acquire(&project_root).unwrap();

    assert_eq!(
        recover_orphaned_publish_temporaries(&project_root, &project_lease),
        Err("hub_cloud_sync_stage_invalid")
    );
    assert_eq!(
        prune_stale_download_stages(
            &project_root,
            ORGANIZATION,
            PROJECT,
            &scope,
            "2",
            &project_lease,
        ),
        Err("hub_cloud_sync_stage_invalid")
    );
    assert!(external_stage.directory.exists());

    drop(project_lease);
    fs::remove_dir_all(project_root).unwrap();
    fs::remove_dir_all(external_root).unwrap();
}

#[test]
fn concurrent_same_content_publishers_never_replace_or_publish_partial_bytes() {
    let (root, _) = fixture();
    let parent = root.join("Content");
    fs::create_dir_all(&parent).unwrap();
    let target = parent.join("remote.txt");
    let writers = (0..8)
        .map(|_| {
            let root = root.clone();
            let target = target.clone();
            std::thread::spawn(move || publish_new_file(&root, &target, STAGE, b"complete"))
        })
        .collect::<Vec<_>>();
    let results = writers
        .into_iter()
        .map(|writer| writer.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(results
        .iter()
        .all(|result| { result.is_ok() || *result == Err("hub_cloud_sync_stage_exists") }));
    assert_eq!(fs::read(&target).unwrap(), b"complete");
    assert_eq!(fs::read_dir(&parent).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn conflict_artifact_quota_reserves_its_actual_serialized_path_bytes() {
    let (root, guid) = fixture();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);
    let paths = (0..MAX_CONFLICT_PATHS)
        .map(|index| format!("Content/{index:03}-{}", "x".repeat(470)))
        .collect::<Vec<_>>();
    let artifact = ConflictArtifact {
        schema_version: STAGE_SCHEMA_VERSION,
        stage_id: staged.stage_id.clone(),
        revision: staged.revision.clone(),
        manifest_digest: staged.manifest_digest.clone(),
        conflict_count: paths.len(),
        paths: paths.clone(),
    };
    let artifact_bytes = serde_json::to_vec(&artifact).unwrap();
    assert!(artifact_bytes.len() as u64 > 65_536);
    let current = cloud_artifact_usage(&root).unwrap();
    assert_eq!(
        write_conflict_artifact_with_limit(
            &staged,
            &paths,
            paths.len(),
            current + artifact_bytes.len() as u64 - 1,
        ),
        Err("hub_cloud_sync_storage_quota_exceeded")
    );
    write_conflict_artifact_with_limit(
        &staged,
        &paths,
        paths.len(),
        current + artifact_bytes.len() as u64,
    )
    .unwrap();
    assert_eq!(
        cloud_artifact_usage(&root).unwrap(),
        current + artifact_bytes.len() as u64
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn verified_stage_adds_files_without_removing_local_only_files_and_retries_idempotently() {
    let (root, guid) = fixture();
    fs::create_dir_all(root.join("Local")).unwrap();
    fs::write(root.join("Local/only-here.txt"), b"keep").unwrap();
    let staged = stage(&root, guid, &[("Content/remote.txt", b"remote")]);

    assert_eq!(
        apply_with_project_lease(&root, guid, &staged, "1").unwrap(),
        ApplyOutcome::Applied {
            applied_files: 1,
            unchanged_files: 0
        }
    );
    assert_eq!(
        fs::read(root.join("Content/remote.txt")).unwrap(),
        b"remote"
    );
    assert_eq!(fs::read(root.join("Local/only-here.txt")).unwrap(), b"keep");
    assert_eq!(
        apply_with_project_lease(&root, guid, &staged, "1").unwrap(),
        ApplyOutcome::Applied {
            applied_files: 0,
            unchanged_files: 1
        }
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_local_conflict_preflights_every_file_and_keeps_staging_for_recovery() {
    let (root, guid) = fixture();
    fs::create_dir_all(root.join("Content")).unwrap();
    fs::write(root.join("Content/conflict.txt"), b"local edit").unwrap();
    let staged = stage(
        &root,
        guid,
        &[
            ("Content/new.txt", b"new"),
            ("Content/conflict.txt", b"remote"),
        ],
    );

    assert_eq!(
        apply_with_project_lease(&root, guid, &staged, "1").unwrap(),
        ApplyOutcome::Conflict {
            paths: vec!["Content/conflict.txt".into()],
            conflict_count: 1
        }
    );
    assert!(!root.join("Content/new.txt").exists());
    assert_eq!(
        fs::read(root.join("Content/conflict.txt")).unwrap(),
        b"local edit"
    );
    assert_eq!(
        load_staged_snapshot(&root, STAGE, "1")
            .unwrap()
            .manifest_digest,
        staged.manifest_digest
    );
    let conflict_artifact = fs::read_dir(&staged.directory)
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(CONFLICT_ARTIFACT_PREFIX)
        })
        .expect("durable conflict artifact");
    let artifact: ConflictArtifact =
        serde_json::from_slice(&fs::read(conflict_artifact.path()).unwrap()).unwrap();
    assert_eq!(artifact.stage_id, STAGE);
    assert_eq!(artifact.revision, "1");
    assert_eq!(artifact.paths, vec!["Content/conflict.txt".to_owned()]);

    fs::remove_file(root.join("Content/conflict.txt")).unwrap();
    assert_eq!(
        apply_with_project_lease(&root, guid, &staged, "1").unwrap(),
        ApplyOutcome::Applied {
            applied_files: 2,
            unchanged_files: 0
        }
    );
    assert_eq!(fs::read(root.join("Content/new.txt")).unwrap(), b"new");
    assert_eq!(
        fs::read(root.join("Content/conflict.txt")).unwrap(),
        b"remote"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn incomplete_or_private_remote_paths_cannot_be_applied() {
    let (root, guid) = fixture();
    let (manifest, digest, mut blobs) = manifest(&[("Content/remote.txt", b"remote")]);
    blobs.clear();
    assert!(stage_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        "1",
        &digest,
        manifest,
        blobs
    )
    .is_err());
    assert!(!stage_path_allowed(".zircon/session.lock"));
    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert!(apply_additions(
        &root,
        guid,
        &StagedSnapshot {
            stage_id: STAGE.into(),
            organization_id: ORGANIZATION.into(),
            project_id: PROJECT.into(),
            revision: "1".into(),
            manifest_digest: "a".repeat(64),
            manifest: Manifest {
                schema_version: 1,
                engine: "Zircon".into(),
                package_lock_digest: "a".repeat(64),
                package_lock: None,
                ignore_policy: "zircon-project-v1".into(),
                source_revision: None,
                files: vec![FileEntry {
                    path: ".zircon/session.lock".into(),
                    digest: "a".repeat(64),
                    bytes: 1
                }]
            },
            directory: stage_directory(&root, STAGE).unwrap()
        },
        "1",
        &project_lease,
    )
    .is_err());
    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn upload_snapshot_and_blobs_resume_only_for_the_same_operation_scope() {
    let (root, guid) = fixture();
    let scope_fingerprint = "c".repeat(64);
    let (manifest, _, blobs) = manifest(&[
        ("Content/scene.txt", b"scene"),
        ("Assets/icon.bin", b"icon"),
    ]);
    let upload = prepare_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        manifest.clone(),
    )
    .unwrap();
    for (entry, bytes) in &blobs {
        store_upload_blob(&upload, entry, bytes).unwrap();
    }

    let resumed = find_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
    )
    .unwrap()
    .unwrap();
    assert_eq!(resumed.operation_id, STAGE);
    assert_eq!(resumed.base_revision, "7");
    assert_eq!(resumed.manifest, manifest);
    for (entry, bytes) in blobs {
        assert_eq!(read_upload_blob(&resumed, &entry).unwrap(), bytes);
    }
    assert!(find_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        "00000000-0000-4000-8000-000000000099",
        guid,
        &scope_fingerprint,
    )
    .is_err());
    assert!(
        find_upload_snapshot(&root, STAGE, ORGANIZATION, PROJECT, guid, &"d".repeat(64),).is_err()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn terminal_upload_discard_requires_the_same_scope_and_releases_quota() {
    let (root, guid) = fixture();
    let scope_fingerprint = "e".repeat(64);
    let (manifest, _, blobs) = manifest(&[("Content/scene.txt", b"scene snapshot")]);
    let upload = prepare_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        manifest,
    )
    .unwrap();
    for (entry, bytes) in blobs {
        store_upload_blob(&upload, &entry, &bytes).unwrap();
    }
    let before = cloud_artifact_usage(&root).unwrap();
    assert!(before > 0);

    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert!(discard_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &"f".repeat(64),
        &project_lease,
    )
    .is_err());
    assert!(upload.directory.exists());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), before);

    discard_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        &project_lease,
    )
    .unwrap();
    assert!(!upload.directory.exists());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), 0);
    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn restart_prunes_only_exact_scope_terminal_uploads_and_releases_their_quota() {
    let (root, guid) = fixture();
    let scope_fingerprint = "e".repeat(64);
    let other_project = "00000000-0000-4000-8000-000000000099";
    let (manifest, _, blobs) = manifest(&[("Content/scene.txt", b"scene snapshot")]);
    for (operation_id, project_id) in [
        (STAGE, PROJECT),
        (NEXT_STAGE, PROJECT),
        (OTHER_STAGE, other_project),
    ] {
        let upload = prepare_upload_snapshot(
            &root,
            operation_id,
            ORGANIZATION,
            project_id,
            guid,
            &scope_fingerprint,
            "7",
            manifest.clone(),
        )
        .unwrap();
        for (entry, bytes) in &blobs {
            store_upload_blob(&upload, entry, bytes).unwrap();
        }
    }
    let before = cloud_artifact_usage(&root).unwrap();
    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    let operations = [
        CloudCommitCleanupRecord {
            operation_id: STAGE.into(),
            organization_id: ORGANIZATION.into(),
            project_id: PROJECT.into(),
            status: OperationStatus::Unknown,
        },
        CloudCommitCleanupRecord {
            operation_id: NEXT_STAGE.into(),
            organization_id: ORGANIZATION.into(),
            project_id: PROJECT.into(),
            status: OperationStatus::Conflict,
        },
        CloudCommitCleanupRecord {
            operation_id: OTHER_STAGE.into(),
            organization_id: ORGANIZATION.into(),
            project_id: other_project.into(),
            status: OperationStatus::Committed,
        },
    ];

    assert_eq!(
        prune_terminal_upload_snapshots(
            &root,
            ORGANIZATION,
            PROJECT,
            guid,
            &scope_fingerprint,
            &operations,
            &project_lease,
        )
        .unwrap(),
        1
    );
    let uploads = root.join(".zircon/cloud/uploads");
    assert!(
        uploads.join(STAGE).exists(),
        "Unknown keeps its retry bytes"
    );
    assert!(!uploads.join(NEXT_STAGE).exists());
    assert!(
        uploads.join(OTHER_STAGE).exists(),
        "a different project keeps its bytes"
    );
    assert!(cloud_artifact_usage(&root).unwrap() < before);
    assert_eq!(
        prune_terminal_upload_snapshots(
            &root,
            ORGANIZATION,
            PROJECT,
            guid,
            &scope_fingerprint,
            &operations,
            &project_lease,
        )
        .unwrap(),
        0,
        "a second startup cleanup is safe after an interrupted terminal cleanup"
    );
    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn terminal_recovery_removes_an_empty_directory_left_after_marker_unlink() {
    let (root, guid) = fixture();
    let scope_fingerprint = "e".repeat(64);
    let operation_id = "00000000-0000-4000-8000-000000000009";
    let (manifest, _, blobs) = manifest(&[("Content/scene.txt", b"scene snapshot")]);
    let upload = prepare_upload_snapshot(
        &root,
        operation_id,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        manifest,
    )
    .unwrap();
    for (entry, bytes) in &blobs {
        store_upload_blob(&upload, entry, bytes).unwrap();
    }

    // This is the durable state after the previous cleanup deleted all content and
    // unlinked the marker, then the process exited before removing the operation root.
    for (entry, _) in blobs {
        fs::remove_file(stage_file_path(&upload.directory, &entry.path).unwrap()).unwrap();
    }
    fs::remove_file(upload.directory.join(UPLOAD_MARKER)).unwrap();
    assert!(fs::read_dir(&upload.directory).unwrap().next().is_none());
    assert!(cloud_artifact_usage(&root).unwrap() > 0);

    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(
        prune_terminal_upload_snapshots(
            &root,
            ORGANIZATION,
            PROJECT,
            guid,
            &scope_fingerprint,
            &[CloudCommitCleanupRecord {
                operation_id: operation_id.into(),
                organization_id: ORGANIZATION.into(),
                project_id: PROJECT.into(),
                status: OperationStatus::Committed,
            }],
            &project_lease,
        )
        .unwrap(),
        1
    );
    assert!(!upload.directory.exists());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), 0);

    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unadmitted_recovery_keeps_nonempty_markerless_data_and_continues_pruning() {
    let (root, guid) = fixture();
    let scope_fingerprint = "e".repeat(64);
    let (manifest, _, blobs) = manifest(&[("Content/scene.txt", b"scene snapshot")]);
    let markerless = prepare_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        manifest.clone(),
    )
    .unwrap();
    let reclaimable = prepare_upload_snapshot(
        &root,
        NEXT_STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        manifest,
    )
    .unwrap();
    for (entry, bytes) in blobs {
        store_upload_blob(&reclaimable, &entry, &bytes).unwrap();
    }
    fs::remove_file(markerless.directory.join(UPLOAD_MARKER)).unwrap();
    let first = markerless.directory.join("interrupted-first.bin");
    let second = markerless.directory.join("interrupted-second.bin");
    fs::write(&first, b"preserve first").unwrap();
    fs::write(&second, b"preserve second").unwrap();

    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(
        prune_unadmitted_upload_snapshots(
            &root,
            ORGANIZATION,
            PROJECT,
            guid,
            &scope_fingerprint,
            &[],
            &project_lease,
        )
        .unwrap(),
        1
    );
    assert!(markerless.directory.exists());
    assert_eq!(fs::read(first).unwrap(), b"preserve first");
    assert_eq!(fs::read(second).unwrap(), b"preserve second");
    assert!(!reclaimable.directory.exists());

    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn terminal_recovery_keeps_a_reused_operation_id_from_another_account_scope() {
    let (root, guid) = fixture();
    let current_scope = "c".repeat(64);
    let other_scope = "d".repeat(64);
    let (manifest, _, blobs) = manifest(&[("Content/scene.txt", b"other account")]);
    let upload = prepare_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &other_scope,
        "7",
        manifest,
    )
    .unwrap();
    for (entry, bytes) in blobs {
        store_upload_blob(&upload, &entry, &bytes).unwrap();
    }
    let before = cloud_artifact_usage(&root).unwrap();
    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(
        prune_terminal_upload_snapshots(
            &root,
            ORGANIZATION,
            PROJECT,
            guid,
            &current_scope,
            &[CloudCommitCleanupRecord {
                operation_id: STAGE.into(),
                organization_id: ORGANIZATION.into(),
                project_id: PROJECT.into(),
                status: OperationStatus::Committed,
            }],
            &project_lease,
        )
        .unwrap(),
        0
    );
    assert!(upload.directory.exists());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), before);
    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn startup_recovery_prunes_only_unjournaled_uploads_matching_the_full_local_scope() {
    let (root, guid) = fixture();
    let scope_fingerprint = "c".repeat(64);
    let other_scope_fingerprint = "d".repeat(64);
    let known_unknown_id = "00000000-0000-4000-8000-000000000005";
    let other_scope_id = "00000000-0000-4000-8000-000000000006";
    let corrupt_marker_id = "00000000-0000-4000-8000-000000000007";

    let (partial_manifest, _, _) = manifest(&[("Content/partial.txt", b"partial")]);
    prepare_upload_snapshot(
        &root,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        partial_manifest,
    )
    .unwrap();

    let (complete_manifest, _, complete_blobs) = manifest(&[("Content/complete.txt", b"complete")]);
    let complete = prepare_upload_snapshot(
        &root,
        STALE_STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        complete_manifest,
    )
    .unwrap();
    for (entry, bytes) in complete_blobs {
        store_upload_blob(&complete, &entry, &bytes).unwrap();
    }

    let (known_manifest, _, _) = manifest(&[("Content/unknown.txt", b"unknown")]);
    prepare_upload_snapshot(
        &root,
        known_unknown_id,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        known_manifest,
    )
    .unwrap();

    let (other_manifest, _, _) = manifest(&[("Content/other-account.txt", b"other")]);
    prepare_upload_snapshot(
        &root,
        other_scope_id,
        ORGANIZATION,
        PROJECT,
        guid,
        &other_scope_fingerprint,
        "7",
        other_manifest,
    )
    .unwrap();

    let corrupt_directory = operation_directory(&root, UPLOAD_ROOT, corrupt_marker_id).unwrap();
    fs::write(corrupt_directory.join(UPLOAD_MARKER), b"not-json").unwrap();

    let before = cloud_artifact_usage(&root).unwrap();
    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    let removed = prune_unadmitted_upload_snapshots(
        &root,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        &[known_unknown_id.to_owned(), other_scope_id.to_owned()],
        &project_lease,
    )
    .unwrap();

    assert_eq!(removed, 2);
    assert!(!root.join(".zircon/cloud/uploads").join(STAGE).exists());
    assert!(!complete.directory.exists());
    assert!(operation_directory(&root, UPLOAD_ROOT, known_unknown_id)
        .unwrap()
        .exists());
    assert!(find_upload_snapshot(
        &root,
        other_scope_id,
        ORGANIZATION,
        PROJECT,
        guid,
        &other_scope_fingerprint,
    )
    .unwrap()
    .is_some());
    assert!(corrupt_directory.exists());
    assert!(cloud_artifact_usage(&root).unwrap() < before);

    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn upload_recovery_bounds_identity_prefix_reads_for_large_manifests() {
    let (root, guid) = fixture();
    let scope_fingerprint = "c".repeat(64);
    let marker = format!(
        "{{\"schemaVersion\":1,\"operationId\":\"{STAGE}\",\"organizationId\":\"{ORGANIZATION}\",\"projectId\":\"{PROJECT}\",\"projectGuid\":\"{guid}\",\"scopeFingerprint\":\"{scope_fingerprint}\",\"baseRevision\":\"7\",\"manifestDigest\":\"{}\",\"manifest\":\"{}",
        "a".repeat(64),
        "x".repeat(64 * 1024),
    );
    let directory = operation_directory(&root, UPLOAD_ROOT, STAGE).unwrap();
    let path = directory.join(UPLOAD_MARKER);
    fs::write(&path, marker.as_bytes()).unwrap();

    let mut budget = ArtifactScanBudget::default();
    let header = read_upload_marker_header(&root, &path, &mut budget).unwrap();
    assert!(upload_marker_header_matches_scope(
        &header,
        STAGE,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
    ));
    assert_eq!(budget.read_bytes, MAX_UPLOAD_MARKER_HEADER_BYTES as u64);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn upload_recovery_rejects_full_marker_read_before_pruning_when_aggregate_budget_is_exhausted() {
    let (root, guid) = fixture();
    let scope_fingerprint = "c".repeat(64);
    let operation_id = "00000000-0000-4000-8000-000000000009";
    let paths = (0..24)
        .map(|index| format!("Content/{index:02}-{}", "x".repeat(80)))
        .collect::<Vec<_>>();
    let content = [b'x'];
    let files = paths
        .iter()
        .map(|path| (path.as_str(), content.as_slice()))
        .collect::<Vec<_>>();
    let (manifest, _, _) = manifest(&files);
    let upload = prepare_upload_snapshot(
        &root,
        operation_id,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        manifest,
    )
    .unwrap();
    let marker_path = upload.directory.join(UPLOAD_MARKER);
    let marker_len = fs::metadata(&marker_path).unwrap().len();
    assert!(marker_len > MAX_UPLOAD_MARKER_HEADER_BYTES as u64);

    let remaining_after_header = MAX_CLOUD_UPLOAD_RECOVERY_READ_BYTES - marker_len + 1;
    let mut budget = ArtifactScanBudget {
        read_bytes: remaining_after_header - MAX_UPLOAD_MARKER_HEADER_BYTES as u64,
        ..ArtifactScanBudget::default()
    };
    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(
        prune_unadmitted_upload_snapshots_with_budget(
            &root,
            ORGANIZATION,
            PROJECT,
            guid,
            &scope_fingerprint,
            &[],
            &mut budget,
            &project_lease,
        ),
        Err("hub_cloud_sync_storage_quota_exceeded")
    );

    assert_eq!(budget.read_bytes, remaining_after_header);
    assert!(upload.directory.exists());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), marker_len);

    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn startup_recovery_finishes_marker_last_cleanup_after_an_interrupted_delete() {
    let (root, guid) = fixture();
    let scope_fingerprint = "c".repeat(64);
    let operation_id = "00000000-0000-4000-8000-000000000008";
    let (manifest, _, blobs) = manifest(&[
        ("Content/first.txt", b"first"),
        ("Assets/second.txt", b"second"),
    ]);
    let upload = prepare_upload_snapshot(
        &root,
        operation_id,
        ORGANIZATION,
        PROJECT,
        guid,
        &scope_fingerprint,
        "7",
        manifest,
    )
    .unwrap();
    for (entry, bytes) in blobs {
        store_upload_blob(&upload, &entry, &bytes).unwrap();
    }
    fs::remove_file(stage_file_path(&upload.directory, "Content/first.txt").unwrap()).unwrap();
    assert!(upload.directory.join(UPLOAD_MARKER).exists());

    let project_lease = ProjectCloudSyncLease::acquire(&root).unwrap();
    assert_eq!(
        prune_unadmitted_upload_snapshots(
            &root,
            ORGANIZATION,
            PROJECT,
            guid,
            &scope_fingerprint,
            &[],
            &project_lease,
        )
        .unwrap(),
        1
    );
    assert!(!upload.directory.exists());
    assert_eq!(cloud_artifact_usage(&root).unwrap(), 0);

    drop(project_lease);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn managed_directory_removal_refuses_a_deep_tree_before_deleting_anything() {
    let (root, _) = fixture();
    let directory = root.join(format!(".zircon/cloud/staging/{STAGE}"));
    let leaf = directory.join("nested/deeper/partial.bin");
    fs::create_dir_all(leaf.parent().unwrap()).unwrap();
    fs::write(&leaf, b"preserve on budget failure").unwrap();

    let mut budget = ArtifactScanBudget::default();
    budget.directories = MAX_ARTIFACT_DIRECTORIES - 2;
    assert_eq!(
        remove_managed_directory_with_budget(&directory, &mut budget),
        Err("hub_cloud_sync_storage_quota_exceeded")
    );
    assert_eq!(fs::read(&leaf).unwrap(), b"preserve on budget failure");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn managed_directory_removal_rejects_nested_symlinks_without_touching_the_target() {
    use std::os::unix::fs::symlink;

    let (root, _) = fixture();
    let (external, _) = fixture();
    let directory = root.join(format!(".zircon/cloud/staging/{STAGE}"));
    fs::create_dir_all(&directory).unwrap();
    let outside = external.join("outside.bin");
    fs::write(&outside, b"outside").unwrap();
    symlink(&outside, directory.join("outside-link.bin")).unwrap();

    let mut budget = ArtifactScanBudget::default();
    assert_eq!(
        remove_managed_directory_with_budget(&directory, &mut budget),
        Err("hub_cloud_sync_stage_invalid")
    );
    assert_eq!(fs::read(outside).unwrap(), b"outside");
    assert!(directory.exists());
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(external).unwrap();
}
