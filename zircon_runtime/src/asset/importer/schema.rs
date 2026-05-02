use super::AssetSchemaMigrationReport;
use crate::asset::AssetImportError;

pub trait AssetSchemaMigrator {
    fn current_schema_version(&self) -> u32;
    fn minimum_supported_schema_version(&self) -> u32;

    fn migrate_source_schema(
        &self,
        source_schema_version: Option<u32>,
    ) -> Result<AssetSchemaMigrationReport, AssetImportError> {
        let target = self.current_schema_version();
        let Some(source) = source_schema_version else {
            return Ok(AssetSchemaMigrationReport {
                source_schema_version: None,
                target_schema_version: target,
                summary: "source schema was not declared; imported as current schema".to_string(),
            });
        };
        if source > target {
            return Err(AssetImportError::SchemaMigration(format!(
                "future asset schema version {source} is newer than importer target {target}"
            )));
        }
        let minimum = self.minimum_supported_schema_version();
        if source < minimum {
            return Err(AssetImportError::SchemaMigration(format!(
                "asset schema version {source} is older than minimum supported {minimum}"
            )));
        }
        let summary = if source == target {
            format!("asset schema already current at version {target}")
        } else {
            format!("asset schema migrated from version {source} to {target}")
        };
        Ok(AssetSchemaMigrationReport {
            source_schema_version: Some(source),
            target_schema_version: target,
            summary,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaticAssetSchemaMigrator {
    current: u32,
    minimum: u32,
}

impl StaticAssetSchemaMigrator {
    pub const fn new(current: u32, minimum: u32) -> Self {
        Self { current, minimum }
    }
}

impl AssetSchemaMigrator for StaticAssetSchemaMigrator {
    fn current_schema_version(&self) -> u32 {
        self.current
    }

    fn minimum_supported_schema_version(&self) -> u32 {
        self.minimum
    }
}
