use std::collections::BTreeMap;

use super::super::super::RetainedEditorHost;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn collect_editor_panes(
        &self,
        ui_asset_instance_ids: Vec<ViewInstanceId>,
        animation_instance_ids: Vec<ViewInstanceId>,
    ) -> Result<
        (
            BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation>,
            BTreeMap<String, crate::ui::animation_editor::AnimationEditorPanePresentation>,
        ),
        crate::ui::host::EditorError,
    > {
        let mut ui_asset_panes = BTreeMap::new();
        let mut animation_panes = BTreeMap::new();
        for instance_id in ui_asset_instance_ids {
            let presentation = self
                .editor_manager
                .enter()
                .map_err(crate::ui::host::EditorError::from)
                .and_then(|manager| manager.ui_asset_editor_pane_presentation(&instance_id))?;
            ui_asset_panes.insert(instance_id.0, presentation);
        }
        for instance_id in animation_instance_ids {
            let presentation = self
                .editor_manager
                .enter()
                .map_err(crate::ui::host::EditorError::from)
                .and_then(|manager| manager.animation_editor_pane_presentation(&instance_id))?;
            animation_panes.insert(instance_id.0, presentation);
        }
        Ok((ui_asset_panes, animation_panes))
    }
}

#[cfg(test)]
#[path = "tests/editor_panes_performance_tests.rs"]
mod performance_tests;
