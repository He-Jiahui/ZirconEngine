use crate::asset::assets::{ImportedAsset, UiIconAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_ui_icon_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let asset = UiIconAsset::from_toml_str(&document).map_err(|source| {
        AssetImportError::UiIconDocument {
            context: "parse ui icon asset",
            source,
        }
    })?;

    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::UiIcon(asset),
    ))
}
