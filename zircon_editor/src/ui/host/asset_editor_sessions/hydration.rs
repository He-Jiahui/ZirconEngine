use std::collections::BTreeMap;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use super::UiAssetStaleImportDiagnostic;
use crate::ui::workbench::view::ViewInstanceId;

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
        // Initial hydration is document-lossy but edge-lossless: a missing,
        // unreadable, or invalid import keeps the last-good authoring session
        // open and must still enter the reverse index for later recovery.
        let (resolution, errors) = self.collect_ui_asset_imports_lossy(&widget_refs, &style_refs);
        let documents = resolution.documents;
        let dependencies = resolution.dependencies;

        // Commit resolved imports and their reverse edges in one dependency
        // epoch. The watcher commit path uses the same dependency -> session
        // lock order, so no worker can validate between these two mutations.
        let mut dependency_generation = self.lock_ui_asset_dependency_generation();
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let mut diagnostics = errors
            .into_iter()
            .map(|diagnostic| (diagnostic.reference.clone(), diagnostic))
            .collect::<BTreeMap<_, _>>();
        if diagnostics.is_empty() {
            if let Err(error) = entry.session.replace_resolved_imports(
                documents.widgets,
                documents.styles,
                documents.v2_widgets,
                documents.v2_styles,
            ) {
                let reference = entry.session.route().asset_id.clone();
                diagnostics.insert(
                    reference.clone(),
                    UiAssetStaleImportDiagnostic {
                        reference,
                        message: error.to_string(),
                    },
                );
            }
        }
        entry.stale_imports = diagnostics;
        dependency_generation.replace_dependencies(instance_id.clone(), dependencies);
        Ok(())
    }
}
