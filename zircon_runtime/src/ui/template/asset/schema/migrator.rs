use serde::Deserialize;
use toml::Value;

use super::flat_nodes;
use super::legacy_template;
use crate::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiAssetHeader, UiAssetMigrationOutcome, UiAssetMigrationReport,
    UiAssetMigrationStep, UiAssetSchemaSourceKind, UiAssetSchemaVersionPolicy, UiTemplateDocument,
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

const DEFAULT_LEGACY_TEMPLATE_ASSET_ID: &str = "legacy.template_fixture";
const DEFAULT_LEGACY_TEMPLATE_DISPLAY_NAME: &str = "Legacy Template Fixture";

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

        if table.contains_key("asset") {
            let header = parse_asset_header(input)?;
            reject_unsupported_source_version(&header)?;
            if table.contains_key("nodes") {
                return Self::migrate_flat_asset(input, header);
            }
            return Self::migrate_tree_asset(input);
        }

        Self::migrate_legacy_template_str(
            DEFAULT_LEGACY_TEMPLATE_ASSET_ID,
            DEFAULT_LEGACY_TEMPLATE_DISPLAY_NAME,
            input,
        )
    }

    pub fn migrate_legacy_template_str(
        asset_id: impl Into<String>,
        display_name: impl Into<String>,
        input: &str,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let legacy: UiTemplateDocument =
            toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
        Self::migrate_legacy_template_document(asset_id, display_name, &legacy)
    }

    pub fn migrate_legacy_template_document(
        asset_id: impl Into<String>,
        display_name: impl Into<String>,
        document: &UiTemplateDocument,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let asset_id = asset_id.into();
        let display_name = display_name.into();
        let source_version = document.version;
        if !UiAssetSchemaVersionPolicy::is_supported_source_schema(source_version) {
            return Err(UiAssetError::UnsupportedSchemaVersion {
                asset_id,
                version: source_version,
                current: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            });
        }

        let mut converted = legacy_template::convert_legacy_template_document(
            asset_id.clone(),
            display_name,
            document,
        )
        .map_err(|error| schema_migration_failed(&asset_id, error))?;
        converted.asset.version = UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;
        converted
            .validate_tree_authority()
            .map_err(|error| schema_migration_failed(&asset_id, error))?;

        let mut report = UiAssetMigrationReport::new(
            UiAssetSchemaSourceKind::LegacyTemplateFixture,
            Some(source_version),
        );
        report.push_step(UiAssetMigrationStep::LegacyTemplateConverted);
        push_version_bump_step(&mut report, source_version);

        Ok(UiAssetMigrationOutcome {
            document: converted,
            report,
        })
    }

    fn migrate_tree_asset(input: &str) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let mut document: UiAssetDocument =
            toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
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
        input: &str,
        header: UiAssetHeader,
    ) -> Result<UiAssetMigrationOutcome, UiAssetError> {
        let mut document = flat_nodes::migrate_flat_toml_str(input)
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

fn parse_asset_header(input: &str) -> Result<UiAssetHeader, UiAssetError> {
    let probe: AssetHeaderProbe =
        toml::from_str(input).map_err(|error| UiAssetError::ParseToml(error.to_string()))?;
    Ok(probe.asset)
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

#[derive(Deserialize)]
struct AssetHeaderProbe {
    pub asset: UiAssetHeader,
}
