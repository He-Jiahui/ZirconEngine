use std::collections::BTreeSet;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime_interface::ui::template::UiAssetKind;

use super::imports::UiAssetImportDocuments;

impl EditorUiHost {
    pub(super) fn hydrate_ui_asset_editor_imports(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let (widget_refs, style_refs) = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session.import_references()
        };
        let mut documents = UiAssetImportDocuments::default();
        let mut visited = BTreeSet::new();
        for reference in widget_refs {
            self.collect_ui_asset_import_document(
                &reference,
                UiAssetKind::Widget,
                &mut documents,
                &mut visited,
            )?;
        }
        for reference in style_refs {
            self.collect_ui_asset_import_document(
                &reference,
                UiAssetKind::Style,
                &mut documents,
                &mut visited,
            )?;
        }

        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .replace_resolved_imports(
                documents.widgets,
                documents.styles,
                documents.v2_widgets,
                documents.v2_styles,
            )
            .map_err(|error| EditorError::UiAsset(error.to_string()))
    }
}
