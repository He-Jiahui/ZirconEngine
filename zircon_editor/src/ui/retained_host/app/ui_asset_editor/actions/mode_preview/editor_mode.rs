use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::asset_editor::UiAssetEditorMode;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_editor_mode_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "mode.design" => self.set_ui_asset_editor_mode(instance_id, UiAssetEditorMode::Design),
            "mode.split" => self.set_ui_asset_editor_mode(instance_id, UiAssetEditorMode::Split),
            "mode.source" => self.set_ui_asset_editor_mode(instance_id, UiAssetEditorMode::Source),
            "mode.preview" => {
                self.set_ui_asset_editor_mode(instance_id, UiAssetEditorMode::Preview)
            }
            _ => UiAssetActionDispatch::Unhandled,
        }
    }

    fn set_ui_asset_editor_mode(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiAssetEditorMode,
    ) -> UiAssetActionDispatch {
        UiAssetActionDispatch::handled(
            self.editor_manager
                .set_ui_asset_editor_mode(instance_id, mode),
        )
    }
}
