use std::{
    ffi::OsStr,
    path::{Component, Path},
};

use crate::file_io::{AnchoredDirectory, AnchoredEntry};

use super::{
    is_publication_record_name, is_publish_temporary_name, is_sha256_digest,
    map_private_open_error, map_stage_tree_error, publication_record_name, read_bounded_file,
    sha256, stage_path_allowed, stage_read_error, temporary_name_matches, validate_stage_id,
    ArtifactScanBudget, ProjectCloudSyncLease, PublicationArtifact, StageMarker, MAX_BLOB_BYTES,
    MAX_CLOUD_DIRECTORY_DEPTH, PUBLICATION_ROOT, PUBLICATION_SCHEMA_VERSION, STAGE_MARKER,
    STAGE_ROOT, STAGE_SCHEMA_VERSION,
};

pub(crate) fn recover_orphaned_publish_temporaries(
    project_root: &Path,
    project_lease: &ProjectCloudSyncLease,
) -> Result<usize, &'static str> {
    recover_with_anchor_hook(project_root, project_lease, || {})
}

fn recover_with_anchor_hook(
    project_root: &Path,
    _project_lease: &ProjectCloudSyncLease,
    after_open: impl FnOnce(),
) -> Result<usize, &'static str> {
    let root = project_root
        .canonicalize()
        .map_err(|_| "hub_cloud_sync_project_unavailable")?;
    let project = AnchoredDirectory::open(&root).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    let Some(cloud) = existing_private_anchor(&project, &[".zircon", "cloud"])? else {
        return Ok(0);
    };
    after_open();
    require_current_path(&cloud)?;
    let mut budget = ArtifactScanBudget::default();
    let mut removed = 0;
    reap_managed_publish_temporaries(&cloud, &mut budget, &mut removed, 0)?;
    reap_project_publications(&project, &mut budget, &mut removed)?;
    Ok(removed)
}

fn existing_private_anchor(
    project: &AnchoredDirectory,
    components: &[&str],
) -> Result<Option<AnchoredDirectory>, &'static str> {
    let mut directory = project
        .try_clone()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    for component in components {
        match directory.open_existing_private_child(OsStr::new(component)) {
            Ok(child) => directory = child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        }
    }
    Ok(Some(directory))
}

fn require_current_path(directory: &AnchoredDirectory) -> Result<(), &'static str> {
    directory
        .path_still_matches()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?
        .then_some(())
        .ok_or("hub_cloud_sync_stage_invalid")
}

fn recovery_names(
    directory: &AnchoredDirectory,
    budget: &ArtifactScanBudget,
) -> Result<Vec<std::ffi::OsString>, &'static str> {
    directory
        .read_names(super::MAX_ARTIFACT_ENTRIES.saturating_sub(budget.entries))
        .map_err(map_stage_tree_error)
}

fn reap_managed_publish_temporaries(
    directory: &AnchoredDirectory,
    budget: &mut ArtifactScanBudget,
    removed: &mut usize,
    depth: usize,
) -> Result<(), &'static str> {
    if depth > MAX_CLOUD_DIRECTORY_DEPTH {
        return Err("hub_cloud_sync_storage_quota_exceeded");
    }
    budget.visit_directory()?;
    for name in recovery_names(directory, budget)? {
        budget.visit_entry()?;
        let entry = directory
            .open_entry(&name)
            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
        match entry {
            AnchoredEntry::Directory(child) => {
                reap_managed_publish_temporaries(child.as_directory(), budget, removed, depth + 1)?;
            }
            AnchoredEntry::File(file) if is_publish_temporary_name(&name.to_string_lossy()) => {
                file.remove().map_err(|_| "hub_cloud_sync_stage_invalid")?;
                *removed = removed.saturating_add(1);
            }
            AnchoredEntry::File(_) => {}
        }
    }
    Ok(())
}

fn reap_project_publications(
    project: &AnchoredDirectory,
    budget: &mut ArtifactScanBudget,
    removed: &mut usize,
) -> Result<(), &'static str> {
    let Some(publications) = existing_private_anchor(project, PUBLICATION_ROOT)? else {
        return Ok(());
    };
    require_current_path(&publications)?;
    budget.visit_directory()?;
    for name in recovery_names(&publications, budget)? {
        budget.visit_entry()?;
        let Some(stage_id) = name.to_str() else {
            continue;
        };
        if validate_stage_id(stage_id).is_err() {
            continue;
        }
        let AnchoredEntry::Directory(stage) = publications
            .open_private_entry(&name)
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
        else {
            continue;
        };
        budget.visit_directory()?;
        reap_stage_publication_records(project, stage_id, stage.as_directory(), budget, removed)?;
        stage
            .remove_if_empty()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    }
    Ok(())
}

pub(super) fn reap_stage_publication_records(
    project: &AnchoredDirectory,
    stage_id: &str,
    directory: &AnchoredDirectory,
    budget: &mut ArtifactScanBudget,
    removed: &mut usize,
) -> Result<(), &'static str> {
    for name in recovery_names(directory, budget)? {
        budget.visit_entry()?;
        let Some(record_name) = name.to_str() else {
            continue;
        };
        if !is_publication_record_name(record_name) {
            continue;
        }
        let AnchoredEntry::File(record) = directory
            .open_private_entry(&name)
            .map_err(|_| "hub_cloud_sync_stage_invalid")?
        else {
            continue;
        };
        let bytes = read_bounded_file(
            record
                .try_clone_reader()
                .map_err(|_| "hub_cloud_sync_stage_invalid")?,
            16 * 1024,
        )?;
        if record_name != publication_record_name(&bytes) {
            return Err("hub_cloud_sync_stage_invalid");
        }
        let artifact: PublicationArtifact =
            serde_json::from_slice(&bytes).map_err(|_| "hub_cloud_sync_stage_invalid")?;
        reap_one_project_publication(project, stage_id, &artifact)?;
        record
            .remove()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?;
        *removed = removed.saturating_add(1);
    }
    Ok(())
}

fn read_stage_relative(
    stage: &AnchoredDirectory,
    relative: &Path,
    limit: usize,
) -> Result<Vec<u8>, &'static str> {
    let parent = relative.parent().ok_or("hub_cloud_sync_stage_invalid")?;
    let mut directory = stage
        .try_clone()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    for component in parent.components() {
        let Component::Normal(name) = component else {
            return Err("hub_cloud_sync_stage_invalid");
        };
        directory = directory
            .open_existing_private_child(name)
            .map_err(map_private_open_error)?;
    }
    let name = relative.file_name().ok_or("hub_cloud_sync_stage_invalid")?;
    let file = directory
        .open_existing_private_file(name)
        .map_err(map_private_open_error)?;
    // Retain the directory and all ancestors through the actual bounded read.
    read_bounded_file(file, limit)
}

fn reap_one_project_publication(
    project: &AnchoredDirectory,
    stage_id: &str,
    artifact: &PublicationArtifact,
) -> Result<(), &'static str> {
    reap_one_with_parent_hook(project, stage_id, artifact, || {})
}

fn reap_one_with_parent_hook(
    project: &AnchoredDirectory,
    stage_id: &str,
    artifact: &PublicationArtifact,
    after_open: impl FnOnce(),
) -> Result<(), &'static str> {
    reap_one_with_hooks(project, stage_id, artifact, || {}, after_open)
}

fn reap_one_with_hooks(
    project: &AnchoredDirectory,
    stage_id: &str,
    artifact: &PublicationArtifact,
    after_marker_read: impl FnOnce(),
    after_open: impl FnOnce(),
) -> Result<(), &'static str> {
    if artifact.schema_version != PUBLICATION_SCHEMA_VERSION
        || artifact.stage_id != stage_id
        || !stage_path_allowed(&artifact.relative_path)
        || !is_sha256_digest(&artifact.digest)
        || artifact.bytes > MAX_BLOB_BYTES as u64
        || !temporary_name_matches(&artifact.temporary_name, stage_id, &artifact.digest)
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let components = STAGE_ROOT
        .iter()
        .copied()
        .chain(std::iter::once(stage_id))
        .collect::<Vec<_>>();
    let stage =
        existing_private_anchor(project, &components)?.ok_or("hub_cloud_sync_stage_invalid")?;
    require_current_path(&stage)?;
    let marker_bytes = read_stage_relative(&stage, Path::new(STAGE_MARKER), 8 * 1024 * 1024 + 4096)
        .map_err(stage_read_error)?;
    after_marker_read();
    require_current_path(&stage)?;
    let marker: StageMarker =
        serde_json::from_slice(&marker_bytes).map_err(|_| "hub_cloud_sync_stage_invalid")?;
    let entry = marker
        .manifest
        .files
        .iter()
        .find(|entry| entry.path == artifact.relative_path)
        .ok_or("hub_cloud_sync_stage_invalid")?;
    if marker.schema_version != STAGE_SCHEMA_VERSION
        || marker.stage_id != stage_id
        || marker.manifest.canonical_digest().ok().as_deref()
            != Some(marker.manifest_digest.as_str())
        || entry.digest != artifact.digest
        || entry.bytes != artifact.bytes
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let staged_bytes =
        read_stage_relative(&stage, Path::new(&artifact.relative_path), MAX_BLOB_BYTES)
            .map_err(stage_read_error)?;
    if staged_bytes.len() as u64 != artifact.bytes || sha256(&staged_bytes) != artifact.digest {
        return Err("hub_cloud_sync_stage_invalid");
    }
    let Some(parent) = existing_project_parent(project, &artifact.relative_path)? else {
        return Ok(());
    };
    after_open();
    require_current_path(&parent)?;
    let temporary = match parent.open_entry(OsStr::new(&artifact.temporary_name)) {
        Ok(AnchoredEntry::File(file)) => file,
        Ok(AnchoredEntry::Directory(_)) => return Err("hub_cloud_sync_stage_invalid"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("hub_cloud_sync_stage_invalid"),
    };
    let temporary_bytes = read_bounded_file(
        temporary
            .try_clone_reader()
            .map_err(|_| "hub_cloud_sync_stage_invalid")?,
        MAX_BLOB_BYTES,
    )?;
    if temporary_bytes.len() > staged_bytes.len()
        || staged_bytes[..temporary_bytes.len()] != temporary_bytes
    {
        return Err("hub_cloud_sync_stage_invalid");
    }
    require_current_path(&stage)?;
    require_current_path(&parent)?;
    temporary
        .remove()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    Ok(())
}

fn existing_project_parent(
    project: &AnchoredDirectory,
    relative: &str,
) -> Result<Option<AnchoredDirectory>, &'static str> {
    let components = relative.split('/').collect::<Vec<_>>();
    let mut parent = project
        .try_clone()
        .map_err(|_| "hub_cloud_sync_stage_invalid")?;
    for component in components.iter().take(components.len().saturating_sub(1)) {
        match parent.open_existing_child_directory(OsStr::new(component)) {
            Ok(child) => parent = child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("hub_cloud_sync_stage_invalid"),
        }
    }
    Ok(Some(parent))
}

#[cfg(test)]
#[path = "recovery/tests/cases.rs"]
mod tests;
