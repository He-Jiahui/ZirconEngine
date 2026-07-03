use crate::asset::assets::{
    ImportedAsset, UiV2AssetDocumentError, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset,
};
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetKind};

pub(crate) fn imported_asset_from_ui_v2_document(
    document: UiV2AssetDocument,
) -> Result<ImportedAsset, UiV2AssetDocumentError> {
    Ok(match document.asset.kind {
        UiV2AssetKind::View => ImportedAsset::UiV2View(UiV2ViewAsset { document }),
        UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens => {
            ImportedAsset::UiV2Style(UiV2StyleAsset { document })
        }
        UiV2AssetKind::Component => ImportedAsset::UiV2Component(UiV2ComponentAsset { document }),
    })
}
