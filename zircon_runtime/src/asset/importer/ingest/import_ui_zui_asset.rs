use super::ui_v2_document_import::imported_asset_from_ui_v2_document;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};
use crate::ui::v2::UiZuiAssetLoader;

pub(crate) fn import_ui_zui_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let parsed = UiZuiAssetLoader::load_zui_str(&document).map_err(|source| {
        AssetImportError::UiV2Document {
            context: "parse .zui ui asset",
            source: source.into(),
        }
    })?;
    let imported = imported_asset_from_ui_v2_document(parsed).map_err(|source| {
        AssetImportError::UiV2Document {
            context: "parse .zui ui asset",
            source,
        }
    })?;
    Ok(AssetImportOutcome::new(context.uri.clone(), imported))
}
