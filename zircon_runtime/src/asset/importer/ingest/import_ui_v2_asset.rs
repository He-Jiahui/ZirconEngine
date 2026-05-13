use crate::asset::assets::{ImportedAsset, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_ui_v2_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    if let Ok(asset) = UiV2ViewAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::UiV2View(asset),
        ));
    }
    if let Ok(asset) = UiV2ComponentAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::UiV2Component(asset),
        ));
    }
    if let Ok(asset) = UiV2StyleAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::UiV2Style(asset),
        ));
    }
    Err(AssetImportError::Parse(format!(
        "parse ui v2 asset toml {}: unsupported or mismatched [asset.kind]",
        context.source_path.display()
    )))
}
