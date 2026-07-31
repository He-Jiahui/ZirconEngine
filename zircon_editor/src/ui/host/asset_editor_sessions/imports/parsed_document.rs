use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::template::UiAssetDocument;
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;

use super::super::{is_v2_backed_ui_asset_id, parse_ui_asset_document_source};

#[derive(Clone, Debug)]
pub(super) struct ParsedUiAssetImportDocument {
    pub(super) document: UiAssetDocument,
    pub(super) v2_document: Option<UiV2AssetDocument>,
}

pub(super) fn parse_ui_asset_import_source(
    normalized_id: &str,
    source: &str,
) -> Result<ParsedUiAssetImportDocument, String> {
    if is_v2_backed_ui_asset_id(normalized_id) {
        let v2_document =
            UiZuiAssetLoader::load_zui_str(source).map_err(|error| error.to_string())?;
        let document = crate::ui::asset_editor::project_v2_document_to_authoring(&v2_document)
            .map_err(|error| error.to_string())?;
        return Ok(ParsedUiAssetImportDocument {
            document,
            v2_document: Some(v2_document),
        });
    }

    let document = parse_ui_asset_document_source(source).map_err(|error| error.to_string())?;
    Ok(ParsedUiAssetImportDocument {
        document,
        v2_document: None,
    })
}
