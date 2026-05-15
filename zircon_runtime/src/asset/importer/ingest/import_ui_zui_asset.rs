use crate::asset::assets::{ImportedAsset, UiV2ComponentAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_ui_zui_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let asset = UiV2ComponentAsset::from_zui_str(&document).map_err(|error| {
        AssetImportError::Parse(format!(
            "parse .zui component asset {}: {error}",
            context.source_path.display()
        ))
    })?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::UiV2Component(asset),
    ))
}
