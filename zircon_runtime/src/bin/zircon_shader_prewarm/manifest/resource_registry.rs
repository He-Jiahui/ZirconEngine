use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use zircon_runtime::asset::project::AssetMetaDocument;
use zircon_runtime::core::resource::{
    ResourceId, ResourceKind, ResourceManager, ResourceRecord, ResourceState,
};

use super::revision::asset_scan_revision_from_source_hash;

#[derive(Clone, Debug, Default)]
pub(crate) struct ShaderPrewarmResourceRegistryOverlay {
    revisions_by_id: BTreeMap<ResourceId, u64>,
    revisions_by_locator: BTreeMap<String, u64>,
}

impl ShaderPrewarmResourceRegistryOverlay {
    pub(crate) fn read(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|error| {
            format!(
                "failed to read shader prewarm resource registry {}: {error}",
                path.display()
            )
        })?;
        let value = serde_json::from_slice::<Value>(&bytes).map_err(|error| {
            format!(
                "failed to parse shader prewarm resource registry {}: {error}",
                path.display()
            )
        })?;
        let records = resource_records_from_json_value(value, path)?;
        Ok(Self::from_records(records))
    }

    pub(crate) fn from_records(records: impl IntoIterator<Item = ResourceRecord>) -> Self {
        let mut overlay = Self::default();
        for record in records {
            if record.kind != ResourceKind::Shader
                || record.state != ResourceState::Ready
                || record.revision == 0
            {
                continue;
            }
            overlay.revisions_by_id.insert(record.id, record.revision);
            overlay
                .revisions_by_locator
                .insert(record.primary_locator.to_string(), record.revision);
            if let Some(artifact_locator) = &record.artifact_locator {
                overlay
                    .revisions_by_locator
                    .insert(artifact_locator.to_string(), record.revision);
            }
        }
        overlay
    }

    pub(crate) fn revision_for(&self, resource_id: ResourceId, stable_label: &str) -> Option<u64> {
        self.revisions_by_id
            .get(&resource_id)
            .copied()
            .or_else(|| self.revisions_by_locator.get(stable_label).copied())
    }
}

pub(crate) fn shader_resource_records_from_asset_root(
    asset_root: &Path,
) -> Result<Vec<ResourceRecord>, String> {
    let mut records = Vec::new();
    collect_shader_resource_records(asset_root, &mut records)?;
    records.sort_by(|left, right| left.primary_locator.cmp(&right.primary_locator));
    Ok(records)
}

pub(crate) fn shader_resource_records_from_asset_roots(
    asset_roots: &[PathBuf],
) -> Result<Vec<ResourceRecord>, String> {
    let mut records = Vec::new();
    for asset_root in asset_roots {
        records.extend(shader_resource_records_from_asset_root(asset_root)?);
    }
    deduplicate_shader_resource_records(records)
}

pub(crate) fn shader_resource_records_from_manager(
    manager: &ResourceManager,
) -> Vec<ResourceRecord> {
    manager.ready_records_for_kind(ResourceKind::Shader)
}

fn deduplicate_shader_resource_records(
    records: Vec<ResourceRecord>,
) -> Result<Vec<ResourceRecord>, String> {
    let mut records_by_id = BTreeMap::new();
    let mut ids_by_locator = BTreeMap::new();
    for record in records {
        if let Some(existing) = records_by_id.get(&record.id) {
            if existing.primary_locator != record.primary_locator {
                return Err(format!(
                    "shader resource registry record id {:?} maps both {} and {}",
                    record.id, existing.primary_locator, record.primary_locator
                ));
            }
            continue;
        }
        if let Some(existing_id) = ids_by_locator.get(&record.primary_locator) {
            return Err(format!(
                "shader resource registry locator {} maps both {:?} and {:?}",
                record.primary_locator, existing_id, record.id
            ));
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

fn resource_records_from_json_value(
    value: Value,
    path: &Path,
) -> Result<Vec<ResourceRecord>, String> {
    let records = if value.is_array() {
        value
    } else if let Some(records) = value.get("resources") {
        records.clone()
    } else if let Some(records) = value.get("records") {
        records.clone()
    } else {
        return Err(format!(
            "shader prewarm resource registry {} must be a ResourceRecord array or contain a resources/records array",
            path.display()
        ));
    };
    serde_json::from_value::<Vec<ResourceRecord>>(records).map_err(|error| {
        format!(
            "failed to decode shader prewarm resource records {}: {error}",
            path.display()
        )
    })
}

fn collect_shader_resource_records(
    root: &Path,
    records: &mut Vec<ResourceRecord>,
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).map_err(|error| {
        format!(
            "failed to read shader resource registry root {}: {error}",
            root.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read shader resource registry root {} entry: {error}",
                root.display()
            )
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
) -> Result<(), String> {
    let meta = AssetMetaDocument::load(meta_path).map_err(|error| {
        format!(
            "failed to load shader resource registry metadata {}: {error}",
            meta_path.display()
        )
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

#[cfg(test)]
mod tests;
