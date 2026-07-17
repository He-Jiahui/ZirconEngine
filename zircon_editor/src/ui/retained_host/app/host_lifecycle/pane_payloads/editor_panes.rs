use std::collections::BTreeMap;

use super::super::super::RetainedEditorHost;
use crate::ui::workbench::view::ViewInstance;

impl RetainedEditorHost {
    pub(super) fn collect_ui_asset_panes(
        &self,
        instances: &[ViewInstance],
    ) -> BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation> {
        instances
            .iter()
            .filter(|instance| instance.descriptor_id.0 == "editor.ui_asset")
            .filter_map(|instance| {
                self.editor_manager
                    .ui_asset_editor_pane_presentation(&instance.instance_id)
                    .ok()
                    .map(|presentation| (instance.instance_id.0.clone(), presentation))
            })
            .collect()
    }

    pub(super) fn collect_animation_editor_panes(
        &self,
        instances: &[ViewInstance],
    ) -> BTreeMap<String, crate::ui::animation_editor::AnimationEditorPanePresentation> {
        instances
            .iter()
            .filter(|instance| {
                matches!(
                    instance.descriptor_id.0.as_str(),
                    "editor.animation_sequence" | "editor.animation_graph"
                )
            })
            .filter_map(|instance| {
                self.editor_manager
                    .animation_editor_pane_presentation(&instance.instance_id)
                    .ok()
                    .map(|presentation| (instance.instance_id.0.clone(), presentation))
            })
            .collect()
    }
}
