use std::collections::{BTreeMap, BTreeSet};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetKind};

impl EditorUiHost {
    pub(super) fn hydrate_ui_asset_editor_imports(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let (widget_refs, style_refs) = {
            let sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session.import_references()
        };
        let mut widget_docs = BTreeMap::<String, UiAssetDocument>::new();
        let mut style_docs = BTreeMap::<String, UiAssetDocument>::new();
        let mut visited = BTreeSet::new();
        for reference in widget_refs {
            self.collect_ui_asset_import_document(
                &reference,
                UiAssetKind::Widget,
                &mut widget_docs,
                &mut style_docs,
                &mut visited,
            )?;
        }
        for reference in style_refs {
            self.collect_ui_asset_import_document(
                &reference,
                UiAssetKind::Style,
                &mut widget_docs,
                &mut style_docs,
                &mut visited,
            )?;
        }

        let mut sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .replace_imports(widget_docs, style_docs)
            .map_err(|error| EditorError::UiAsset(error.to_string()))
    }
}
