mod recovery;

#[cfg(test)]
use recovery::reap_stage_publication_records;
pub(crate) use recovery::recover_orphaned_publish_temporaries;

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    account::operations::{CloudCommitCleanupRecord, OperationStatus},
    file_io::{AnchoredDirectory, TreeRemovalLimits, TreeRemovalUsage},
    projects::CloudAccountScope,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zircon_runtime_interface::project::ProjectGuid;

use super::{
    manifest::{
        valid_path, FileEntry, Manifest, MAX_BLOB_BYTES, MAX_MANIFEST_BYTES, MAX_PROJECT_BYTES,
    },
    snapshot::verify_project_guid,
    sync::ProjectCloudSyncLease,
};

const STAGE_ROOT: &[&str] = &[".zircon", "cloud", "staging"];
const UPLOAD_ROOT: &[&str] = &[".zircon", "cloud", "uploads"];
const PUBLICATION_ROOT: &[&str] = &[".zircon", "cloud", "publications"];
const STAGE_MARKER: &str = ".cloud-sync-stage.json";
const UPLOAD_MARKER: &str = ".cloud-sync-upload.json";
const CONFLICT_ARTIFACT_PREFIX: &str = ".cloud-sync-conflict-";
const STAGE_SCHEMA_VERSION: u32 = 1;
const PUBLICATION_SCHEMA_VERSION: u32 = 1;
const MAX_CONFLICT_PATHS: usize = 256;
const MAX_CLOUD_ARTIFACT_BYTES: u64 =
    MAX_PROJECT_BYTES * 2 + (MAX_MANIFEST_BYTES as u64 * 2) + 65_536;
const MAX_ARTIFACT_ENTRIES: usize = 50_000;
const MAX_ARTIFACT_DIRECTORIES: usize = 10_000;
const MAX_CLOUD_DIRECTORY_DEPTH: usize = 256;
const MAX_UPLOAD_MARKER_HEADER_BYTES: usize = 1024;
const MAX_CLOUD_UPLOAD_RECOVERY_READ_BYTES: u64 = 64 * 1024 * 1024;
const PUBLISH_TEMP_PREFIX: &str = ".hub-cloud-sync-tmp-";
static NEXT_PUBLISH_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StageMarker {
    schema_version: u32,
    stage_id: String,
    organization_id: String,
    project_id: String,
    revision: String,
    manifest_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope_fingerprint: Option<String>,
    manifest: Manifest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UploadMarker {
    schema_version: u32,
    operation_id: String,
    organization_id: String,
    project_id: String,
    project_guid: ProjectGuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope_fingerprint: Option<String>,
    base_revision: String,
    manifest_digest: String,
    manifest: Manifest,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UploadMarkerHeader {
    schema_version: u32,
    operation_id: String,
    organization_id: String,
    project_id: String,
    project_guid: ProjectGuid,
    #[serde(default)]
    scope_fingerprint: Option<String>,
    base_revision: String,
    manifest_digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConflictArtifact {
    schema_version: u32,
    stage_id: String,
    revision: String,
    manifest_digest: String,
    conflict_count: usize,
    paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublicationArtifact {
    schema_version: u32,
    stage_id: String,
    relative_path: String,
    temporary_name: String,
    digest: String,
    bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ApplyOutcome {
    Applied {
        applied_files: usize,
        unchanged_files: usize,
    },
    Conflict {
        paths: Vec<String>,
        conflict_count: usize,
    },
}

#[derive(Clone)]
pub(crate) struct StagedSnapshot {
    project_root: PathBuf,
    pub(crate) stage_id: String,
    pub(crate) organization_id: String,
    pub(crate) project_id: String,
    pub(crate) revision: String,
    pub(crate) manifest_digest: String,
    pub(crate) manifest: Manifest,
    pub(crate) directory: PathBuf,
}

#[derive(Clone)]
pub(crate) struct UploadSnapshot {
    project_root: PathBuf,
    pub(crate) operation_id: String,
    pub(crate) organization_id: String,
    pub(crate) project_id: String,
    pub(crate) base_revision: String,
    pub(crate) manifest_digest: String,
    pub(crate) manifest: Manifest,
    directory: PathBuf,
    marker_bytes: Vec<u8>,
}

pub(crate) fn stage_directory(root: &Path, stage_id: &str) -> Result<PathBuf, &'static str> {
    validate_stage_id(stage_id)?;
    operation_directory(root, STAGE_ROOT, stage_id)
}

/// A deterministic id lets a later Hub process reopen and finish an interrupted
/// download for this local project and exact remote revision.
pub(crate) fn stable_download_stage_id(
    project_guid: ProjectGuid,
    scope: &CloudAccountScope,
    organization_id: &str,
    project_id: &str,
    revision: &str,
) -> Result<String, &'static str> {
    validate_stage_id(organization_id)?;
    validate_stage_id(project_id)?;
    validate_revision(revision)?;
    let local_id = project_guid.to_string();
    let mut hasher = Sha256::new();
    for component in [
        b"zircon-hub-cloud-stage-v1".as_slice(),
        local_id.as_bytes(),
        scope.environment.issuer.as_bytes(),
        scope.environment.client_id.as_bytes(),
        scope.environment.service_url.as_bytes(),
        scope.subject.as_bytes(),
        organization_id.as_bytes(),
        project_id.as_bytes(),
        revision.as_bytes(),
    ] {
        hasher.update((component.len() as u64).to_be_bytes());
        hasher.update(component);
    }
    let mut digest = format!("{:x}", hasher.finalize());
    digest.replace_range(12..13, "5");
    let variant = (digest.as_bytes()[16] as char).to_digit(16).unwrap_or(0) & 0x03;
    digest.replace_range(
        16..17,
        &char::from_digit(8 + variant, 16).unwrap_or('8').to_string(),
    );
    let stage_id = format!(
        "{}-{}-{}-{}-{}",
        &digest[..8],
        &digest[8..12],
        &digest[12..16],
        &digest[16..20],
        &digest[20..32]
    );
    validate_stage_id(&stage_id)?;
    Ok(stage_id)
}

pub(crate) fn download_scope_fingerprint(
    project_guid: ProjectGuid,
    scope: &CloudAccountScope,
    organization_id: &str,
    project_id: &str,
) -> String {
    let mut hasher = Sha256::new();
    let local_id = project_guid.to_string();
    for component in [
        b"zircon-hub-cloud-scope-v1".as_slice(),
        local_id.as_bytes(),
        scope.environment.issuer.as_bytes(),
        scope.environment.client_id.as_bytes(),
        scope.environment.service_url.as_bytes(),
        scope.subject.as_bytes(),
        organization_id.as_bytes(),
        project_id.as_bytes(),
    ] {
        hasher.update((component.len() as u64).to_be_bytes());
        hasher.update(component);
    }
    format!("{:x}", hasher.finalize())
}

pub(crate) fn stage_snapshot(
    root: &Path,
    stage_id: &str,
    organization_id: &str,
    project_id: &str,
    revision: &str,
    manifest_digest: &str,
    manifest: Manifest,
    blobs: impl IntoIterator<Item = (FileEntry, Vec<u8>)>,
) -> Result<StagedSnapshot, &'static str> {
    let snapshot = prepare_download_stage(
        root,
        stage_id,
        organization_id,
        project_id,
        revision,
        manifest_digest,
        manifest,
    )?;
    for (entry, bytes) in blobs {
        store_download_blob(&snapshot, &entry, &bytes)?;
    }
    verify_staged_files(&snapshot)?;
    Ok(snapshot)
}

pub(crate) fn prepare_download_stage(
    root: &Path,
    stage_id: &str,
    organization_id: &str,
    project_id: &str,
    revision: &str,
    manifest_digest: &str,
    manifest: Manifest,
) -> Result<StagedSnapshot, &'static str> {
    prepare_download_stage_with_scope(
        root,
        stage_id,
        organization_id,
        project_id,
        revision,
        manifest_digest,
        None,
        manifest,
    )
}

pub(crate) fn prepare_scoped_download_stage(
    root: &Path,
    stage_id: &str,
    organization_id: &str,
    project_id: &str,
    revision: &str,
    manifest_digest: &str,
    scope_fingerprint: &str,
    manifest: Manifest,
) -> Result<StagedSnapshot, &'static str> {
    if scope_fingerprint.len() != 64
        || !scope_fingerprint
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    prepare_download_stage_with_scope(
        root,
        stage_id,
        organization_id,
        project_id,
        revision,
        manifest_digest,
        Some(scope_fingerprint),
        manifest,
    )
}

fn prepare_download_stage_with_scope(
    root: &Path,
    stage_id: &str,
    organization_id: &str,
    project_id: &str,
    revision: &str,
    manifest_digest: &str,
    scope_fingerprint: Option<&str>,
    manifest: Manifest,
) -> Result<StagedSnapshot, &'static str> {
    let project_root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let directory = stage_directory(&project_root, stage_id)?;
    let calculated = manifest
        .canonical_digest()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    manifest
        .require_present_package_lock()
        .map_err(|_| "hub_cloud_sync_package_lock_incompatible")?;
    if calculated != manifest_digest
        || manifest
            .files
            .iter()
            .any(|entry| !stage_path_allowed(&entry.path))
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let marker = StageMarker {
        schema_version: STAGE_SCHEMA_VERSION,
        stage_id: stage_id.to_owned(),
        organization_id: organization_id.to_owned(),
        project_id: project_id.to_owned(),
        revision: revision.to_owned(),
        manifest_digest: manifest_digest.to_owned(),
        scope_fingerprint: scope_fingerprint.map(str::to_owned),
        manifest,
    };
    ensure_artifact_capacity(
        &project_root,
        &directory,
        expected_artifact_bytes(&marker.manifest, &marker)?,
    )?;
    admit_or_write_marker(&project_root, &directory, &marker)?;
    Ok(staged_snapshot(project_root, directory, marker))
}

pub(crate) fn store_download_blob(
    snapshot: &StagedSnapshot,
    entry: &FileEntry,
    bytes: &[u8],
) -> Result<(), &'static str> {
    if snapshot
        .manifest
        .files
        .iter()
        .find(|candidate| candidate.path == entry.path)
        != Some(entry)
        || bytes.len() as u64 != entry.bytes
        || sha256(bytes) != entry.digest
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    stage_blob(
        &snapshot.project_root,
        &snapshot.directory,
        &entry.path,
        bytes,
    )
}

pub(crate) fn prepare_upload_snapshot(
    root: &Path,
    operation_id: &str,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
    base_revision: &str,
    manifest: Manifest,
) -> Result<UploadSnapshot, &'static str> {
    validate_stage_id(operation_id)?;
    validate_revision(base_revision)?;
    if !is_sha256_digest(scope_fingerprint) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    if manifest
        .files
        .iter()
        .any(|entry| !stage_path_allowed(&entry.path))
    {
        return Err("hub_cloud_sync_snapshot_invalid");
    }
    let project_root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let manifest_digest = manifest
        .canonical_digest()
        .map_err(|_| "hub_cloud_sync_snapshot_invalid")?;
    manifest
        .require_present_package_lock()
        .map_err(|_| "hub_cloud_sync_package_lock_incompatible")?;
    let directory = operation_directory(&project_root, UPLOAD_ROOT, operation_id)?;
    let marker = UploadMarker {
        schema_version: STAGE_SCHEMA_VERSION,
        operation_id: operation_id.to_owned(),
        organization_id: organization_id.to_owned(),
        project_id: project_id.to_owned(),
        project_guid,
        scope_fingerprint: Some(scope_fingerprint.to_owned()),
        base_revision: base_revision.to_owned(),
        manifest_digest,
        manifest,
    };
    ensure_artifact_capacity(
        &project_root,
        &directory,
        expected_artifact_bytes(&marker.manifest, &marker)?,
    )?;
    let marker_bytes = admit_or_write_upload_marker(&project_root, &directory, &marker)?;
    Ok(upload_snapshot(
        project_root,
        directory,
        marker,
        marker_bytes,
    ))
}

pub(crate) fn load_upload_snapshot(
    root: &Path,
    operation_id: &str,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
) -> Result<UploadSnapshot, &'static str> {
    validate_stage_id(operation_id)?;
    if !is_sha256_digest(scope_fingerprint) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let project_root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let directory = operation_directory(&project_root, UPLOAD_ROOT, operation_id)?;
    let bytes = read_private_bounded(
        &project_root,
        &directory.join(UPLOAD_MARKER),
        8 * 1024 * 1024 + 4096,
    )
    .map_err(stage_read_error)?;
    let marker: UploadMarker =
        serde_json::from_slice(&bytes).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if !upload_marker_matches_scope(
        &marker,
        operation_id,
        organization_id,
        project_id,
        project_guid,
        scope_fingerprint,
    ) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    marker
        .manifest
        .require_present_package_lock()
        .map_err(|_| "hub_cloud_sync_package_lock_incompatible")?;
    Ok(upload_snapshot(project_root, directory, marker, bytes))
}

pub(crate) fn find_upload_snapshot(
    root: &Path,
    operation_id: &str,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
) -> Result<Option<UploadSnapshot>, &'static str> {
    validate_stage_id(operation_id)?;
    if !is_sha256_digest(scope_fingerprint) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let project_root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let directory = operation_directory(&project_root, UPLOAD_ROOT, operation_id)?;
    let bytes = match read_private_bounded(
        &project_root,
        &directory.join(UPLOAD_MARKER),
        8 * 1024 * 1024 + 4096,
    ) {
        Ok(bytes) => bytes,
        Err("hub_cloud_sync_stage_missing") => return Ok(None),
        Err(error) => return Err(stage_read_error(error)),
    };
    let marker: UploadMarker =
        serde_json::from_slice(&bytes).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if !upload_marker_matches_scope(
        &marker,
        operation_id,
        organization_id,
        project_id,
        project_guid,
        scope_fingerprint,
    ) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    marker
        .manifest
        .require_present_package_lock()
        .map_err(|_| "hub_cloud_sync_package_lock_incompatible")?;
    Ok(Some(upload_snapshot(
        project_root,
        directory,
        marker,
        bytes,
    )))
}

pub(crate) fn store_upload_blob(
    upload: &UploadSnapshot,
    entry: &FileEntry,
    bytes: &[u8],
) -> Result<(), &'static str> {
    if upload
        .manifest
        .files
        .iter()
        .find(|candidate| candidate.path == entry.path)
        != Some(entry)
        || bytes.len() as u64 != entry.bytes
        || sha256(bytes) != entry.digest
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    stage_blob(&upload.project_root, &upload.directory, &entry.path, bytes)
}

pub(crate) fn read_upload_blob(
    upload: &UploadSnapshot,
    entry: &FileEntry,
) -> Result<Vec<u8>, &'static str> {
    if upload
        .manifest
        .files
        .iter()
        .find(|candidate| candidate.path == entry.path)
        != Some(entry)
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let path = stage_file_path(&upload.directory, &entry.path)?;
    let bytes = read_private_bounded(&upload.project_root, &path, MAX_BLOB_BYTES)
        .map_err(stage_read_error)?;
    if bytes.len() as u64 != entry.bytes || sha256(&bytes) != entry.digest {
        return Err("hub_cloud_sync_stage_invalid");
    }
    Ok(bytes)
}

pub(crate) fn load_staged_snapshot(
    root: &Path,
    stage_id: &str,
    expected_revision: &str,
) -> Result<StagedSnapshot, &'static str> {
    let project_root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let directory = stage_directory(&project_root, stage_id)?;
    let marker_path = directory.join(STAGE_MARKER);
    let bytes = read_private_bounded(&project_root, &marker_path, 8 * 1024 * 1024 + 4096)
        .map_err(stage_read_error)?;
    let marker: StageMarker =
        serde_json::from_slice(&bytes).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if marker.schema_version != STAGE_SCHEMA_VERSION
        || marker.stage_id != stage_id
        || marker.revision != expected_revision
        || marker
            .scope_fingerprint
            .as_deref()
            .is_some_and(|fingerprint| !is_sha256_digest(fingerprint))
        || marker.manifest.canonical_digest().ok().as_deref()
            != Some(marker.manifest_digest.as_str())
        || marker.manifest.require_present_package_lock().is_err()
        || marker
            .manifest
            .files
            .iter()
            .any(|entry| !stage_path_allowed(&entry.path))
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    validate_revision(&marker.revision)?;
    Ok(staged_snapshot(project_root, directory, marker))
}

pub(crate) fn verify_staged_files(snapshot: &StagedSnapshot) -> Result<(), &'static str> {
    for entry in &snapshot.manifest.files {
        let path = stage_file_path(&snapshot.directory, &entry.path)?;
        let bytes = read_private_bounded(&snapshot.project_root, &path, MAX_BLOB_BYTES)
            .map_err(stage_read_error)?;
        if bytes.len() as u64 != entry.bytes || sha256(&bytes) != entry.digest {
            return Err("hub_cloud_sync_stage_invalid");
        }
    }
    Ok(())
}

pub(crate) fn apply_additions(
    root: &Path,
    expected_guid: ProjectGuid,
    snapshot: &StagedSnapshot,
    current_revision: &str,
    _project_lease: &ProjectCloudSyncLease,
) -> Result<ApplyOutcome, &'static str> {
    snapshot
        .manifest
        .require_present_package_lock()
        .map_err(|_| "hub_cloud_sync_package_lock_incompatible")?;
    if snapshot.revision != current_revision {
        return Err("hub_cloud_sync_remote_changed");
    }
    verify_project_guid(root, expected_guid)?;
    verify_staged_files(snapshot)?;

    let mut conflicts = Vec::new();
    let mut conflict_count = 0_usize;
    let mut unchanged = 0_usize;
    let root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    for entry in &snapshot.manifest.files {
        let target = match project_file_path(&root, &entry.path) {
            Ok(target) => target,
            Err("hub_cloud_sync_local_conflict") => {
                record_conflict(&mut conflicts, &mut conflict_count, &entry.path);
                continue;
            }
            Err(error) => return Err(error),
        };
        match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                record_conflict(&mut conflicts, &mut conflict_count, &entry.path);
            }
            Ok(_) => {
                let bytes = read_project_file(&root, &entry.path)?;
                if sha256(&bytes) == entry.digest && bytes.len() as u64 == entry.bytes {
                    unchanged += 1;
                } else {
                    record_conflict(&mut conflicts, &mut conflict_count, &entry.path);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("hub_cloud_sync_project_changed"),
        }
    }
    if conflict_count > 0 {
        write_conflict_artifact(snapshot, &conflicts, conflict_count)?;
        return Ok(ApplyOutcome::Conflict {
            paths: conflicts,
            conflict_count,
        });
    }

    let mut applied = 0_usize;
    for entry in &snapshot.manifest.files {
        let target = project_file_path(&root, &entry.path)?;
        if target.exists() {
            continue;
        }
        ensure_project_parents(&root, &entry.path)?;
        let staged_path = stage_file_path(&snapshot.directory, &entry.path)?;
        let bytes =
            read_private_bounded(&root, &staged_path, MAX_BLOB_BYTES).map_err(stage_read_error)?;
        if bytes.len() as u64 != entry.bytes || sha256(&bytes) != entry.digest {
            return Err("hub_cloud_sync_stage_invalid");
        }
        publish_project_file(&root, &target, &snapshot.stage_id, &entry.path, &bytes)?;
        applied += 1;
    }
    verify_project_guid(&root, expected_guid)?;
    Ok(ApplyOutcome::Applied {
        applied_files: applied,
        unchanged_files: unchanged,
    })
}

fn staged_snapshot(
    project_root: PathBuf,
    directory: PathBuf,
    marker: StageMarker,
) -> StagedSnapshot {
    StagedSnapshot {
        project_root,
        stage_id: marker.stage_id,
        organization_id: marker.organization_id,
        project_id: marker.project_id,
        revision: marker.revision,
        manifest_digest: marker.manifest_digest,
        manifest: marker.manifest,
        directory,
    }
}

fn upload_snapshot(
    project_root: PathBuf,
    directory: PathBuf,
    marker: UploadMarker,
    marker_bytes: Vec<u8>,
) -> UploadSnapshot {
    UploadSnapshot {
        project_root,
        operation_id: marker.operation_id,
        organization_id: marker.organization_id,
        project_id: marker.project_id,
        base_revision: marker.base_revision,
        manifest_digest: marker.manifest_digest,
        manifest: marker.manifest,
        directory,
        marker_bytes,
    }
}

fn upload_marker_matches_scope(
    marker: &UploadMarker,
    operation_id: &str,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
) -> bool {
    marker.schema_version == STAGE_SCHEMA_VERSION
        && marker.operation_id == operation_id
        && marker.organization_id == organization_id
        && marker.project_id == project_id
        && marker.project_guid == project_guid
        && marker.scope_fingerprint.as_deref() == Some(scope_fingerprint)
        && marker.manifest.canonical_digest().ok().as_deref()
            == Some(marker.manifest_digest.as_str())
        && marker
            .manifest
            .files
            .iter()
            .all(|entry| stage_path_allowed(&entry.path))
        && validate_revision(&marker.base_revision).is_ok()
}

fn upload_marker_header_matches_scope(
    header: &UploadMarkerHeader,
    operation_id: &str,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
) -> bool {
    header.schema_version == STAGE_SCHEMA_VERSION
        && header.operation_id == operation_id
        && header.organization_id == organization_id
        && header.project_id == project_id
        && header.project_guid == project_guid
        && header.scope_fingerprint.as_deref() == Some(scope_fingerprint)
        && is_sha256_digest(&header.manifest_digest)
        && validate_revision(&header.base_revision).is_ok()
}

fn read_upload_marker_header(
    project_root: &Path,
    path: &Path,
    budget: &mut ArtifactScanBudget,
) -> Result<UploadMarkerHeader, &'static str> {
    let mut file = open_private_artifact_file(project_root, path)?;
    let metadata = file
        .metadata()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if !metadata.is_file() || metadata.len() > (8 * 1024 * 1024 + 4096) as u64 {
        return Err("hub_cloud_sync_stage_invalid");
    }

    let prefix_limit = metadata.len().min(MAX_UPLOAD_MARKER_HEADER_BYTES as u64);
    budget.visit_read_bytes(prefix_limit)?;
    let mut prefix = Vec::with_capacity(MAX_UPLOAD_MARKER_HEADER_BYTES);
    (&mut file)
        .take(prefix_limit)
        .read_to_end(&mut prefix)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if file
        .metadata()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?
        .len()
        != metadata.len()
    {
        return Err("hub_cloud_sync_stage_invalid");
    }

    // Hub-authored markers serialize the identity header before the manifest. Requiring that
    // stable order lets recovery inspect a small prefix and leave foreign large markers unread.
    let manifest_field = b",\"manifest\":";
    let end = prefix
        .windows(manifest_field.len())
        .position(|window| window == manifest_field)
        .ok_or("hub_cloud_sync_stage_invalid")?;
    let mut header_bytes = prefix[..end].to_vec();
    header_bytes.push(b'}');
    serde_json::from_slice(&header_bytes).map_err(|_| "hub_cloud_sync_stage_invalid")
}

fn operation_directory(
    root: &Path,
    private_root: &[&str],
    operation_id: &str,
) -> Result<PathBuf, &'static str> {
    let root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    validate_stage_id(operation_id)?;
    let mut directory = AnchoredDirectory::open_for_directory_writes(&root)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    for component in private_root
        .iter()
        .copied()
        .chain(std::iter::once(operation_id))
    {
        directory = directory
            .ensure_child_directory(std::ffi::OsStr::new(component))
            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    }
    Ok(directory.path().to_path_buf())
}

pub(crate) fn prune_stale_download_stages(
    project_root: &Path,
    organization_id: &str,
    project_id: &str,
    scope_fingerprint: &str,
    current_revision: &str,
    _project_lease: &ProjectCloudSyncLease,
) -> Result<usize, &'static str> {
    validate_stage_id(organization_id)?;
    validate_stage_id(project_id)?;
    validate_revision(current_revision)?;
    if !is_sha256_digest(scope_fingerprint) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let Some(stage_root) = existing_private_directory(&root, STAGE_ROOT)? else {
        return Ok(0);
    };

    let current_revision = revision_number(current_revision)?;
    let mut budget = ArtifactScanBudget::default();
    let mut removed = 0;
    for child in fs::read_dir(&stage_root).map_err(|_| "hub_cloud_sync_stage_invalid")? {
        budget.visit_entry()?;
        let child = child.map_err(|_| "hub_cloud_sync_stage_invalid")?;
        let directory = child.path();
        let metadata =
            fs::symlink_metadata(&directory).map_err(|_| "hub_cloud_sync_stage_invalid")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        let stage_id = child.file_name().to_string_lossy().into_owned();
        if validate_stage_id(&stage_id).is_err() {
            continue;
        }
        budget.visit_directory()?;
        let marker_path = directory.join(STAGE_MARKER);
        let marker_bytes = match read_private_bounded(&root, &marker_path, 8 * 1024 * 1024 + 4096) {
            Ok(bytes) => bytes,
            Err("hub_cloud_sync_stage_missing") => continue,
            Err(_) => continue,
        };
        let marker: StageMarker = match serde_json::from_slice(&marker_bytes) {
            Ok(marker) => marker,
            Err(_) => continue,
        };
        if marker.schema_version != STAGE_SCHEMA_VERSION
            || marker.stage_id != stage_id
            || marker.organization_id != organization_id
            || marker.project_id != project_id
            || marker.scope_fingerprint.as_deref() != Some(scope_fingerprint)
            || marker.manifest.canonical_digest().ok().as_deref()
                != Some(marker.manifest_digest.as_str())
            || marker
                .manifest
                .files
                .iter()
                .any(|entry| !stage_path_allowed(&entry.path))
        {
            continue;
        }
        let revision = match revision_number(&marker.revision) {
            Ok(revision) => revision,
            Err(_) => continue,
        };
        if revision >= current_revision || has_conflict_artifact(&directory, &mut budget)? {
            continue;
        }
        remove_managed_directory_preserving_marker_with_budget(
            &directory,
            STAGE_MARKER,
            &marker_bytes,
            &mut budget,
        )?;
        discard_operation_directory(&root, PUBLICATION_ROOT, &stage_id)?;
        removed += 1;
    }
    Ok(removed)
}

fn has_conflict_artifact(
    directory: &Path,
    budget: &mut ArtifactScanBudget,
) -> Result<bool, &'static str> {
    for child in fs::read_dir(directory).map_err(|_| "hub_cloud_sync_stage_invalid")? {
        budget.visit_entry()?;
        let child = child.map_err(|_| "hub_cloud_sync_stage_invalid")?;
        let metadata =
            fs::symlink_metadata(child.path()).map_err(|_| "hub_cloud_sync_stage_invalid")?;
        if metadata.file_type().is_symlink() {
            return Err("hub_cloud_sync_stage_invalid");
        }
        if child
            .file_name()
            .to_string_lossy()
            .starts_with(CONFLICT_ARTIFACT_PREFIX)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn existing_private_directory(
    project_root: &Path,
    components: &[&str],
) -> Result<Option<PathBuf>, &'static str> {
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let mut directory =
        AnchoredDirectory::open(&root).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    for component in components {
        match directory.open_existing_private_child(std::ffi::OsStr::new(component)) {
            Ok(child) => directory = child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        }
    }
    Ok(Some(directory.path().to_path_buf()))
}

fn validate_existing_directory_chain(
    project_root: &Path,
    directory: &Path,
) -> Result<(), &'static str> {
    use std::path::Component;

    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let relative = directory
        .strip_prefix(&root)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    let mut current = root;
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err("hub_cloud_sync_stage_invalid");
        };
        current.push(name);
        let metadata =
            fs::symlink_metadata(&current).map_err(|_| "hub_cloud_sync_stage_invalid")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("hub_cloud_sync_stage_invalid");
        }
    }
    Ok(())
}

pub(crate) fn discard_upload_snapshot(
    project_root: &Path,
    operation_id: &str,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
    _project_lease: &ProjectCloudSyncLease,
) -> Result<(), &'static str> {
    let snapshot = load_upload_snapshot(
        project_root,
        operation_id,
        organization_id,
        project_id,
        project_guid,
        scope_fingerprint,
    )?;
    let directory = operation_directory(project_root, UPLOAD_ROOT, operation_id)?;
    discard_upload_directory(&directory, &snapshot.marker_bytes)
}

pub(crate) fn prune_unadmitted_upload_snapshots(
    project_root: &Path,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
    journal_operation_ids: &[String],
    project_lease: &ProjectCloudSyncLease,
) -> Result<usize, &'static str> {
    prune_unadmitted_upload_snapshots_with_budget(
        project_root,
        organization_id,
        project_id,
        project_guid,
        scope_fingerprint,
        journal_operation_ids,
        &mut ArtifactScanBudget::default(),
        project_lease,
    )
}

pub(crate) fn prune_terminal_upload_snapshots(
    project_root: &Path,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
    terminal_operations: &[CloudCommitCleanupRecord],
    project_lease: &ProjectCloudSyncLease,
) -> Result<usize, &'static str> {
    let mut budget = ArtifactScanBudget::default();
    prune_terminal_upload_snapshots_with_budget(
        project_root,
        organization_id,
        project_id,
        project_guid,
        scope_fingerprint,
        terminal_operations,
        &mut budget,
        project_lease,
    )
}

pub(crate) fn prune_upload_snapshots(
    project_root: &Path,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
    journal_operation_ids: &[String],
    terminal_operations: &[CloudCommitCleanupRecord],
    project_lease: &ProjectCloudSyncLease,
) -> Result<usize, &'static str> {
    let mut budget = ArtifactScanBudget::default();
    let unadmitted = prune_unadmitted_upload_snapshots_with_budget(
        project_root,
        organization_id,
        project_id,
        project_guid,
        scope_fingerprint,
        journal_operation_ids,
        &mut budget,
        project_lease,
    )?;
    let terminal = prune_terminal_upload_snapshots_with_budget(
        project_root,
        organization_id,
        project_id,
        project_guid,
        scope_fingerprint,
        terminal_operations,
        &mut budget,
        project_lease,
    )?;
    unadmitted
        .checked_add(terminal)
        .ok_or("hub_cloud_sync_stage_invalid")
}

fn prune_terminal_upload_snapshots_with_budget(
    project_root: &Path,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
    terminal_operations: &[CloudCommitCleanupRecord],
    budget: &mut ArtifactScanBudget,
    _project_lease: &ProjectCloudSyncLease,
) -> Result<usize, &'static str> {
    validate_stage_id(organization_id)?;
    validate_stage_id(project_id)?;
    if !is_sha256_digest(scope_fingerprint) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let mut removed: usize = 0;
    for operation in terminal_operations {
        if !operation.status.is_terminal()
            || operation.organization_id != organization_id
            || operation.project_id != project_id
            || validate_stage_id(&operation.operation_id).is_err()
        {
            continue;
        }
        let mut marker_components = UPLOAD_ROOT.to_vec();
        marker_components.push(&operation.operation_id);
        let Some(directory) = existing_private_directory(&root, &marker_components)? else {
            continue;
        };
        budget.visit_directory()?;
        let marker_path = directory.join(UPLOAD_MARKER);
        let header = match read_upload_marker_header(&root, &marker_path, budget) {
            Ok(header) => header,
            Err("hub_cloud_sync_stage_missing") => {
                // A terminal operation can reach this markerless state only after an
                // earlier cleanup removed every child and then stopped before removing
                // the operation directory. Remove it only if the anchored probe confirms
                // it is empty; markerless nonempty data has no recoverable identity.
                budget.visit_entry()?;
                if AnchoredDirectory::remove_empty_directory_path_if_empty(&directory)
                    .map_err(|_| "hub_cloud_sync_cleanup_pending")?
                {
                    removed = removed.saturating_add(1);
                }
                continue;
            }
            Err("hub_cloud_sync_storage_quota_exceeded") => {
                return Err("hub_cloud_sync_storage_quota_exceeded");
            }
            Err(_) => continue,
        };
        if !upload_marker_header_matches_scope(
            &header,
            &operation.operation_id,
            organization_id,
            project_id,
            project_guid,
            scope_fingerprint,
        ) {
            continue;
        }
        let marker_bytes = match read_private_bounded_with_budget(
            &root,
            &marker_path,
            8 * 1024 * 1024 + 4096,
            budget,
        ) {
            Ok(bytes) => bytes,
            Err("hub_cloud_sync_storage_quota_exceeded") => {
                return Err("hub_cloud_sync_storage_quota_exceeded");
            }
            Err(_) => continue,
        };
        let marker: UploadMarker = match serde_json::from_slice(&marker_bytes) {
            Ok(marker) => marker,
            Err(_) => continue,
        };
        if !upload_marker_matches_scope(
            &marker,
            &operation.operation_id,
            organization_id,
            project_id,
            project_guid,
            scope_fingerprint,
        ) {
            continue;
        }
        discard_upload_directory_with_budget(&directory, Some(&marker_bytes), budget)?;
        removed = removed.saturating_add(1);
    }
    Ok(removed)
}

fn prune_unadmitted_upload_snapshots_with_budget(
    project_root: &Path,
    organization_id: &str,
    project_id: &str,
    project_guid: ProjectGuid,
    scope_fingerprint: &str,
    journal_operation_ids: &[String],
    budget: &mut ArtifactScanBudget,
    _project_lease: &ProjectCloudSyncLease,
) -> Result<usize, &'static str> {
    validate_stage_id(organization_id)?;
    validate_stage_id(project_id)?;
    if !is_sha256_digest(scope_fingerprint) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let Some(upload_root) = existing_private_directory(&root, UPLOAD_ROOT)? else {
        return Ok(0);
    };

    budget.visit_directory()?;
    let mut removed: usize = 0;
    for child in fs::read_dir(&upload_root).map_err(|_| "hub_cloud_sync_stage_invalid")? {
        budget.visit_entry()?;
        let child = child.map_err(|_| "hub_cloud_sync_stage_invalid")?;
        let directory = child.path();
        let metadata =
            fs::symlink_metadata(&directory).map_err(|_| "hub_cloud_sync_stage_invalid")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        let operation_id = child.file_name().to_string_lossy().into_owned();
        if validate_stage_id(&operation_id).is_err() {
            continue;
        }
        budget.visit_directory()?;
        if journal_operation_ids
            .iter()
            .any(|journal_id| journal_id == &operation_id)
        {
            // The journal owns every known identity, especially Unknown receipts.
            continue;
        }

        let marker_path = directory.join(UPLOAD_MARKER);
        let header = match read_upload_marker_header(&root, &marker_path, &mut budget) {
            Ok(header) => header,
            Err("hub_cloud_sync_stage_missing") => {
                if AnchoredDirectory::remove_empty_directory_path_if_empty(&directory)
                    .map_err(|_| "hub_cloud_sync_cleanup_pending")?
                {
                    removed = removed.saturating_add(1);
                } else {
                    budget.visit_entry()?;
                }
                continue;
            }
            Err("hub_cloud_sync_storage_quota_exceeded") => {
                return Err("hub_cloud_sync_storage_quota_exceeded");
            }
            // A corrupt or unreadable marker does not establish which account owns the bytes.
            Err(_) => continue,
        };
        if !upload_marker_header_matches_scope(
            &header,
            &operation_id,
            organization_id,
            project_id,
            project_guid,
            scope_fingerprint,
        ) {
            continue;
        }
        let marker_bytes = match read_private_bounded_with_budget(
            &root,
            &marker_path,
            8 * 1024 * 1024 + 4096,
            budget,
        ) {
            Ok(bytes) => bytes,
            Err("hub_cloud_sync_storage_quota_exceeded") => {
                return Err("hub_cloud_sync_storage_quota_exceeded");
            }
            Err(_) => continue,
        };
        let marker: UploadMarker = match serde_json::from_slice(&marker_bytes) {
            Ok(marker) => marker,
            Err(_) => continue,
        };
        if !upload_marker_matches_scope(
            &marker,
            &operation_id,
            organization_id,
            project_id,
            project_guid,
            scope_fingerprint,
        ) {
            // Project and account mismatches stay untouched because the marker cannot
            // authorize them.
            continue;
        }

        discard_upload_directory_with_budget(&directory, Some(&marker_bytes), &mut budget)?;
        removed = removed.saturating_add(1);
    }
    Ok(removed)
}

pub(crate) fn discard_bound_download_stage(
    project_root: &Path,
    stage_id: &str,
    organization_id: &str,
    project_id: &str,
    expected_revision: &str,
    scope_fingerprint: &str,
    _project_lease: &ProjectCloudSyncLease,
) -> Result<(), &'static str> {
    validate_stage_id(stage_id)?;
    validate_stage_id(organization_id)?;
    validate_stage_id(project_id)?;
    validate_revision(expected_revision)?;
    if !is_sha256_digest(scope_fingerprint) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let directory = stage_directory(&root, stage_id)?;
    let marker_path = directory.join(STAGE_MARKER);
    let bytes = match read_private_bounded(&root, &marker_path, 8 * 1024 * 1024 + 4096) {
        Ok(bytes) => bytes,
        Err("hub_cloud_sync_stage_missing") => match fs::read_dir(&directory) {
            Ok(mut entries) => {
                if entries.next().is_none() {
                    AnchoredDirectory::remove_empty_directory_path(&directory)
                        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                    discard_operation_directory(&root, PUBLICATION_ROOT, stage_id)?;
                    return Ok(());
                }
                return Err("hub_cloud_sync_stage_invalid");
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                discard_operation_directory(&root, PUBLICATION_ROOT, stage_id)?;
                return Ok(());
            }
            _ => return Err("hub_cloud_sync_stage_invalid"),
        },
        Err(error) => return Err(error),
    };
    let marker: StageMarker =
        serde_json::from_slice(&bytes).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if marker.schema_version != STAGE_SCHEMA_VERSION
        || marker.stage_id != stage_id
        || marker.organization_id != organization_id
        || marker.project_id != project_id
        || marker.revision != expected_revision
        || marker.scope_fingerprint.as_deref() != Some(scope_fingerprint)
        || marker.manifest.canonical_digest().ok().as_deref()
            != Some(marker.manifest_digest.as_str())
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    remove_managed_directory_preserving_marker(&directory, STAGE_MARKER, &bytes)?;
    discard_operation_directory(&root, PUBLICATION_ROOT, stage_id)
}

fn discard_operation_directory(
    project_root: &Path,
    private_root: &[&str],
    operation_id: &str,
) -> Result<(), &'static str> {
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let mut directory = root.clone();
    for component in private_root.iter().chain(std::iter::once(&operation_id)) {
        directory.push(component);
        match fs::symlink_metadata(&directory) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err("hub_cloud_sync_stage_invalid");
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        }
    }
    if !directory.starts_with(&root) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    remove_managed_directory(&directory)
}

fn discard_upload_directory(directory: &Path, expected_marker: &[u8]) -> Result<(), &'static str> {
    let mut budget = ArtifactScanBudget::default();
    discard_upload_directory_with_budget(directory, Some(expected_marker), &mut budget)
}

fn discard_upload_directory_with_budget(
    directory: &Path,
    expected_marker: Option<&[u8]>,
    budget: &mut ArtifactScanBudget,
) -> Result<(), &'static str> {
    let (parent, name) = split_managed_path(directory, "hub_cloud_sync_cleanup_pending")?;
    let anchor = AnchoredDirectory::open(parent).map_err(map_cleanup_tree_error)?;
    let usage = anchor
        .remove_tree_preserving_marker(
            name,
            std::ffi::OsStr::new(UPLOAD_MARKER),
            expected_marker,
            remaining_tree_limits(budget),
        )
        .map_err(map_cleanup_tree_error)?;
    charge_tree_usage(budget, usage)
}

fn remove_managed_directory(directory: &Path) -> Result<(), &'static str> {
    let mut budget = ArtifactScanBudget::default();
    remove_managed_directory_with_budget(directory, &mut budget)
}

fn remove_managed_directory_with_budget(
    directory: &Path,
    budget: &mut ArtifactScanBudget,
) -> Result<(), &'static str> {
    let (parent, name) = split_managed_path(directory, "hub_cloud_sync_stage_invalid")?;
    let anchor = AnchoredDirectory::open(parent).map_err(map_stage_tree_error)?;
    let usage = anchor
        .remove_tree(name, remaining_tree_limits(budget))
        .map_err(map_stage_tree_error)?;
    charge_tree_usage(budget, usage)
}

fn remove_managed_directory_preserving_marker(
    directory: &Path,
    marker: &str,
    expected_marker: &[u8],
) -> Result<(), &'static str> {
    let mut budget = ArtifactScanBudget::default();
    remove_managed_directory_preserving_marker_with_budget(
        directory,
        marker,
        expected_marker,
        &mut budget,
    )
}

fn remove_managed_directory_preserving_marker_with_budget(
    directory: &Path,
    marker: &str,
    expected_marker: &[u8],
    budget: &mut ArtifactScanBudget,
) -> Result<(), &'static str> {
    let (parent, name) = split_managed_path(directory, "hub_cloud_sync_stage_invalid")?;
    let anchor = AnchoredDirectory::open(parent).map_err(map_stage_tree_error)?;
    let usage = anchor
        .remove_tree_preserving_marker(
            name,
            std::ffi::OsStr::new(marker),
            Some(expected_marker),
            remaining_tree_limits(budget),
        )
        .map_err(map_stage_tree_error)?;
    charge_tree_usage(budget, usage)
}

fn split_managed_path<'a>(
    path: &'a Path,
    invalid: &'static str,
) -> Result<(&'a Path, &'a std::ffi::OsStr), &'static str> {
    let parent = path.parent().ok_or(invalid)?;
    let name = path.file_name().ok_or(invalid)?;
    Ok((parent, name))
}

fn remaining_tree_limits(budget: &ArtifactScanBudget) -> TreeRemovalLimits {
    TreeRemovalLimits {
        entries: MAX_ARTIFACT_ENTRIES.saturating_sub(budget.entries),
        directories: MAX_ARTIFACT_DIRECTORIES.saturating_sub(budget.directories),
    }
}

fn charge_tree_usage(
    budget: &mut ArtifactScanBudget,
    usage: TreeRemovalUsage,
) -> Result<(), &'static str> {
    budget.entries = budget
        .entries
        .checked_add(usage.entries)
        .filter(|entries| *entries <= MAX_ARTIFACT_ENTRIES)
        .ok_or("hub_cloud_sync_storage_quota_exceeded")?;
    budget.directories = budget
        .directories
        .checked_add(usage.directories)
        .filter(|directories| *directories <= MAX_ARTIFACT_DIRECTORIES)
        .ok_or("hub_cloud_sync_storage_quota_exceeded")?;
    Ok(())
}

fn map_stage_tree_error(error: std::io::Error) -> &'static str {
    if error.kind() == std::io::ErrorKind::InvalidData {
        "hub_cloud_sync_storage_quota_exceeded"
    } else {
        "hub_cloud_sync_stage_invalid"
    }
}

fn map_cleanup_tree_error(error: std::io::Error) -> &'static str {
    if error.kind() == std::io::ErrorKind::InvalidData {
        "hub_cloud_sync_storage_quota_exceeded"
    } else if error.kind() == std::io::ErrorKind::InvalidInput {
        "hub_cloud_sync_stage_invalid"
    } else {
        "hub_cloud_sync_cleanup_pending"
    }
}

fn expected_artifact_bytes<T: Serialize>(
    manifest: &Manifest,
    marker: &T,
) -> Result<u64, &'static str> {
    let content = manifest.files.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.bytes)
            .ok_or("hub_cloud_sync_stage_invalid")
    })?;
    content
        .checked_add(
            serde_json::to_vec(marker)
                .map_err(|_| "hub_cloud_sync_stage_invalid")?
                .len() as u64,
        )
        .ok_or("hub_cloud_sync_stage_invalid")
}

fn ensure_artifact_capacity(
    project_root: &Path,
    current_directory: &Path,
    planned_bytes: u64,
) -> Result<(), &'static str> {
    ensure_artifact_capacity_with_limit(
        project_root,
        current_directory,
        planned_bytes,
        MAX_CLOUD_ARTIFACT_BYTES,
    )
}

fn ensure_artifact_capacity_with_limit(
    project_root: &Path,
    current_directory: &Path,
    planned_bytes: u64,
    limit: u64,
) -> Result<(), &'static str> {
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    if !current_directory.starts_with(&root) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    validate_existing_directory_chain(&root, current_directory)?;
    let used = cloud_artifact_usage(&root)?;
    let mut current_budget = ArtifactScanBudget::default();
    let current_bytes = measure_optional_directory(current_directory, &mut current_budget)?;
    let projected = used
        .checked_sub(current_bytes)
        .and_then(|remaining| remaining.checked_add(current_bytes.max(planned_bytes)))
        .ok_or("hub_cloud_sync_stage_invalid")?;
    if projected > limit {
        return Err("hub_cloud_sync_storage_quota_exceeded");
    }
    Ok(())
}

fn ensure_artifact_additional_capacity(
    project_root: &Path,
    additional_bytes: u64,
) -> Result<(), &'static str> {
    ensure_artifact_additional_capacity_with_limit(
        project_root,
        additional_bytes,
        MAX_CLOUD_ARTIFACT_BYTES,
    )
}

fn ensure_artifact_additional_capacity_with_limit(
    project_root: &Path,
    additional_bytes: u64,
    limit: u64,
) -> Result<(), &'static str> {
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let projected = cloud_artifact_usage(&root)?
        .checked_add(additional_bytes)
        .ok_or("hub_cloud_sync_stage_invalid")?;
    if projected > limit {
        return Err("hub_cloud_sync_storage_quota_exceeded");
    }
    Ok(())
}

fn cloud_artifact_usage(project_root: &Path) -> Result<u64, &'static str> {
    let mut budget = ArtifactScanBudget::default();
    let mut used = 0_u64;
    for components in [UPLOAD_ROOT, STAGE_ROOT, PUBLICATION_ROOT] {
        if let Some(directory) = existing_private_directory(project_root, components)? {
            used = used
                .checked_add(measure_directory(&directory, &mut budget, 0)?)
                .ok_or("hub_cloud_sync_stage_invalid")?;
        }
    }
    Ok(used)
}

#[derive(Default)]
struct ArtifactScanBudget {
    entries: usize,
    directories: usize,
    read_bytes: u64,
}

impl ArtifactScanBudget {
    fn visit_entry(&mut self) -> Result<(), &'static str> {
        self.entries = self
            .entries
            .checked_add(1)
            .ok_or("hub_cloud_sync_storage_quota_exceeded")?;
        (self.entries <= MAX_ARTIFACT_ENTRIES)
            .then_some(())
            .ok_or("hub_cloud_sync_storage_quota_exceeded")
    }

    fn visit_directory(&mut self) -> Result<(), &'static str> {
        self.directories = self
            .directories
            .checked_add(1)
            .ok_or("hub_cloud_sync_storage_quota_exceeded")?;
        (self.directories <= MAX_ARTIFACT_DIRECTORIES)
            .then_some(())
            .ok_or("hub_cloud_sync_storage_quota_exceeded")
    }

    fn visit_read_bytes(&mut self, bytes: u64) -> Result<(), &'static str> {
        let total = self
            .read_bytes
            .checked_add(bytes)
            .ok_or("hub_cloud_sync_storage_quota_exceeded")?;
        if total > MAX_CLOUD_UPLOAD_RECOVERY_READ_BYTES {
            return Err("hub_cloud_sync_storage_quota_exceeded");
        }
        self.read_bytes = total;
        Ok(())
    }
}

fn measure_optional_directory(
    directory: &Path,
    budget: &mut ArtifactScanBudget,
) -> Result<u64, &'static str> {
    match fs::symlink_metadata(directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(_) => Err("hub_cloud_sync_stage_invalid"),
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err("hub_cloud_sync_stage_invalid")
        }
        Ok(_) => measure_directory(directory, budget, 0),
    }
}

fn measure_directory(
    directory: &Path,
    budget: &mut ArtifactScanBudget,
    depth: usize,
) -> Result<u64, &'static str> {
    if depth > MAX_CLOUD_DIRECTORY_DEPTH {
        return Err("hub_cloud_sync_storage_quota_exceeded");
    }
    budget.visit_directory()?;
    let mut bytes = 0_u64;
    for child in fs::read_dir(directory).map_err(|_| "hub_cloud_sync_stage_invalid")? {
        budget.visit_entry()?;
        let child = child.map_err(|_| "hub_cloud_sync_stage_invalid")?;
        let metadata =
            fs::symlink_metadata(child.path()).map_err(|_| "hub_cloud_sync_stage_invalid")?;
        if metadata.file_type().is_symlink() {
            return Err("hub_cloud_sync_stage_invalid");
        }
        if metadata.is_dir() {
            bytes = bytes
                .checked_add(measure_directory(&child.path(), budget, depth + 1)?)
                .ok_or("hub_cloud_sync_stage_invalid")?;
        } else if metadata.is_file() {
            bytes = bytes
                .checked_add(metadata.len())
                .ok_or("hub_cloud_sync_stage_invalid")?;
        } else {
            return Err("hub_cloud_sync_stage_invalid");
        }
    }
    Ok(bytes)
}

fn publication_record_name(bytes: &[u8]) -> String {
    format!(".cloud-sync-publication-{}.json", sha256(bytes))
}

fn is_publication_record_name(name: &str) -> bool {
    let Some(digest) = name
        .strip_prefix(".cloud-sync-publication-")
        .and_then(|name| name.strip_suffix(".json"))
    else {
        return false;
    };
    is_sha256_digest(digest)
}

fn temporary_name_matches(name: &str, stage_id: &str, digest: &str) -> bool {
    let prefix = format!(
        "{PUBLISH_TEMP_PREFIX}{}-{digest}-",
        sha256(stage_id.as_bytes())
    );
    name.strip_prefix(&prefix)
        .is_some_and(|suffix| is_publish_temporary_suffix(suffix))
}

fn is_publish_temporary_suffix(suffix: &str) -> bool {
    let Some(stem) = suffix.strip_suffix(".tmp") else {
        return false;
    };
    let mut parts = stem.split('-');
    let (Some(process), Some(timestamp), Some(sequence), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    [process, timestamp, sequence]
        .iter()
        .all(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
}

fn is_publish_temporary_name(name: &str) -> bool {
    let Some(stem) = name
        .strip_prefix(PUBLISH_TEMP_PREFIX)
        .and_then(|name| name.strip_suffix(".tmp"))
    else {
        return false;
    };
    let mut parts = stem.split('-');
    let (Some(stage), Some(content), Some(process), Some(timestamp), Some(sequence), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return false;
    };
    [stage, content]
        .iter()
        .all(|digest| digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
        && [process, timestamp, sequence]
            .iter()
            .all(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
}

fn admit_or_write_marker(
    project_root: &Path,
    directory: &Path,
    marker: &StageMarker,
) -> Result<(), &'static str> {
    let target = directory.join(STAGE_MARKER);
    let bytes = serde_json::to_vec(marker).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if bytes.len() > 8 * 1024 * 1024 + 4096 {
        return Err("hub_cloud_sync_stage_invalid");
    }
    match read_private_bounded(project_root, &target, 8 * 1024 * 1024 + 4096) {
        Ok(existing) if existing == bytes => return Ok(()),
        Ok(_) => return Err("hub_cloud_sync_stage_invalid"),
        Err("hub_cloud_sync_stage_missing") => {}
        Err(error) => return Err(error),
    }
    publish_new_file(directory, &target, &marker.stage_id, &bytes)?;
    Ok(())
}

fn admit_or_write_upload_marker(
    project_root: &Path,
    directory: &Path,
    marker: &UploadMarker,
) -> Result<Vec<u8>, &'static str> {
    let target = directory.join(UPLOAD_MARKER);
    let bytes = serde_json::to_vec(marker).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if bytes.len() > 8 * 1024 * 1024 + 4096 {
        return Err("hub_cloud_sync_stage_invalid");
    }
    match read_private_bounded(project_root, &target, 8 * 1024 * 1024 + 4096) {
        Ok(existing) if existing == bytes => return Ok(existing),
        Ok(_) => return Err("hub_cloud_sync_stage_invalid"),
        Err("hub_cloud_sync_stage_missing") => {}
        Err(error) => return Err(error),
    }
    publish_new_file(directory, &target, &marker.operation_id, &bytes)?;
    Ok(bytes)
}

fn stage_blob(
    project_root: &Path,
    directory: &Path,
    relative: &str,
    bytes: &[u8],
) -> Result<(), &'static str> {
    let target = stage_file_path(directory, relative)?;
    match read_private_bounded(project_root, &target, MAX_BLOB_BYTES) {
        Ok(existing) if existing == bytes => return Ok(()),
        Ok(_) => return Err("hub_cloud_sync_stage_invalid"),
        Err("hub_cloud_sync_stage_missing") => {}
        Err(error) => return Err(error),
    }
    ensure_stage_parents(directory, relative)?;
    match publish_new_file(directory, &target, "stage", bytes) {
        Ok(()) => Ok(()),
        Err("hub_cloud_sync_stage_exists") => {
            let existing = read_private_bounded(project_root, &target, MAX_BLOB_BYTES)
                .map_err(stage_read_error)?;
            (existing == bytes)
                .then_some(())
                .ok_or("hub_cloud_sync_stage_invalid")
        }
        Err(error) => Err(error),
    }
}

fn publish_new_file(
    root: &Path,
    target: &Path,
    stage_id: &str,
    bytes: &[u8],
) -> Result<(), &'static str> {
    publish_new_file_with_hook(root, target, stage_id, bytes, || {})
}

fn publish_new_file_with_hook<F>(
    root: &Path,
    target: &Path,
    stage_id: &str,
    bytes: &[u8],
    before_publish: F,
) -> Result<(), &'static str>
where
    F: FnOnce(),
{
    publish_new_file_with_hooks(root, target, stage_id, bytes, before_publish, || {})
}

fn publish_new_file_with_hooks<F, G>(
    root: &Path,
    target: &Path,
    stage_id: &str,
    bytes: &[u8],
    before_publish: F,
    after_identity_check: G,
) -> Result<(), &'static str>
where
    F: FnOnce(),
    G: FnOnce(),
{
    let (parent, target) = contained_publish_target(root, target)?;
    let target_name = target.file_name().ok_or("hub_cloud_sync_stage_invalid")?;
    let parent_anchor = AnchoredDirectory::open_for_file_writes(&parent)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    before_publish();
    if !parent_anchor
        .path_still_matches()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let digest = sha256(bytes);
    let mut after_identity_check = Some(after_identity_check);
    for _ in 0..128 {
        let path = next_publish_temporary_path(&parent, stage_id, &digest)?;
        let name = path
            .file_name()
            .ok_or("hub_cloud_sync_stage_invalid")?
            .to_os_string();
        let temporary_parent = parent_anchor
            .try_clone()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
        let file = match parent_anchor.create_new_file(&name) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        };
        let mut temporary = PublishTemporary {
            parent: temporary_parent,
            name,
            file: Some(file),
            receipt: None,
        };
        // On Unix an opened parent remains usable after a rename. Check again after creating
        // the name but before any project bytes can be written through that handle.
        if !parent_anchor
            .path_still_matches()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
        {
            return Err("hub_cloud_sync_stage_invalid");
        }
        let file = temporary
            .file
            .as_mut()
            .ok_or("hub_cloud_sync_stage_invalid")?;
        if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
            return Err("hub_cloud_sync_stage_invalid");
        }
        temporary.file.take();
        if !parent_anchor
            .path_still_matches()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
        {
            return Err("hub_cloud_sync_stage_invalid");
        }
        if let Some(hook) = after_identity_check.take() {
            hook();
        }
        match parent_anchor.hard_link(&temporary.name, target_name) {
            Ok(()) => {
                // Unix keeps an opened directory usable if another process renames it.
                // Recheck after the link and retract it if the directory left its path.
                match parent_anchor.path_still_matches() {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        parent_anchor
                            .remove_file(target_name)
                            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                        return Err("hub_cloud_sync_stage_invalid");
                    }
                }
                parent_anchor
                    .remove_file(&temporary.name)
                    .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err("hub_cloud_sync_stage_exists");
            }
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        }
    }
    Err("hub_cloud_sync_stage_invalid")
}

fn publish_project_file(
    project_root: &Path,
    target: &Path,
    stage_id: &str,
    relative_path: &str,
    bytes: &[u8],
) -> Result<(), &'static str> {
    publish_project_file_with_hook(project_root, target, stage_id, relative_path, bytes, || {})
}

fn publish_project_file_with_hook<F>(
    project_root: &Path,
    target: &Path,
    stage_id: &str,
    relative_path: &str,
    bytes: &[u8],
    before_publish: F,
) -> Result<(), &'static str>
where
    F: FnOnce(),
{
    publish_project_file_with_hooks(
        project_root,
        target,
        stage_id,
        relative_path,
        bytes,
        before_publish,
        || {},
        || {},
    )
}

fn publish_project_file_with_hooks<F, G>(
    project_root: &Path,
    target: &Path,
    stage_id: &str,
    relative_path: &str,
    bytes: &[u8],
    before_publish: F,
    after_identity_check: G,
    after_link: impl FnOnce(),
) -> Result<(), &'static str>
where
    F: FnOnce(),
    G: FnOnce(),
{
    let project_root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    let (parent, target) = contained_publish_target(&project_root, target)?;
    let parent_anchor = AnchoredDirectory::open_for_file_writes(&parent)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if !parent_anchor
        .path_still_matches()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let target_name = target.file_name().ok_or("hub_cloud_sync_stage_invalid")?;
    let digest = sha256(bytes);
    let mut before_publish = Some(before_publish);
    let mut after_identity_check = Some(after_identity_check);
    let mut after_link = Some(after_link);

    for _ in 0..128 {
        let path = next_publish_temporary_path(&parent, stage_id, &digest)?;
        let temporary_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("hub_cloud_sync_stage_invalid")?
            .to_owned();
        let artifact = PublicationArtifact {
            schema_version: PUBLICATION_SCHEMA_VERSION,
            stage_id: stage_id.to_owned(),
            relative_path: relative_path.to_owned(),
            temporary_name,
            digest: digest.clone(),
            bytes: bytes.len() as u64,
        };
        let record_path = match persist_publication_artifact(&project_root, &artifact) {
            Ok(path) => path,
            Err("hub_cloud_sync_stage_exists") => continue,
            Err(error) => return Err(error),
        };

        if let Some(hook) = before_publish.take() {
            hook();
        }
        match parent_anchor.path_still_matches() {
            Ok(true) => {}
            Ok(false) | Err(_) => {
                AnchoredDirectory::remove_file_path(&record_path)
                    .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                return Err("hub_cloud_sync_stage_invalid");
            }
        }
        if let Some(hook) = after_identity_check.take() {
            hook();
        }

        let name = path
            .file_name()
            .ok_or("hub_cloud_sync_stage_invalid")?
            .to_os_string();
        let temporary_parent = match parent_anchor.try_clone() {
            Ok(parent) => parent,
            Err(_) => {
                let _ = AnchoredDirectory::remove_file_path(&record_path);
                return Err("hub_cloud_sync_stage_invalid");
            }
        };
        let file = match parent_anchor.create_new_project_file(&name) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                AnchoredDirectory::remove_file_path(&record_path)
                    .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                continue;
            }
            Err(_) => {
                let _ = AnchoredDirectory::remove_file_path(&record_path);
                return Err("hub_cloud_sync_stage_invalid");
            }
        };
        let mut temporary = PublishTemporary {
            parent: temporary_parent,
            name,
            file: Some(file),
            receipt: Some(record_path),
        };
        // A parent moved after the earlier identity check can still accept a new file through
        // its open handle. Reject that state before writing downloaded project bytes.
        if !parent_anchor
            .path_still_matches()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
        {
            return Err("hub_cloud_sync_stage_invalid");
        }
        let file = temporary
            .file
            .as_mut()
            .ok_or("hub_cloud_sync_stage_invalid")?;
        if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
            return Err("hub_cloud_sync_stage_invalid");
        }
        temporary.file.take();
        if !parent_anchor
            .path_still_matches()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
        {
            return Err("hub_cloud_sync_stage_invalid");
        }
        match parent_anchor.hard_link(&temporary.name, target_name) {
            Ok(()) => {
                if let Some(hook) = after_link.take() {
                    hook();
                }
                // Unix keeps an opened directory usable if another process renames it.
                // Recheck after the link and retract it if the directory left its path.
                match parent_anchor.path_still_matches() {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        parent_anchor
                            .remove_file(target_name)
                            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                        temporary
                            .finish_after_publication()
                            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                        return Err("hub_cloud_sync_stage_invalid");
                    }
                }
                temporary
                    .finish_after_publication()
                    .map_err(|_| "hub_cloud_sync_stage_invalid")?;
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err("hub_cloud_sync_stage_exists");
            }
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        }
    }
    Err("hub_cloud_sync_stage_invalid")
}

fn persist_publication_artifact(
    project_root: &Path,
    artifact: &PublicationArtifact,
) -> Result<PathBuf, &'static str> {
    let record_bytes = serde_json::to_vec(artifact).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    ensure_artifact_additional_capacity(project_root, record_bytes.len() as u64)?;
    let directory = operation_directory(project_root, PUBLICATION_ROOT, &artifact.stage_id)?;
    let target = directory.join(publication_record_name(&record_bytes));
    publish_new_file(&directory, &target, &artifact.stage_id, &record_bytes)?;
    Ok(target)
}

fn contained_publish_target(
    root: &Path,
    target: &Path,
) -> Result<(PathBuf, PathBuf), &'static str> {
    use std::path::Component;

    let root = root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    let relative_parent = target
        .parent()
        .and_then(|parent| parent.strip_prefix(&root).ok())
        .ok_or("hub_cloud_sync_stage_invalid")?;
    let mut parent = root;
    for component in relative_parent.components() {
        let Component::Normal(name) = component else {
            return Err("hub_cloud_sync_stage_invalid");
        };
        parent.push(name);
        let metadata = fs::symlink_metadata(&parent).map_err(|_| "hub_cloud_sync_stage_invalid")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("hub_cloud_sync_stage_invalid");
        }
    }
    let name = target.file_name().ok_or("hub_cloud_sync_stage_invalid")?;
    Ok((parent.clone(), parent.join(name)))
}

struct PublishTemporary {
    parent: AnchoredDirectory,
    name: std::ffi::OsString,
    file: Option<File>,
    receipt: Option<PathBuf>,
}

impl PublishTemporary {
    fn finish_after_publication(&mut self) -> std::io::Result<()> {
        self.finish_after_publication_with(AnchoredDirectory::remove_file_path)
    }

    fn finish_after_publication_with(
        &mut self,
        remove_receipt: impl FnOnce(&Path) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        self.file.take();
        match self.parent.remove_file(&self.name) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if let Some(receipt) = self.receipt.take() {
            remove_receipt(&receipt)?;
        }
        Ok(())
    }
}

impl Drop for PublishTemporary {
    fn drop(&mut self) {
        self.file.take();
        let temporary_removed = match self.parent.remove_file(&self.name) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => false,
        };
        if temporary_removed {
            if let Some(receipt) = self.receipt.take() {
                let _ = AnchoredDirectory::remove_file_path(&receipt);
            }
        }
    }
}

fn next_publish_temporary_path(
    parent: &Path,
    stage_id: &str,
    content_digest: &str,
) -> Result<PathBuf, &'static str> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?
        .as_nanos();
    let sequence = NEXT_PUBLISH_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    Ok(parent.join(format!(
        "{PUBLISH_TEMP_PREFIX}{}-{}-{}-{timestamp}-{sequence}.tmp",
        sha256(stage_id.as_bytes()),
        content_digest,
        std::process::id(),
    )))
}

fn ensure_stage_parents(directory: &Path, relative: &str) -> Result<(), &'static str> {
    ensure_relative_parent_directories(directory, relative, true)
}

fn ensure_project_parents(root: &Path, relative: &str) -> Result<(), &'static str> {
    ensure_relative_parent_directories(root, relative, false)
}

fn ensure_relative_parent_directories(
    root: &Path,
    relative: &str,
    private: bool,
) -> Result<(), &'static str> {
    if !valid_path(relative) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let mut components = relative.split('/').peekable();
    let mut parent = AnchoredDirectory::open_for_directory_writes(root)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            break;
        }
        if component.is_empty() || matches!(component, "." | "..") {
            return Err("hub_cloud_sync_stage_invalid");
        }
        parent = if private {
            parent.ensure_child_directory(std::ffi::OsStr::new(component))
        } else {
            parent.ensure_project_child_directory(std::ffi::OsStr::new(component))
        }
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    }
    Ok(())
}

fn project_file_path(root: &Path, relative: &str) -> Result<PathBuf, &'static str> {
    if !stage_path_allowed(relative) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let mut path = root.to_path_buf();
    for component in relative.split('/') {
        path.push(component);
        if path.exists() {
            let metadata =
                fs::symlink_metadata(&path).map_err(|_| "hub_cloud_sync_project_changed")?;
            if metadata.file_type().is_symlink() {
                return Err("hub_cloud_sync_local_conflict");
            }
            if path != root.join(relative) && !metadata.is_dir() {
                return Err("hub_cloud_sync_local_conflict");
            }
        }
    }
    Ok(path)
}

fn stage_file_path(directory: &Path, relative: &str) -> Result<PathBuf, &'static str> {
    if !stage_path_allowed(relative) {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let components = relative.split('/').collect::<Vec<_>>();
    let mut path = directory.to_path_buf();
    for component in components.iter().take(components.len().saturating_sub(1)) {
        path = path.join(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err("hub_cloud_sync_stage_invalid");
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        }
    }
    path.push(components.last().ok_or("hub_cloud_sync_stage_invalid")?);
    Ok(path)
}

fn read_project_file(root: &Path, relative: &str) -> Result<Vec<u8>, &'static str> {
    let target = project_file_path(root, relative)?;
    read_bounded(&target, MAX_BLOB_BYTES).map_err(|error| {
        if error == "hub_cloud_sync_stage_missing" {
            "hub_cloud_sync_project_changed"
        } else {
            error
        }
    })
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, &'static str> {
    read_bounded_file(open_without_following_links(path)?, limit)
}

fn read_private_bounded(
    project_root: &Path,
    path: &Path,
    limit: usize,
) -> Result<Vec<u8>, &'static str> {
    read_bounded_file(open_private_artifact_file(project_root, path)?, limit)
}

fn read_bounded_file(mut file: File, limit: usize) -> Result<Vec<u8>, &'static str> {
    let before = file
        .metadata()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if !before.is_file() || before.len() > limit as u64 {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    (&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                "hub_cloud_sync_stage_missing"
            } else {
                "hub_cloud_sync_stage_invalid"
            }
        })?;
    if bytes.len() as u64 != before.len()
        || file
            .metadata()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
            .len()
            != before.len()
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    Ok(bytes)
}

fn read_private_bounded_with_budget(
    project_root: &Path,
    path: &Path,
    limit: usize,
    budget: &mut ArtifactScanBudget,
) -> Result<Vec<u8>, &'static str> {
    read_bounded_file_with_budget(
        open_private_artifact_file(project_root, path)?,
        limit,
        budget,
    )
}

fn read_bounded_file_with_budget(
    mut file: File,
    limit: usize,
    budget: &mut ArtifactScanBudget,
) -> Result<Vec<u8>, &'static str> {
    let before = file
        .metadata()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if !before.is_file() || before.len() > limit as u64 {
        return Err("hub_cloud_sync_stage_invalid");
    }
    // Reserve the descriptor's observed length before reading. Limiting the read to that same
    // length means a concurrent extension is detected below without exceeding the reservation.
    budget.visit_read_bytes(before.len())?;
    let mut bytes = Vec::with_capacity(before.len() as usize);
    (&mut file)
        .take(before.len())
        .read_to_end(&mut bytes)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                "hub_cloud_sync_stage_missing"
            } else {
                "hub_cloud_sync_stage_invalid"
            }
        })?;
    if bytes.len() as u64 != before.len()
        || file
            .metadata()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
            .len()
            != before.len()
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    Ok(bytes)
}

fn open_private_artifact_file(project_root: &Path, path: &Path) -> Result<File, &'static str> {
    use std::path::Component;

    let root = if path.strip_prefix(project_root).is_ok() {
        project_root.to_path_buf()
    } else {
        project_root
            .canonicalize()
            .map_err(|_| "hub_cloud_sync_project_unavailable")?
    };
    let parent = path.parent().ok_or("hub_cloud_sync_stage_invalid")?;
    let relative_parent = parent
        .strip_prefix(&root)
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    let mut anchor = AnchoredDirectory::open(&root).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    for component in relative_parent.components() {
        let Component::Normal(name) = component else {
            return Err("hub_cloud_sync_stage_invalid");
        };
        anchor = anchor
            .open_existing_private_child(name)
            .map_err(map_private_open_error)?;
    }
    let name = path.file_name().ok_or("hub_cloud_sync_stage_invalid")?;
    anchor
        .open_existing_private_file(name)
        .map_err(map_private_open_error)
}

fn map_private_open_error(error: std::io::Error) -> &'static str {
    if error.kind() == std::io::ErrorKind::NotFound {
        "hub_cloud_sync_stage_missing"
    } else {
        "hub_cloud_sync_stage_invalid"
    }
}

fn open_without_following_links(path: &Path) -> Result<File, &'static str> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    "hub_cloud_sync_stage_missing"
                } else {
                    "hub_cloud_sync_stage_invalid"
                }
            })?;
        if file
            .metadata()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
            .file_attributes()
            & FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err("hub_cloud_sync_stage_invalid");
        }
        Ok(file)
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        const O_NOFOLLOW: i32 = 0x0002_0000;
        OpenOptions::new()
            .read(true)
            .custom_flags(O_NOFOLLOW)
            .open(path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    "hub_cloud_sync_stage_missing"
                } else {
                    "hub_cloud_sync_stage_invalid"
                }
            })
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let _ = path;
        Err("hub_cloud_sync_stage_invalid")
    }
    #[cfg(not(any(windows, unix)))]
    {
        OpenOptions::new().read(true).open(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                "hub_cloud_sync_stage_missing"
            } else {
                "hub_cloud_sync_stage_invalid"
            }
        })
    }
}

fn stage_path_allowed(path: &str) -> bool {
    valid_path(path)
        && !path
            .split('/')
            .any(|part| part.eq_ignore_ascii_case(".zircon"))
}

fn validate_stage_id(stage_id: &str) -> Result<(), &'static str> {
    super::super::operations::validate_id(stage_id).map_err(|_| "hub_cloud_sync_stage_invalid")
}

fn validate_revision(revision: &str) -> Result<(), &'static str> {
    if revision_number(revision).is_err() {
        return Err("hub_cloud_sync_stage_invalid");
    }
    Ok(())
}

fn revision_number(revision: &str) -> Result<i64, &'static str> {
    let number = revision
        .parse::<i64>()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    if number < 0 || number.to_string() != revision {
        return Err("hub_cloud_sync_stage_invalid");
    }
    Ok(number)
}

fn is_sha256_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn stage_read_error(error: &'static str) -> &'static str {
    if error == "hub_cloud_sync_stage_missing" {
        "hub_cloud_sync_stage_invalid"
    } else {
        error
    }
}

fn record_conflict(paths: &mut Vec<String>, count: &mut usize, path: &str) {
    *count += 1;
    if paths.len() < MAX_CONFLICT_PATHS {
        paths.push(path.to_owned());
    }
}

fn write_conflict_artifact(
    snapshot: &StagedSnapshot,
    paths: &[String],
    conflict_count: usize,
) -> Result<(), &'static str> {
    write_conflict_artifact_with_limit(snapshot, paths, conflict_count, MAX_CLOUD_ARTIFACT_BYTES)
}

fn write_conflict_artifact_with_limit(
    snapshot: &StagedSnapshot,
    paths: &[String],
    conflict_count: usize,
    quota_limit: u64,
) -> Result<(), &'static str> {
    let artifact = ConflictArtifact {
        schema_version: STAGE_SCHEMA_VERSION,
        stage_id: snapshot.stage_id.clone(),
        revision: snapshot.revision.clone(),
        manifest_digest: snapshot.manifest_digest.clone(),
        conflict_count,
        paths: paths.to_vec(),
    };
    let bytes = serde_json::to_vec(&artifact).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    let name = format!("{CONFLICT_ARTIFACT_PREFIX}{}.json", sha256(&bytes));
    let target = snapshot.directory.join(name);
    match read_private_bounded(&snapshot.project_root, &target, 1024 * 1024) {
        Ok(existing) if existing == bytes => return Ok(()),
        Ok(_) => return Err("hub_cloud_sync_stage_invalid"),
        Err("hub_cloud_sync_stage_missing") => {}
        Err(error) => return Err(error),
    }
    ensure_artifact_additional_capacity_with_limit(
        &snapshot.project_root,
        bytes.len() as u64,
        quota_limit,
    )?;
    match publish_new_file(&snapshot.directory, &target, &snapshot.stage_id, &bytes) {
        Ok(()) => Ok(()),
        Err("hub_cloud_sync_stage_exists") => {
            let existing = read_private_bounded(&snapshot.project_root, &target, 1024 * 1024)
                .map_err(stage_read_error)?;
            (existing == bytes)
                .then_some(())
                .ok_or("hub_cloud_sync_stage_invalid")
        }
        Err(error) => Err(error),
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "tests/staging.rs"]
mod tests;
