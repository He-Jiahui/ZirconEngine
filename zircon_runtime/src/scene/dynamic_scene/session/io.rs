use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::scene::{EntityRemap, LevelSystem, World};

use super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
    RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport,
    RuntimeSessionArchivePathStatus, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionArchiveStatistics,
    RuntimeSessionLevelRestoreReport, RuntimeSessionMetadata, RuntimeSessionSlotDiffReport,
    RuntimeSessionSlotSummary,
};

static TEMP_ARCHIVE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn load_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    RuntimeSessionArchive::from_versioned_json(&fs::read_to_string(path)?)
}

pub(super) fn load_or_empty_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    match fs::read_to_string(path) {
        Ok(json) => RuntimeSessionArchive::from_versioned_json(&json),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(RuntimeSessionArchive::empty()),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn load_manifest_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    load_from_path(path)?.manifest()
}

pub(super) fn single_slot_archive_from_path(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<RuntimeSessionArchive, RuntimeSessionArchiveError> {
    load_from_path(path)?.single_slot_archive(slot_id)
}

pub(super) fn slot_summary_from_path(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<Option<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?.slot(slot_id).cloned())
}

pub(super) fn contains_slot_from_path(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<bool, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?.slot(slot_id).is_some())
}

pub(super) fn slot_ids_from_path(
    path: impl AsRef<Path>,
) -> Result<Vec<String>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?
        .slot_ids()
        .map(str::to_string)
        .collect())
}

pub(super) fn slots_with_tag_from_path(
    path: impl AsRef<Path>,
    tag: &str,
) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?
        .slots_with_tag(tag)
        .cloned()
        .collect())
}

pub(super) fn slots_matching_display_name_from_path(
    path: impl AsRef<Path>,
    query: &str,
) -> Result<Vec<RuntimeSessionSlotSummary>, RuntimeSessionArchiveError> {
    Ok(load_manifest_from_path(path)?
        .slots_matching_display_name(query)
        .cloned()
        .collect())
}

pub(super) fn latest_updated_slot_id_from_path(
    path: impl AsRef<Path>,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    load_from_path(path)?.latest_updated_slot_id()
}

pub(super) fn oldest_updated_slot_id_from_path(
    path: impl AsRef<Path>,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    load_from_path(path)?.oldest_updated_slot_id()
}

pub(super) fn latest_updated_slot_id_with_tag_from_path(
    path: impl AsRef<Path>,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    load_from_path(path)?.latest_updated_slot_id_with_tag(tag)
}

pub(super) fn oldest_updated_slot_id_with_tag_from_path(
    path: impl AsRef<Path>,
    tag: &str,
) -> Result<Option<String>, RuntimeSessionArchiveError> {
    load_from_path(path)?.oldest_updated_slot_id_with_tag(tag)
}

pub(super) fn statistics_from_path(
    path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveStatistics, RuntimeSessionArchiveError> {
    load_from_path(path)?.statistics()
}

pub(super) fn inspect_path(path: impl AsRef<Path>) -> RuntimeSessionArchivePathStatus {
    match fs::read_to_string(path) {
        Ok(json) => match RuntimeSessionArchive::from_versioned_json(&json)
            .and_then(|archive| archive.manifest())
        {
            Ok(manifest) => RuntimeSessionArchivePathStatus::Available { manifest },
            Err(error) => RuntimeSessionArchivePathStatus::Invalid { error },
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            RuntimeSessionArchivePathStatus::Missing
        }
        Err(error) => RuntimeSessionArchivePathStatus::Invalid {
            error: error.into(),
        },
    }
}

pub(super) fn save_to_path(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<(), RuntimeSessionArchiveError> {
    let path = path.as_ref();
    ensure_parent_dir(path)?;
    fs::write(path, archive.to_versioned_json_pretty()?)?;
    Ok(())
}

pub(super) fn save_to_path_atomically(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<(), RuntimeSessionArchiveError> {
    let path = path.as_ref();
    ensure_parent_dir(path)?;
    let temp_path = temporary_archive_path(path, "tmp");
    let payload = archive.to_versioned_json_pretty()?;
    if let Err(error) = fs::write(&temp_path, payload) {
        let _ = fs::remove_file(&temp_path);
        return Err(error.into());
    }

    let backup_path = prepare_existing_target_backup(path, &temp_path)?;

    match fs::rename(&temp_path, path) {
        Ok(()) => {
            if let Some(backup_path) = backup_path.as_ref() {
                let _ = fs::remove_file(backup_path);
            }
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            restore_existing_target_backup(path, backup_path.as_deref());
            Err(error.into())
        }
    }
}

pub(super) fn capture_world_slot_to_path_atomically(
    path: impl AsRef<Path>,
    slot_id: impl Into<String>,
    world: &World,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let mut archive = load_or_empty_from_path(path)?;
    archive.capture_world_slot(slot_id, world, metadata)?;
    save_to_path_atomically(&archive, path)?;
    archive.manifest()
}

pub(super) fn save_single_slot_archive_from_path_atomically(
    source_path: impl AsRef<Path>,
    slot_id: &str,
    target_path: impl AsRef<Path>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let source_path = source_path.as_ref();
    let target_path = target_path.as_ref();
    if source_path == target_path {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "runtime session single-slot archive target must differ from source archive",
        )
        .into());
    }

    let archive = single_slot_archive_from_path(source_path, slot_id)?;
    save_to_path_atomically(&archive, target_path)?;
    archive.manifest()
}

pub(super) fn capture_level_slot_to_path_atomically(
    path: impl AsRef<Path>,
    slot_id: impl Into<String>,
    level: &LevelSystem,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let mut archive = load_or_empty_from_path(path)?;
    archive.capture_level_slot(slot_id, level)?;
    save_to_path_atomically(&archive, path)?;
    archive.manifest()
}

pub(super) fn restore_slot_from_path_to_empty_world(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<World, RuntimeSessionArchiveError> {
    load_from_path(path)?.restore_slot_to_empty_world(slot_id)
}

pub(super) fn restore_slot_from_path_into_level(
    path: impl AsRef<Path>,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<RuntimeSessionLevelRestoreReport, RuntimeSessionArchiveError> {
    load_from_path(path)?.restore_slot_into_level(slot_id, level)
}

pub(super) fn apply_slot_from_path_to_world(
    path: impl AsRef<Path>,
    slot_id: &str,
    world: &mut World,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    load_from_path(path)?.apply_slot(slot_id, world)
}

pub(super) fn apply_slot_from_path_to_level(
    path: impl AsRef<Path>,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<EntityRemap, RuntimeSessionArchiveError> {
    load_from_path(path)?.apply_slot_to_level(slot_id, level)
}

pub(super) fn diff_slot_from_path_with_world(
    path: impl AsRef<Path>,
    slot_id: &str,
    world: &World,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    load_from_path(path)?.diff_slot_with_world(slot_id, world)
}

pub(super) fn diff_slot_from_path_with_level(
    path: impl AsRef<Path>,
    slot_id: &str,
    level: &LevelSystem,
) -> Result<RuntimeSessionSlotDiffReport, RuntimeSessionArchiveError> {
    load_from_path(path)?.diff_slot_with_level(slot_id, level)
}

pub(super) fn rename_slot_at_path_atomically(
    path: impl AsRef<Path>,
    old_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.rename_slot(old_slot_id, new_slot_id)?;
        Ok(())
    })
}

pub(super) fn update_slot_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    slot_id: &str,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.update_slot_metadata(slot_id, metadata)?;
        Ok(())
    })
}

pub(super) fn touch_slot_at_path_atomically(
    path: impl AsRef<Path>,
    slot_id: &str,
    updated_at_unix_millis: u64,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.touch_slot(slot_id, updated_at_unix_millis)?;
        Ok(())
    })
}

pub(super) fn remove_slot_at_path_atomically(
    path: impl AsRef<Path>,
    slot_id: &str,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.remove_slot(slot_id).map(|_| ()).ok_or_else(|| {
            RuntimeSessionArchiveError::MissingSlot {
                slot_id: slot_id.to_string(),
            }
        })
    })
}

pub(super) fn copy_slot_at_path_atomically(
    path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.copy_slot(source_slot_id, new_slot_id)?;
        Ok(())
    })
}

pub(super) fn copy_slot_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.copy_slot_with_metadata(source_slot_id, new_slot_id, metadata)?;
        Ok(())
    })
}

pub(super) fn import_slot_from_archive_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.import_slot_from_archive(incoming, source_slot_id, new_slot_id)?;
        Ok(())
    })
}

pub(super) fn import_slot_from_archive_path_at_path_atomically(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let incoming = load_from_path(source_path)?;
    import_slot_from_archive_at_path_atomically(path, &incoming, source_slot_id, new_slot_id)
}

pub(super) fn import_slot_from_archive_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_atomically(path, |archive| {
        archive.import_slot_from_archive_with_metadata(
            incoming,
            source_slot_id,
            new_slot_id,
            metadata,
        )?;
        Ok(())
    })
}

pub(super) fn import_slot_from_archive_path_with_metadata_at_path_atomically(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    source_slot_id: &str,
    new_slot_id: impl Into<String>,
    metadata: RuntimeSessionMetadata,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    let incoming = load_from_path(source_path)?;
    import_slot_from_archive_with_metadata_at_path_atomically(
        path,
        &incoming,
        source_slot_id,
        new_slot_id,
        metadata,
    )
}

pub(super) fn merge_archive_at_path_atomically(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    mutate_archive_at_path_with_report_atomically(path, |archive| {
        archive.merge_archive(incoming, policy)
    })
}

pub(super) fn preview_merge_archive_at_path(
    path: impl AsRef<Path>,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    load_from_path(path)?.preview_merge_archive(incoming, policy)
}

pub(super) fn merge_archive_from_path_at_path_atomically(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    let incoming = load_from_path(source_path)?;
    merge_archive_at_path_atomically(path, &incoming, policy)
}

pub(super) fn preview_merge_archive_from_path_at_path(
    path: impl AsRef<Path>,
    source_path: impl AsRef<Path>,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    let incoming = load_from_path(source_path)?;
    preview_merge_archive_at_path(path, &incoming, policy)
}

pub(super) fn prune_slots_at_path_atomically(
    path: impl AsRef<Path>,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    mutate_archive_at_path_with_report_atomically(path, |archive| archive.prune_slots(policy))
}

pub(super) fn prune_slots_with_tag_at_path_atomically(
    path: impl AsRef<Path>,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    mutate_archive_at_path_with_report_atomically(path, |archive| {
        archive.prune_slots_with_tag(tag, policy)
    })
}

pub(super) fn preview_prune_slots_from_path(
    path: impl AsRef<Path>,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    load_from_path(path)?.preview_prune_slots(policy)
}

pub(super) fn preview_prune_slots_with_tag_from_path(
    path: impl AsRef<Path>,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    load_from_path(path)?.preview_prune_slots_with_tag(tag, policy)
}

fn mutate_archive_at_path_atomically(
    path: impl AsRef<Path>,
    mutate: impl FnOnce(&mut RuntimeSessionArchive) -> Result<(), RuntimeSessionArchiveError>,
) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
    mutate_archive_at_path_with_report_atomically(path, |archive| {
        mutate(archive)?;
        archive.manifest()
    })
}

fn mutate_archive_at_path_with_report_atomically<T>(
    path: impl AsRef<Path>,
    mutate: impl FnOnce(&mut RuntimeSessionArchive) -> Result<T, RuntimeSessionArchiveError>,
) -> Result<T, RuntimeSessionArchiveError> {
    let path = path.as_ref();
    let mut archive = load_from_path(path)?;
    let report = mutate(&mut archive)?;
    save_to_path_atomically(&archive, path)?;
    Ok(report)
}

fn ensure_parent_dir(path: &Path) -> Result<(), RuntimeSessionArchiveError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn prepare_existing_target_backup(
    path: &Path,
    temp_path: &Path,
) -> Result<Option<PathBuf>, RuntimeSessionArchiveError> {
    if !path.exists() {
        return Ok(None);
    }

    if !path.is_file() {
        let _ = fs::remove_file(temp_path);
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "runtime session archive target path is not a file",
        )
        .into());
    }

    let backup_path = temporary_archive_path(path, "bak");
    if let Err(error) = fs::rename(path, &backup_path) {
        let _ = fs::remove_file(temp_path);
        return Err(error.into());
    }
    Ok(Some(backup_path))
}

fn restore_existing_target_backup(path: &Path, backup_path: Option<&Path>) {
    if let Some(backup_path) = backup_path {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
        let _ = fs::rename(backup_path, path);
    }
}

fn temporary_archive_path(path: &Path, extension: &str) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("runtime-session-archive");
    let counter = TEMP_ARCHIVE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    parent.join(format!(
        ".{file_name}.{}.{}.{}.{}",
        process::id(),
        unique,
        counter,
        extension
    ))
}
