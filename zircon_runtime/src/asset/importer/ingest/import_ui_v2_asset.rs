use crate::asset::assets::{ImportedAsset, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

pub(crate) fn import_ui_v2_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_text()?;
    let document = crate::ui::v2::UiZuiAssetLoader::load_zui_str(&source).map_err(|source| {
        AssetImportError::UiV2Document {
            context: "parse .zui ui asset",
            source: source.into(),
        }
    })?;
    let imported = match document.asset.kind {
        UiV2AssetKind::View => ImportedAsset::UiV2View(UiV2ViewAsset { document }),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => {
            ImportedAsset::UiV2Style(UiV2StyleAsset { document })
        }
        UiV2AssetKind::Component => ImportedAsset::UiV2Component(UiV2ComponentAsset { document }),
    };
    Ok(AssetImportOutcome::new(context.uri.clone(), imported))
}
