use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceState,
};

use super::AssetMetaDocument;

pub type ShaderResourceRecordExportResult<T> =
    std::result::Result<T, ShaderResourceRecordExportError>;

#[derive(Debug, Error)]
pub enum ShaderResourceRecordExportError {
    #[error("failed to read shader resource registry root {path:?}: {source}")]
    ReadRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read shader resource registry root {path:?} entry: {source}")]
    ReadRootEntry {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to load shader resource registry metadata {path:?}: {source}")]
    LoadMetadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "shader resource registry record id {id:?} maps both {existing_locator} and {new_locator}"
    )]
    DuplicateRecordId {
        id: ResourceId,
        existing_locator: ResourceLocator,
        new_locator: ResourceLocator,
    },
    #[error("shader resource registry locator {locator} maps both {existing_id:?} and {new_id:?}")]
    DuplicateLocator {
        locator: ResourceLocator,
        existing_id: ResourceId,
        new_id: ResourceId,
    },
}

pub fn shader_resource_records_from_asset_root(
    asset_root: &Path,
) -> ShaderResourceRecordExportResult<Vec<ResourceRecord>> {
    let mut records = Vec::new();
    collect_shader_resource_records(asset_root, &mut records)?;
    deduplicate_shader_resource_records(records)
}

pub fn shader_resource_records_from_asset_roots(
    asset_roots: &[PathBuf],
) -> ShaderResourceRecordExportResult<Vec<ResourceRecord>> {
    let mut records = Vec::new();
    for asset_root in asset_roots {
        records.extend(shader_resource_records_from_asset_root(asset_root)?);
    }
    deduplicate_shader_resource_records(records)
}

fn deduplicate_shader_resource_records(
    records: Vec<ResourceRecord>,
) -> ShaderResourceRecordExportResult<Vec<ResourceRecord>> {
    // Resource ids and locators are both stable identity inputs for staged prewarm.
    // Matching pairs collapse; mismatched pairs fail before either caller consumes them.
    let mut records_by_id: BTreeMap<ResourceId, ResourceRecord> = BTreeMap::new();
    let mut ids_by_locator: BTreeMap<ResourceLocator, ResourceId> = BTreeMap::new();
    for record in records {
        if let Some(existing) = records_by_id.get(&record.id) {
            if existing.primary_locator != record.primary_locator {
                return Err(ShaderResourceRecordExportError::DuplicateRecordId {
                    id: record.id,
                    existing_locator: existing.primary_locator.clone(),
                    new_locator: record.primary_locator,
                });
            }
            continue;
        }
        if let Some(existing_id) = ids_by_locator.get(&record.primary_locator) {
            return Err(ShaderResourceRecordExportError::DuplicateLocator {
                locator: record.primary_locator,
                existing_id: *existing_id,
                new_id: record.id,
            });
        }
        ids_by_locator.insert(record.primary_locator.clone(), record.id);
        records_by_id.insert(record.id, record);
    }
    let mut records = records_by_id.into_values().collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.primary_locator
            .cmp(&right.primary_locator)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(records)
}

fn collect_shader_resource_records(
    root: &Path,
    records: &mut Vec<ResourceRecord>,
) -> ShaderResourceRecordExportResult<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|source| ShaderResourceRecordExportError::ReadRoot {
        path: root.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ShaderResourceRecordExportError::ReadRootEntry {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_shader_resource_records(&path, records)?;
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".zmeta"))
        {
            append_shader_records_from_meta(&path, records)?;
        }
    }
    Ok(())
}

fn append_shader_records_from_meta(
    meta_path: &Path,
    records: &mut Vec<ResourceRecord>,
) -> ShaderResourceRecordExportResult<()> {
    let meta = AssetMetaDocument::load(meta_path).map_err(|source| {
        ShaderResourceRecordExportError::LoadMetadata {
            path: meta_path.to_path_buf(),
            source,
        }
    })?;
    let revision = asset_scan_revision_from_source_hash(&meta.source_hash);

    if meta.entries.is_empty() {
        if meta.asset_kind == ResourceKind::Shader {
            records.push(shader_record_for_meta_root(&meta, revision));
        }
        return Ok(());
    }

    for entry in &meta.entries {
        if entry.asset_kind != ResourceKind::Shader {
            continue;
        }
        let mut record = ResourceRecord::new(
            ResourceId::from_asset_uuid(entry.uuid),
            entry.asset_kind,
            entry.url.clone(),
        )
        .with_source_hash(meta.source_hash.clone())
        .with_importer_id(meta.importer_id.clone())
        .with_importer_version(meta.importer_version)
        .with_config_hash(meta.config_hash.clone())
        .with_state(ResourceState::Ready);
        record.revision = revision;
        if let Some(artifact_locator) = entry.artifact_locator.clone() {
            record = record.with_artifact_locator(artifact_locator);
        }
        records.push(record);
    }
    Ok(())
}

fn shader_record_for_meta_root(meta: &AssetMetaDocument, revision: u64) -> ResourceRecord {
    let mut record = ResourceRecord::new(
        ResourceId::from_asset_uuid(meta.uuid),
        meta.asset_kind,
        meta.url.clone(),
    )
    .with_source_hash(meta.source_hash.clone())
    .with_importer_id(meta.importer_id.clone())
    .with_importer_version(meta.importer_version)
    .with_config_hash(meta.config_hash.clone())
    .with_state(ResourceState::Ready);
    record.revision = revision;
    if let Some(artifact_locator) = meta.artifact_locator.clone() {
        record = record.with_artifact_locator(artifact_locator);
    }
    record
}

fn asset_scan_revision_from_source_hash(source_hash: &str) -> u64 {
    let source_hash = source_hash.trim();
    if source_hash.is_empty() {
        return 1;
    }
    non_zero_revision_from_hash(blake3::hash(source_hash.as_bytes()))
}

fn non_zero_revision_from_hash(hash: blake3::Hash) -> u64 {
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    let revision = u64::from_le_bytes(bytes);
    if revision == 0 {
        1
    } else {
        revision
    }
}
