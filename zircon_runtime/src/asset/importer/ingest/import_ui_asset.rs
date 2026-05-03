use crate::asset::assets::{ImportedAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetSchemaMigrationReport,
};
use crate::ui::template::UiAssetLoader;

pub(crate) fn import_ui_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let migration = UiAssetLoader::load_toml_str_with_migration_report(&document)
        .map_err(|error| AssetImportError::Parse(error.to_string()))?;
    let migration_report = AssetSchemaMigrationReport {
        source_schema_version: migration.report.source_schema_version,
        target_schema_version: migration.report.target_schema_version,
        summary: format!(
            "ui asset schema {:?}; steps: {:?}",
            migration.report.source_kind, migration.report.steps
        ),
    };

    if let Ok(asset) = UiLayoutAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiLayout(asset))
            .with_migration_report(migration_report));
    }
    if let Ok(asset) = UiWidgetAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiWidget(asset))
            .with_migration_report(migration_report));
    }
    if let Ok(asset) = UiStyleAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiStyle(asset))
            .with_migration_report(migration_report));
    }
    Err(AssetImportError::Parse(format!(
        "parse ui asset toml {}: unsupported or mismatched [asset.kind]",
        context.source_path.display()
    )))
}
