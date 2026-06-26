use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde_json::Value;
use zircon_runtime::core::resource::{ResourceId, ResourceKind, ResourceRecord};

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
            if record.kind != ResourceKind::Shader || record.revision == 0 {
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
