use super::*;
use crate::ui::asset_editor::session::promotion_state::reference_asset_id;

impl UiAssetEditorSession {
    pub(in crate::ui::asset_editor::session) fn existing_external_widget_source(
        &self,
        asset_id: &str,
    ) -> Result<Option<String>, UiAssetEditorSessionError> {
        self.existing_external_asset_source(
            asset_id,
            self.compiler_imports
                .widgets
                .iter()
                .find_map(|(reference, document)| {
                    (reference_asset_id(reference) == asset_id).then_some(document)
                }),
        )
    }

    pub(in crate::ui::asset_editor::session) fn existing_external_style_source(
        &self,
        asset_id: &str,
    ) -> Result<Option<String>, UiAssetEditorSessionError> {
        self.existing_external_asset_source(asset_id, self.compiler_imports.styles.get(asset_id))
    }

    fn existing_external_asset_source(
        &self,
        asset_id: &str,
        imported_document: Option<&UiAssetDocument>,
    ) -> Result<Option<String>, UiAssetEditorSessionError> {
        if self.route.asset_id == asset_id {
            return Ok(Some(self.last_valid_source_text.clone()));
        }

        imported_document
            .map(|document| {
                if asset_id.ends_with(".zui") {
                    serialize_v2_projection_document(document, None)
                } else {
                    serialize_document(document)
                }
            })
            .transpose()
    }
}
