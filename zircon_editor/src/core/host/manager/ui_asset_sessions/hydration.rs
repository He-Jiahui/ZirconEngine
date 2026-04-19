use std::collections::{BTreeMap, BTreeSet};

use crate::view::ViewInstanceId;
use crate::{EditorError, EditorManager};
use zircon_runtime::ui::template::UiAssetDocument;

impl EditorManager {
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
                zircon_runtime::ui::template::UiAssetKind::Widget,
                &mut widget_docs,
                &mut style_docs,
                &mut visited,
            )?;
        }
        for reference in style_refs {
            self.collect_ui_asset_import_document(
                &reference,
                zircon_runtime::ui::template::UiAssetKind::Style,
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
