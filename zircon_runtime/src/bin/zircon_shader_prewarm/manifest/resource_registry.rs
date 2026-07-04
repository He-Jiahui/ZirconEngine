use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use zircon_runtime::asset::project::{
    shader_resource_records_from_asset_root as project_shader_resource_records_from_asset_root,
    shader_resource_records_from_asset_roots as project_shader_resource_records_from_asset_roots,
    ShaderResourceRecordExportError,
};
use zircon_runtime::core::resource::{
    ResourceId, ResourceKind, ResourceManager, ResourceRecord, ResourceState,
};

use crate::error::{ShaderPrewarmResourceRegistryError, ShaderPrewarmResourceRegistryResult};

#[derive(Clone, Debug, Default)]
pub(crate) struct ShaderPrewarmResourceRegistryOverlay {
    revisions_by_id: BTreeMap<ResourceId, u64>,
    revisions_by_locator: BTreeMap<String, u64>,
}

impl ShaderPrewarmResourceRegistryOverlay {
    pub(crate) fn read(path: &Path) -> ShaderPrewarmResourceRegistryResult<Self> {
        let bytes = fs::read(path).map_err(|source| ShaderPrewarmResourceRegistryError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let value = serde_json::from_slice::<Value>(&bytes).map_err(|source| {
            ShaderPrewarmResourceRegistryError::Parse {
                path: path.to_path_buf(),
                source,
            }
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
) -> ShaderPrewarmResourceRegistryResult<Vec<ResourceRecord>> {
    project_shader_resource_records_from_asset_root(asset_root).map_err(Into::into)
}

pub(crate) fn shader_resource_records_from_asset_roots(
    asset_roots: &[PathBuf],
) -> ShaderPrewarmResourceRegistryResult<Vec<ResourceRecord>> {
    project_shader_resource_records_from_asset_roots(asset_roots).map_err(Into::into)
}

pub(crate) fn shader_resource_records_from_manager(
    manager: &ResourceManager,
) -> Vec<ResourceRecord> {
    manager.ready_records_for_kind(ResourceKind::Shader)
}

fn resource_records_from_json_value(
    value: Value,
    path: &Path,
) -> ShaderPrewarmResourceRegistryResult<Vec<ResourceRecord>> {
    let records = if value.is_array() {
        value
    } else if let Some(records) = value.get("resources") {
        records.clone()
    } else if let Some(records) = value.get("records") {
        records.clone()
    } else {
        return Err(ShaderPrewarmResourceRegistryError::MissingRecordsArray {
            path: path.to_path_buf(),
        });
    };
    serde_json::from_value::<Vec<ResourceRecord>>(records).map_err(|source| {
        ShaderPrewarmResourceRegistryError::DecodeRecords {
            path: path.to_path_buf(),
            source,
        }
    })
}

impl From<ShaderResourceRecordExportError> for ShaderPrewarmResourceRegistryError {
    fn from(error: ShaderResourceRecordExportError) -> Self {
        match error {
            ShaderResourceRecordExportError::ReadRoot { path, source } => {
                ShaderPrewarmResourceRegistryError::ReadRoot { path, source }
            }
            ShaderResourceRecordExportError::ReadRootEntry { path, source } => {
                ShaderPrewarmResourceRegistryError::ReadRootEntry { path, source }
            }
            ShaderResourceRecordExportError::LoadMetadata { path, source } => {
                ShaderPrewarmResourceRegistryError::LoadMetadata { path, source }
            }
            ShaderResourceRecordExportError::DuplicateRecordId {
                id,
                existing_locator,
                new_locator,
            } => ShaderPrewarmResourceRegistryError::DuplicateRecordId {
                id,
                existing_locator,
                new_locator,
            },
            ShaderResourceRecordExportError::DuplicateLocator {
                locator,
                existing_id,
                new_id,
            } => ShaderPrewarmResourceRegistryError::DuplicateLocator {
                locator,
                existing_id,
                new_id,
            },
        }
    }
}

#[cfg(test)]
mod tests;
