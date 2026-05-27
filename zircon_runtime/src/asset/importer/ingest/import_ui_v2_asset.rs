use crate::asset::assets::{ImportedAsset, UiV2StyleAsset, UiV2ViewAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};
use crate::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

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
    if let Ok(asset) = UiV2StyleAsset::from_toml_str(&document) {
        return Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::UiV2Style(asset),
        ));
    }
    if let Ok(parsed) = UiV2AssetLoader::load_toml_str(&document) {
        if parsed.asset.kind == UiV2AssetKind::Component {
            return Err(AssetImportError::Parse(format!(
                "parse ui v2 asset toml {}: component documents must use `.zui`, not `.v2.ui.toml`",
                context.source_path.display()
            )));
        }
    }
    Err(AssetImportError::Parse(format!(
        "parse ui v2 asset toml {}: unsupported or mismatched [asset.kind]",
        context.source_path.display()
    )))
}
