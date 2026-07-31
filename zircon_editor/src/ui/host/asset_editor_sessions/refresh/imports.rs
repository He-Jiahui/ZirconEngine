use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::template::UiAssetKind;

use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::imports::{UiAssetImportResolution, UiAssetImportTraversal};
use super::super::UiAssetStaleImportDiagnostic;

impl EditorUiHost {
    pub(super) fn apply_import_ui_asset_changes(
        &self,
        _changed_asset_ids: &BTreeSet<String>,
        import_instances: &BTreeSet<ViewInstanceId>,
    ) -> Result<BTreeSet<ViewInstanceId>, EditorError> {
        let entries = {
            let sessions = self.lock_ui_asset_sessions();
            import_instances
                .iter()
                .filter_map(|instance_id| {
                    sessions.get(instance_id).map(|entry| {
                        let (widgets, styles) = entry.session.import_references();
                        (instance_id.clone(), widgets, styles)
                    })
                })
                .collect::<Vec<_>>()
        };

        let mut sync_instances = BTreeSet::new();
        for (instance_id, widget_refs, style_refs) in entries {
            let (resolution, errors) =
                self.collect_ui_asset_imports_lossy(&widget_refs, &style_refs);
            let UiAssetImportResolution {
                documents,
                dependencies,
            } = resolution;
            let mut dependency_generation = self.lock_ui_asset_dependency_generation();
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(&instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            if errors.is_empty() {
                entry
                    .session
                    .replace_resolved_imports(
                        documents.widgets,
                        documents.styles,
                        documents.v2_widgets,
                        documents.v2_styles,
                    )
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                entry.stale_imports.clear();
            } else {
                entry.stale_imports = errors
                    .into_iter()
                    .map(|error| (error.reference.clone(), error))
                    .collect::<BTreeMap<_, _>>();
            }
            dependency_generation.replace_dependencies(instance_id.clone(), dependencies);
            let _ = sync_instances.insert(instance_id);
        }
        Ok(sync_instances)
    }

    pub(in crate::ui::host::asset_editor_sessions) fn collect_ui_asset_imports_lossy(
        &self,
        widget_refs: &[String],
        style_refs: &[String],
    ) -> (UiAssetImportResolution, Vec<UiAssetStaleImportDiagnostic>) {
        let mut traversal = UiAssetImportTraversal::default();
        let mut errors = Vec::new();

        for reference in widget_refs {
            if let Err(message) = self.try_collect_ui_asset_import_document(
                reference,
                UiAssetKind::Widget,
                &mut traversal,
            ) {
                errors.push(UiAssetStaleImportDiagnostic {
                    reference: normalize_ui_asset_asset_id(reference).to_string(),
                    message,
                });
            }
        }
        for reference in style_refs {
            if let Err(message) = self.try_collect_ui_asset_import_document(
                reference,
                UiAssetKind::Style,
                &mut traversal,
            ) {
                errors.push(UiAssetStaleImportDiagnostic {
                    reference: normalize_ui_asset_asset_id(reference).to_string(),
                    message,
                });
            }
        }

        (traversal.finish_resolution(), errors)
    }
}
