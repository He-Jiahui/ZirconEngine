use crate::asset::assets::{ImportedAsset, UiLayoutAsset, UiStyleAsset, UiWidgetAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_ui_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    if let Ok(asset) = UiLayoutAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiLayout(asset)));
    }
    if let Ok(asset) = UiWidgetAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiWidget(asset)));
    }
    if let Ok(asset) = UiStyleAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(ImportedAsset::UiStyle(asset)));
    }
    Err(AssetImportError::Parse(format!(
        "parse ui asset toml {}: unsupported or mismatched [asset.kind]",
        context.source_path.display()
    )))
}
