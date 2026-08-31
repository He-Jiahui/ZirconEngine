use toml::Value;

use super::flat_nodes;
use crate::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetMigrationOutcome, UiAssetMigrationReport,
    UiAssetMigrationStep, UiAssetSchemaSourceKind, UiAssetSchemaVersionPolicy,
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

#[derive(Default)]
pub struct UiAssetSchemaMigrator;

impl UiAssetSchemaMigrator {
    pub fn migrate_toml_str(input: &str) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let value: Value =
            toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
        let Some(table) = value.as_table() else {
            return Err(UiAssetError::ParseToml(
                "ui asset source must be a TOML table".to_string(),
            ));
        };

        let header = validate_owned_source_header(parse_asset_header_value(&value)?)?;
        if table.contains_key("nodes") {
            return Self::migrate_flat_asset(value, header);
        }
        Self::migrate_tree_asset(value)
    }

    fn migrate_tree_asset(value: Value) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let mut document: UiAssetDocument = value
            .try_into()
            .map_err(|error: toml::de::Error| UiAssetError::ParseToml(error.to_string()))?;
        reject_unsupported_source_version(&document.asset)?;

        let source_version = document.asset.version;
        let source_kind =
            if UiAssetSchemaVersionPolicy::requires_source_schema_migration(source_version) {
                UiAssetSchemaSourceKind::OlderTree
            } else {
                UiAssetSchemaSourceKind::CurrentTree
            };
        let mut report = UiAssetMigrationReport::new(source_kind, Some(source_version));
        push_version_bump_step(&mut report, source_version);
        document.asset.version = UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;
        document.validate_tree_authority()?;
        report.push_step(UiAssetMigrationStep::CurrentTreeValidated);

        Ok(UiAssetMigrationOutcome { document, report })
    }

    fn migrate_flat_asset(
        value: Value,
        header: UiAssetHeader,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let mut document = flat_nodes::migrate_flat_value(value)
            .map_err(|error| schema_migration_failed(&header.id, error))?;
        let source_version = header.version;
        let mut report = UiAssetMigrationReport::new(
            UiAssetSchemaSourceKind::FlatNodeTable,
            Some(source_version),
        );
        report.push_step(UiAssetMigrationStep::FlatNodeTableMaterialized);
        push_version_bump_step(&mut report, source_version);
        document.asset.version = UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;
        document
            .validate_tree_authority()
            .map_err(|error| schema_migration_failed(&header.id, error))?;

        Ok(UiAssetMigrationOutcome { document, report })
    }
}

fn parse_asset_header_value(value: &Value) -> Result<UiAssetHeader, UiAssetError> {
    value
        .as_table()
        .and_then(|table| table.get("asset"))
        .cloned()
        .ok_or_else(|| UiAssetError::ParseToml("ui asset source is missing [asset]".to_string()))?
        .try_into()
        .map_err(|error: toml::de::Error| UiAssetError::ParseToml(error.to_string()))
}

fn reject_unsupported_source_version(header: &UiAssetHeader) -> Result<(), UiAssetError> {
    if !UiAssetSchemaVersionPolicy::is_supported_source_schema(header.version) {
        return Err(UiAssetError::UnsupportedSchemaVersion {
            asset_id: header.id.clone(),
            version: header.version,
            current: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn validate_owned_source_header(header: UiAssetHeader) -> Result<UiAssetHeader, UiAssetError> {
    if !UiAssetSchemaVersionPolicy::is_supported_source_schema(header.version) {
        return Err(UiAssetError::UnsupportedSchemaVersion {
            asset_id: header.id,
            version: header.version,
            current: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        });
    }
    Ok(header)
}

fn push_version_bump_step(report: &mut UiAssetMigrationReport, source_version: u32) {
    if UiAssetSchemaVersionPolicy::requires_source_schema_migration(source_version) {
        report.push_step(UiAssetMigrationStep::SourceVersionBumped {
            from: source_version,
            to: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        });
    }
}

fn schema_migration_failed(asset_id: &str, error: UiAssetError) -> UiAssetError {
    UiAssetError::SchemaMigrationFailed {
        asset_id: asset_id.to_string(),
        detail: error.to_string(),
    }
}

#[cfg(test)]
#[path = "migrator/owned_header_tests.rs"]
mod owned_header_tests;
